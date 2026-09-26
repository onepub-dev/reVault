//! Pre-match block splitter ported from `zstd_preSplit.c`.

use super::params::Strategy;
use crate::common::MAX_BLOCK_SIZE;

const BLOCK_SIZE_MAX: usize = MAX_BLOCK_SIZE as usize;
const CHUNK_SIZE: usize = 8 * 1024;
const SEGMENT_SIZE: usize = 512;
const THRESHOLD_PENALTY_RATE: u64 = 16;
const THRESHOLD_BASE: u64 = THRESHOLD_PENALTY_RATE - 2;
const THRESHOLD_PENALTY: i32 = 3;
const HASH_LENGTH: usize = 2;
const HASH_LOG_MAX: usize = 10;
const HASH_TABLE_SIZE: usize = 1 << HASH_LOG_MAX;
const KNUTH: u32 = 0x9e37_79b9;

#[derive(Clone, Debug)]
pub(super) struct FrameProgress {
    consumed_src_size: usize,
    produced_size: usize,
}

impl FrameProgress {
    pub(super) fn new() -> Self {
        // C adds the frame header to producedCSize only after frame chunking,
        // so the pre-split savings gate starts from zero for a one-shot frame.
        Self {
            consumed_src_size: 0,
            produced_size: 0,
        }
    }

    pub(super) fn next_block_size(
        &self,
        remaining: &[u8],
        strategy: Strategy,
        block_size_max: usize,
    ) -> usize {
        optimal_block_size(
            remaining,
            block_size_max,
            0,
            strategy,
            self.consumed_src_size as i64 - self.produced_size as i64,
        )
    }

    pub(super) fn record_block(&mut self, src_size: usize, produced_size: usize) {
        self.consumed_src_size += src_size;
        self.produced_size += produced_size;
    }
}

fn optimal_block_size(
    src: &[u8],
    block_size_max: usize,
    split_level: i32,
    strategy: Strategy,
    savings: i64,
) -> usize {
    if src.len() < BLOCK_SIZE_MAX || block_size_max < BLOCK_SIZE_MAX {
        return src.len().min(block_size_max);
    }

    if savings < 3 {
        return BLOCK_SIZE_MAX;
    }

    let split_level = match split_level {
        1 => return BLOCK_SIZE_MAX,
        0 => default_split_level(strategy),
        2..=6 => split_level - 2,
        _ => {
            debug_assert!((0..=6).contains(&split_level));
            0
        }
    };

    split_block(&src[..BLOCK_SIZE_MAX], split_level)
}

fn default_split_level(strategy: Strategy) -> i32 {
    const SPLIT_LEVELS: [i32; 10] = [0, 0, 1, 2, 2, 3, 3, 4, 4, 4];
    SPLIT_LEVELS[strategy as usize]
}

fn split_block(block: &[u8], level: i32) -> usize {
    debug_assert_eq!(block.len(), BLOCK_SIZE_MAX);
    debug_assert!((0..=4).contains(&level));

    if level == 0 {
        split_block_from_borders(block)
    } else {
        split_block_by_chunks(block, level - 1)
    }
}

fn split_block_by_chunks(block: &[u8], level: i32) -> usize {
    debug_assert_eq!(block.len(), BLOCK_SIZE_MAX);
    debug_assert!((0..=3).contains(&level));

    let (sampling_rate, hash_log) = match level {
        0 => (43, 8),
        1 => (11, 9),
        2 => (5, 10),
        3 => (1, 10),
        _ => unreachable!("level is checked by debug_assert"),
    };

    let mut past_events = Fingerprint::record(&block[..CHUNK_SIZE], sampling_rate, hash_log);
    let mut penalty = THRESHOLD_PENALTY;

    for pos in (CHUNK_SIZE..=BLOCK_SIZE_MAX - CHUNK_SIZE).step_by(CHUNK_SIZE) {
        let new_events =
            Fingerprint::record(&block[pos..pos + CHUNK_SIZE], sampling_rate, hash_log);
        if compare_fingerprints(&past_events, &new_events, penalty, hash_log) {
            return pos;
        }
        past_events.merge(&new_events);
        if penalty > 0 {
            penalty -= 1;
        }
    }

    BLOCK_SIZE_MAX
}

fn split_block_from_borders(block: &[u8]) -> usize {
    debug_assert_eq!(block.len(), BLOCK_SIZE_MAX);

    let past_events = Fingerprint::histogram(&block[..SEGMENT_SIZE]);
    let new_events = Fingerprint::histogram(&block[BLOCK_SIZE_MAX - SEGMENT_SIZE..]);
    if !compare_fingerprints(&past_events, &new_events, 0, 8) {
        return BLOCK_SIZE_MAX;
    }

    let middle_start = BLOCK_SIZE_MAX / 2 - SEGMENT_SIZE / 2;
    let middle_events = Fingerprint::histogram(&block[middle_start..middle_start + SEGMENT_SIZE]);
    let dist_from_begin = fingerprint_distance(&past_events, &middle_events, 8);
    let dist_from_end = fingerprint_distance(&new_events, &middle_events, 8);
    let min_distance = (SEGMENT_SIZE * SEGMENT_SIZE / 3) as u64;

    if abs_diff(dist_from_begin, dist_from_end) < min_distance {
        64 * 1024
    } else if dist_from_begin > dist_from_end {
        32 * 1024
    } else {
        96 * 1024
    }
}

#[derive(Clone, Debug)]
struct Fingerprint {
    events: [u32; HASH_TABLE_SIZE],
    nb_events: usize,
}

impl Fingerprint {
    fn record(src: &[u8], sampling_rate: usize, hash_log: usize) -> Self {
        debug_assert!(src.len() >= HASH_LENGTH);
        debug_assert!((8..=HASH_LOG_MAX).contains(&hash_log));

        let mut fp = Self::default();
        match (sampling_rate, hash_log) {
            (43, 8) => fp.record_hash8::<43>(src),
            (11, 9) => fp.record_hash2::<11, 9>(src),
            (5, 10) => fp.record_hash2::<5, 10>(src),
            (1, 10) => fp.record_hash2::<1, 10>(src),
            _ => fp.record_generic(src, sampling_rate, hash_log),
        }
        fp
    }

    fn record_hash8<const SAMPLING_RATE: usize>(&mut self, src: &[u8]) {
        let limit = src.len() - HASH_LENGTH + 1;
        let mut pos = 0;
        while pos < limit {
            self.events[usize::from(src[pos])] += 1;
            pos += SAMPLING_RATE;
        }
        self.nb_events = limit / SAMPLING_RATE;
    }

    fn record_hash2<const SAMPLING_RATE: usize, const HASH_LOG: usize>(&mut self, src: &[u8]) {
        debug_assert!((9..=HASH_LOG_MAX).contains(&HASH_LOG));

        if SAMPLING_RATE == 1 {
            self.record_hash2_rate1::<HASH_LOG>(src);
            return;
        }

        let limit = src.len() - HASH_LENGTH + 1;
        let mut pos = 0;
        while pos < limit {
            let value = u32::from(u16::from_le_bytes([src[pos], src[pos + 1]]));
            let hash = (value.wrapping_mul(KNUTH) >> (32 - HASH_LOG)) as usize;
            self.events[hash] += 1;
            pos += SAMPLING_RATE;
        }
        self.nb_events = limit / SAMPLING_RATE;
    }

    fn record_hash2_rate1<const HASH_LOG: usize>(&mut self, src: &[u8]) {
        debug_assert!((9..=HASH_LOG_MAX).contains(&HASH_LOG));

        let limit = src.len() - HASH_LENGTH + 1;
        let mut pos = 0;
        let mut value = u32::from(u16::from_le_bytes([src[0], src[1]]));
        while pos < limit {
            let hash = (value.wrapping_mul(KNUTH) >> (32 - HASH_LOG)) as usize;
            self.events[hash] += 1;
            pos += 1;
            if pos < limit {
                value = (value >> 8) | (u32::from(src[pos + 1]) << 8);
            }
        }
        self.nb_events = limit;
    }

    fn record_generic(&mut self, src: &[u8], sampling_rate: usize, hash_log: usize) {
        let limit = src.len() - HASH_LENGTH + 1;
        for pos in (0..limit).step_by(sampling_rate) {
            self.events[hash2(&src[pos..], hash_log)] += 1;
        }
        self.nb_events = limit / sampling_rate;
    }

    fn histogram(src: &[u8]) -> Self {
        let mut fp = Self::default();
        for &byte in src {
            fp.events[usize::from(byte)] += 1;
        }
        fp.nb_events = src.len();
        fp
    }

    fn merge(&mut self, other: &Self) {
        for (acc, value) in self.events.iter_mut().zip(other.events.iter().copied()) {
            *acc += value;
        }
        self.nb_events += other.nb_events;
    }
}

impl Default for Fingerprint {
    fn default() -> Self {
        Self {
            events: [0; HASH_TABLE_SIZE],
            nb_events: 0,
        }
    }
}

fn hash2(src: &[u8], hash_log: usize) -> usize {
    debug_assert!(src.len() >= HASH_LENGTH);
    debug_assert!((8..=HASH_LOG_MAX).contains(&hash_log));

    if hash_log == 8 {
        usize::from(src[0])
    } else {
        let value = u32::from(u16::from_le_bytes([src[0], src[1]]));
        (value.wrapping_mul(KNUTH) >> (32 - hash_log)) as usize
    }
}

fn compare_fingerprints(
    reference: &Fingerprint,
    new_fingerprint: &Fingerprint,
    penalty: i32,
    hash_log: usize,
) -> bool {
    debug_assert!(reference.nb_events > 0);
    debug_assert!(new_fingerprint.nb_events > 0);

    let p50 = reference.nb_events as u64 * new_fingerprint.nb_events as u64;
    let deviation = fingerprint_distance(reference, new_fingerprint, hash_log);
    let threshold = p50 * (THRESHOLD_BASE + penalty as u64) / THRESHOLD_PENALTY_RATE;
    deviation >= threshold
}

fn fingerprint_distance(left: &Fingerprint, right: &Fingerprint, hash_log: usize) -> u64 {
    debug_assert!(hash_log <= HASH_LOG_MAX);

    let len = 1usize << hash_log;
    (0..len)
        .map(|idx| {
            abs_diff(
                u64::from(left.events[idx]) * right.nb_events as u64,
                u64::from(right.events[idx]) * left.nb_events as u64,
            )
        })
        .sum()
}

fn abs_diff(left: u64, right: u64) -> u64 {
    left.abs_diff(right)
}

#[cfg(test)]
mod tests {
    use alloc::{vec, vec::Vec};

    use super::*;

    #[test]
    fn first_full_block_is_not_split_without_savings() {
        let block = half_and_half_block();

        assert_eq!(
            optimal_block_size(&block, BLOCK_SIZE_MAX, 0, Strategy::BtUltra2, 2),
            BLOCK_SIZE_MAX
        );
    }

    #[test]
    fn default_double_fast_strategy_uses_border_splitter_after_savings() {
        let block = half_and_half_block();

        assert_eq!(
            optimal_block_size(&block, BLOCK_SIZE_MAX, 0, Strategy::DFast, 3),
            64 * 1024
        );
    }

    #[test]
    fn chunk_splitter_finds_abrupt_change_after_first_chunk() {
        let mut block = vec![b'b'; BLOCK_SIZE_MAX];
        block[..CHUNK_SIZE].fill(b'a');

        assert_eq!(split_block_by_chunks(&block, 0), CHUNK_SIZE);
    }

    #[test]
    fn rate1_hash2_record_matches_generic_hash_stream() {
        let data = (0..8192)
            .map(|idx| (idx * 37 + idx / 7) as u8)
            .collect::<Vec<_>>();

        let mut specialized = Fingerprint::default();
        specialized.record_hash2::<1, 10>(&data);
        let mut generic = Fingerprint::default();
        generic.record_generic(&data, 1, 10);

        assert_eq!(specialized.nb_events, generic.nb_events);
        assert_eq!(specialized.events, generic.events);
    }

    #[test]
    fn border_splitter_keeps_uniform_edges_together() {
        let block = vec![b'a'; BLOCK_SIZE_MAX];

        assert_eq!(split_block_from_borders(&block), BLOCK_SIZE_MAX);
    }

    #[test]
    fn frame_progress_tracks_c_savings_gate_inputs() {
        let mut progress = FrameProgress::new();
        progress.record_block(BLOCK_SIZE_MAX, 100);

        assert_eq!(
            progress.consumed_src_size as i64 - progress.produced_size as i64,
            130_972
        );
    }

    fn half_and_half_block() -> Vec<u8> {
        let mut block = vec![b'a'; BLOCK_SIZE_MAX];
        block[BLOCK_SIZE_MAX / 2..].fill(b'b');
        block
    }
}
