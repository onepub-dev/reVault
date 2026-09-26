use alloc::vec::Vec;

use super::table::{
    build_table_from_probabilities, build_table_from_probabilities_with_scratch, FSETable,
    FSETableBuildScratch,
};

pub fn build_table_from_data(
    data: impl Iterator<Item = u8>,
    max_log: u8,
    avoid_0_numbit: bool,
) -> FSETable {
    let mut counts = [0; 256];
    let mut max_symbol = 0;
    for symbol in data {
        let symbol = usize::from(symbol);
        counts[symbol] += 1;
        max_symbol = max_symbol.max(symbol);
    }
    build_table_from_counts(&counts[..=max_symbol], max_log, avoid_0_numbit)
}

pub(crate) fn build_table_from_data_with_scratch(
    data: impl Iterator<Item = u8>,
    max_log: u8,
    avoid_0_numbit: bool,
    scratch: &mut FSETableBuildScratch,
) -> FSETable {
    let mut counts = [0; 256];
    let mut max_symbol = 0;
    for symbol in data {
        let symbol = usize::from(symbol);
        counts[symbol] += 1;
        max_symbol = max_symbol.max(symbol);
    }
    build_table_from_counts_with_scratch(&counts[..=max_symbol], max_log, avoid_0_numbit, scratch)
}

fn build_table_from_counts(counts: &[usize], max_log: u8, avoid_0_numbit: bool) -> FSETable {
    let (probs, acc_log) = normalized_probabilities_from_counts(counts, max_log, avoid_0_numbit);
    build_table_from_probabilities(&probs, acc_log)
}

fn build_table_from_counts_with_scratch(
    counts: &[usize],
    max_log: u8,
    avoid_0_numbit: bool,
    scratch: &mut FSETableBuildScratch,
) -> FSETable {
    let mut probabilities = [0_i32; 256];
    let acc_log = normalized_probabilities_from_counts_into(
        counts,
        max_log,
        avoid_0_numbit,
        &mut probabilities[..counts.len()],
    );
    build_table_from_probabilities_with_scratch(&probabilities[..counts.len()], acc_log, scratch)
}

pub(crate) fn normalized_probabilities_from_counts_into(
    counts: &[usize],
    max_log: u8,
    avoid_0_numbit: bool,
    probabilities: &mut [i32],
) -> u8 {
    assert_eq!(counts.len(), probabilities.len());
    if max_log <= 6 {
        return old_normalize_counts_into(counts, max_log, avoid_0_numbit, probabilities);
    }

    let total = counts.iter().sum::<usize>();
    assert!(total > 0);
    let max_symbol = counts.iter().rposition(|count| *count > 0).unwrap_or(0);
    let acc_log = optimal_table_log(max_log, total, max_symbol);
    let low_prob_count = if total >= 2048 { -1 } else { 1 };
    if normalize_counts_with_total_into(counts, total, acc_log, low_prob_count, probabilities) {
        acc_log
    } else {
        old_normalize_counts_into(counts, max_log, avoid_0_numbit, probabilities)
    }
}

pub(crate) fn normalized_probabilities_from_counts(
    counts: &[usize],
    max_log: u8,
    avoid_0_numbit: bool,
) -> (Vec<i32>, u8) {
    let mut probabilities = alloc::vec![0_i32; counts.len()];
    let acc_log = normalized_probabilities_from_counts_into(
        counts,
        max_log,
        avoid_0_numbit,
        &mut probabilities,
    );
    (probabilities, acc_log)
}

pub(crate) fn normalize_counts_with_table_log(
    counts: &[usize],
    total: usize,
    table_log: u8,
    max_log: u8,
    low_prob_count: i32,
    avoid_0_numbit: bool,
) -> (Vec<i32>, u8) {
    normalize_counts_with_total(counts, total, table_log, low_prob_count)
        .map(|probs| (probs, table_log))
        .unwrap_or_else(|| old_normalize_counts(counts, max_log, avoid_0_numbit))
}

pub(crate) fn normalize_counts_with_table_log_into(
    counts: &[usize],
    total: usize,
    table_log: u8,
    max_log: u8,
    low_prob_count: i32,
    avoid_0_numbit: bool,
    probabilities: &mut [i32],
) -> u8 {
    assert_eq!(counts.len(), probabilities.len());
    if normalize_counts_with_total_into(counts, total, table_log, low_prob_count, probabilities) {
        table_log
    } else {
        old_normalize_counts_into(counts, max_log, avoid_0_numbit, probabilities)
    }
}

/// C-width counterpart used by the native sequence-count transaction.
/// zstd keeps these bounded histograms as `unsigned` through normalization.
#[cfg(test)]
pub(crate) fn normalize_u32_counts_with_table_log(
    counts: &[u32],
    total: u32,
    table_log: u8,
    max_log: u8,
    low_prob_count: i32,
    avoid_0_numbit: bool,
) -> (Vec<i32>, u8) {
    normalize_u32_counts_with_total(counts, total, table_log, low_prob_count)
        .map(|probs| (probs, table_log))
        .unwrap_or_else(|| {
            let widened = counts
                .iter()
                .copied()
                .map(|count| count as usize)
                .collect::<Vec<_>>();
            old_normalize_counts(&widened, max_log, avoid_0_numbit)
        })
}

pub(crate) fn normalize_u32_counts_with_table_log_into(
    counts: &[u32],
    total: u32,
    table_log: u8,
    max_log: u8,
    low_prob_count: i32,
    avoid_0_numbit: bool,
    probabilities: &mut [i32],
) -> u8 {
    assert_eq!(counts.len(), probabilities.len());
    if normalize_u32_counts_with_total_into(counts, total, table_log, low_prob_count, probabilities)
    {
        table_log
    } else {
        let mut widened = [0_usize; 256];
        for (destination, source) in widened.iter_mut().zip(counts.iter().copied()) {
            *destination = source as usize;
        }
        old_normalize_counts_into(
            &widened[..counts.len()],
            max_log,
            avoid_0_numbit,
            probabilities,
        )
    }
}

/// Build the FSE table used for Huffman weight descriptions.
///
/// zstd's `HUF_compressWeights()` normalizes low-probability symbols as `1`
/// instead of `-1`, which can produce a larger NCount header but a shorter
/// compressed weight stream.
pub(crate) fn build_huffman_weight_table_from_data(data: &[u8], max_log: u8) -> FSETable {
    let mut counts = [0; 256];
    let mut max_symbol = 0;
    for symbol in data {
        let symbol = usize::from(*symbol);
        counts[symbol] += 1;
        max_symbol = max_symbol.max(symbol);
    }

    let counts = &counts[..=max_symbol];
    let total = data.len();
    let acc_log = optimal_table_log(max_log, total, max_symbol);
    let (probs, acc_log) = normalize_counts_with_total(counts, total, acc_log, 1)
        .map(|probs| (probs, acc_log))
        .unwrap_or_else(|| old_normalize_counts(counts, max_log, false));
    build_table_from_probabilities(&probs, acc_log)
}

pub(crate) fn build_huffman_weight_table_from_data_with_scratch(
    data: &[u8],
    max_log: u8,
    scratch: &mut FSETableBuildScratch,
) -> FSETable {
    let mut counts = [0; 256];
    let mut max_symbol = 0;
    for symbol in data {
        let symbol = usize::from(*symbol);
        counts[symbol] += 1;
        max_symbol = max_symbol.max(symbol);
    }

    let counts = &counts[..=max_symbol];
    let total = data.len();
    let acc_log = optimal_table_log(max_log, total, max_symbol);
    let mut probabilities = [0_i32; 256];
    let probabilities = &mut probabilities[..counts.len()];
    let acc_log = if normalize_counts_with_total_into(counts, total, acc_log, 1, probabilities) {
        acc_log
    } else {
        old_normalize_counts_into(counts, max_log, false, probabilities)
    };
    build_table_from_probabilities_with_scratch(probabilities, acc_log, scratch)
}

pub(crate) fn optimal_table_log(max_log: u8, total: usize, max_symbol: usize) -> u8 {
    const MIN_TABLE_LOG: u8 = 5;
    // zstd's default FSE_MAX_MEMORY_USAGE is 14, so FSE_MAX_TABLELOG is 12.
    const MAX_TABLE_LOG: u8 = 12;

    let max_bits_src = (total - 1).ilog2().saturating_sub(2);
    let min_bits_src = total.ilog2() + 1;
    let min_bits_symbols = if max_symbol == 0 {
        0
    } else {
        max_symbol.ilog2() + 2
    };
    let min_bits = min_bits_src.min(min_bits_symbols);
    let table_log = u32::from(max_log).min(max_bits_src).max(min_bits);
    table_log.clamp(u32::from(MIN_TABLE_LOG), u32::from(MAX_TABLE_LOG)) as u8
}

pub(crate) fn optimal_table_log_u32(max_log: u8, total: u32, max_symbol: u32) -> u8 {
    const MIN_TABLE_LOG: u8 = 5;
    const MAX_TABLE_LOG: u8 = 12;

    let max_bits_src = (total - 1).ilog2().saturating_sub(2);
    let min_bits_src = total.ilog2() + 1;
    let min_bits_symbols = if max_symbol == 0 {
        0
    } else {
        max_symbol.ilog2() + 2
    };
    let min_bits = min_bits_src.min(min_bits_symbols);
    let table_log = u32::from(max_log).min(max_bits_src).max(min_bits);
    table_log.clamp(u32::from(MIN_TABLE_LOG), u32::from(MAX_TABLE_LOG)) as u8
}

#[cfg(test)]
pub(super) fn normalize_counts(
    counts: &[usize],
    table_log: u8,
    low_prob_count: i32,
) -> Option<Vec<i32>> {
    let total = counts.iter().sum::<usize>();
    normalize_counts_with_total(counts, total, table_log, low_prob_count)
}

fn normalize_counts_with_total(
    counts: &[usize],
    total: usize,
    table_log: u8,
    low_prob_count: i32,
) -> Option<Vec<i32>> {
    let mut normalized = alloc::vec![0_i32; counts.len()];
    normalize_counts_with_total_into(counts, total, table_log, low_prob_count, &mut normalized)
        .then_some(normalized)
}

fn normalize_counts_with_total_into(
    counts: &[usize],
    total: usize,
    table_log: u8,
    low_prob_count: i32,
    normalized: &mut [i32],
) -> bool {
    debug_assert_eq!(total, counts.iter().sum::<usize>());
    debug_assert_eq!(counts.len(), normalized.len());
    normalized.fill(0);
    let table_size = 1i32 << table_log;
    let low_threshold = total >> table_log;
    let scale = 62 - table_log;
    let step = (1u64 << 62) / total as u64;
    let v_step = 1u64 << (scale - 20);
    let rtb_table = [
        0u64, 473_195, 504_333, 520_860, 550_000, 700_000, 750_000, 830_000,
    ];

    let mut still_to_distribute = table_size;
    let mut largest = 0usize;
    let mut largest_probability = 0i32;

    for (symbol, count) in counts.iter().copied().enumerate() {
        if count == 0 {
            continue;
        }
        if count == total {
            normalized[symbol] = table_size;
            return true;
        }
        if count <= low_threshold {
            normalized[symbol] = low_prob_count;
            still_to_distribute -= 1;
            continue;
        }

        let scaled = count as u64 * step;
        let mut probability = (scaled >> scale) as i32;
        if probability < 8 {
            let rest_to_beat = v_step * rtb_table[probability as usize];
            if scaled - ((probability as u64) << scale) > rest_to_beat {
                probability += 1;
            }
        }
        if probability > largest_probability {
            largest_probability = probability;
            largest = symbol;
        }
        normalized[symbol] = probability;
        still_to_distribute -= probability;
    }

    if -still_to_distribute >= normalized[largest] >> 1 {
        normalize_counts_slow_into(counts, total, table_log, low_prob_count, normalized)
    } else {
        normalized[largest] += still_to_distribute;
        true
    }
}

#[cfg(test)]
fn normalize_u32_counts_with_total(
    counts: &[u32],
    total: u32,
    table_log: u8,
    low_prob_count: i32,
) -> Option<Vec<i32>> {
    let mut normalized = alloc::vec![0_i32; counts.len()];
    normalize_u32_counts_with_total_into(counts, total, table_log, low_prob_count, &mut normalized)
        .then_some(normalized)
}

fn normalize_u32_counts_with_total_into(
    counts: &[u32],
    total: u32,
    table_log: u8,
    low_prob_count: i32,
    normalized: &mut [i32],
) -> bool {
    debug_assert_eq!(total, counts.iter().sum::<u32>());
    debug_assert_eq!(counts.len(), normalized.len());
    normalized.fill(0);
    let table_size = 1i32 << table_log;
    let low_threshold = total >> table_log;
    let scale = 62 - table_log;
    let step = (1u64 << 62) / u64::from(total);
    let v_step = 1u64 << (scale - 20);
    let rtb_table = [
        0u64, 473_195, 504_333, 520_860, 550_000, 700_000, 750_000, 830_000,
    ];

    let mut still_to_distribute = table_size;
    let mut largest = 0usize;
    let mut largest_probability = 0i32;

    for (symbol, count) in counts.iter().copied().enumerate() {
        if count == 0 {
            continue;
        }
        if count == total {
            normalized[symbol] = table_size;
            return true;
        }
        if count <= low_threshold {
            normalized[symbol] = low_prob_count;
            still_to_distribute -= 1;
            continue;
        }

        let scaled = u64::from(count) * step;
        let mut probability = (scaled >> scale) as i32;
        if probability < 8 {
            let rest_to_beat = v_step * rtb_table[probability as usize];
            if scaled - ((probability as u64) << scale) > rest_to_beat {
                probability += 1;
            }
        }
        if probability > largest_probability {
            largest_probability = probability;
            largest = symbol;
        }
        normalized[symbol] = probability;
        still_to_distribute -= probability;
    }

    if -still_to_distribute >= normalized[largest] >> 1 {
        normalize_u32_counts_slow_into(counts, total, table_log, low_prob_count, normalized)
    } else {
        normalized[largest] += still_to_distribute;
        true
    }
}

fn normalize_u32_counts_slow_into(
    counts: &[u32],
    total: u32,
    table_log: u8,
    low_prob_count: i32,
    normalized: &mut [i32],
) -> bool {
    const NOT_YET_ASSIGNED: i32 = -2;

    normalized.fill(0);
    let mut distributed = 0u32;
    let mut remaining_total = total;
    let mut low_one = (remaining_total * 3) >> (table_log + 1);
    let low_threshold = remaining_total >> table_log;

    for (symbol, count) in counts.iter().copied().enumerate() {
        if count == 0 {
            continue;
        }
        if count <= low_threshold {
            normalized[symbol] = low_prob_count;
            distributed += 1;
            remaining_total -= count;
            continue;
        }
        if count <= low_one {
            normalized[symbol] = 1;
            distributed += 1;
            remaining_total -= count;
            continue;
        }
        normalized[symbol] = NOT_YET_ASSIGNED;
    }

    let mut to_distribute = (1u32 << table_log) - distributed;
    if to_distribute == 0 {
        return true;
    }

    if remaining_total / to_distribute > low_one {
        low_one = (remaining_total * 3) / (to_distribute * 2);
        for (symbol, count) in counts.iter().copied().enumerate() {
            if normalized[symbol] == NOT_YET_ASSIGNED && count <= low_one {
                normalized[symbol] = 1;
                distributed += 1;
                remaining_total -= count;
            }
        }
        to_distribute = (1u32 << table_log) - distributed;
    }

    if distributed as usize == counts.len() {
        let max_symbol = counts
            .iter()
            .copied()
            .enumerate()
            .max_by_key(|(_, count)| *count)
            .map(|(symbol, _)| symbol)
            .expect("a non-empty histogram has a maximum");
        normalized[max_symbol] += to_distribute as i32;
        return true;
    }

    if remaining_total == 0 {
        let mut symbol = 0usize;
        while to_distribute > 0 {
            if normalized[symbol] > 0 {
                normalized[symbol] += 1;
                to_distribute -= 1;
            }
            symbol = (symbol + 1) % counts.len();
        }
        return true;
    }

    let v_step_log = 62 - table_log;
    let mid = (1u64 << (v_step_log - 1)) - 1;
    let r_step =
        (((1u64 << v_step_log) * u64::from(to_distribute)) + mid) / u64::from(remaining_total);
    let mut tmp_total = mid;
    for (symbol, count) in counts.iter().copied().enumerate() {
        if normalized[symbol] == NOT_YET_ASSIGNED {
            let end = tmp_total + u64::from(count) * r_step;
            let start = tmp_total >> v_step_log;
            let finish = end >> v_step_log;
            let weight = finish - start;
            if weight < 1 {
                return false;
            }
            normalized[symbol] = weight as i32;
            tmp_total = end;
        }
    }

    true
}

fn normalize_counts_slow_into(
    counts: &[usize],
    total: usize,
    table_log: u8,
    low_prob_count: i32,
    normalized: &mut [i32],
) -> bool {
    const NOT_YET_ASSIGNED: i32 = -2;

    normalized.fill(0);
    let mut distributed = 0usize;
    let mut remaining_total = total;
    let mut low_one = (remaining_total * 3) >> (table_log + 1);
    let low_threshold = remaining_total >> table_log;

    for (symbol, count) in counts.iter().copied().enumerate() {
        if count == 0 {
            continue;
        }
        if count <= low_threshold {
            normalized[symbol] = low_prob_count;
            distributed += 1;
            remaining_total -= count;
            continue;
        }
        if count <= low_one {
            normalized[symbol] = 1;
            distributed += 1;
            remaining_total -= count;
            continue;
        }
        normalized[symbol] = NOT_YET_ASSIGNED;
    }

    let mut to_distribute = (1usize << table_log) - distributed;
    if to_distribute == 0 {
        return true;
    }

    if remaining_total / to_distribute > low_one {
        low_one = (remaining_total * 3) / (to_distribute * 2);
        for (symbol, count) in counts.iter().copied().enumerate() {
            if normalized[symbol] == NOT_YET_ASSIGNED && count <= low_one {
                normalized[symbol] = 1;
                distributed += 1;
                remaining_total -= count;
            }
        }
        to_distribute = (1usize << table_log) - distributed;
    }

    if distributed == counts.len() {
        let max_symbol = counts
            .iter()
            .copied()
            .enumerate()
            .max_by_key(|(_, count)| *count)
            .map(|(symbol, _)| symbol)
            .expect("a non-empty histogram has a maximum");
        normalized[max_symbol] += to_distribute as i32;
        return true;
    }

    if remaining_total == 0 {
        let mut symbol = 0usize;
        while to_distribute > 0 {
            if normalized[symbol] > 0 {
                normalized[symbol] += 1;
                to_distribute -= 1;
            }
            symbol = (symbol + 1) % counts.len();
        }
        return true;
    }

    let v_step_log = 62 - table_log;
    let mid = (1u64 << (v_step_log - 1)) - 1;
    let r_step = (((1u64 << v_step_log) * to_distribute as u64) + mid) / remaining_total as u64;
    let mut tmp_total = mid;
    for (symbol, count) in counts.iter().copied().enumerate() {
        if normalized[symbol] == NOT_YET_ASSIGNED {
            let end = tmp_total + count as u64 * r_step;
            let start = tmp_total >> v_step_log;
            let finish = end >> v_step_log;
            let weight = finish - start;
            if weight < 1 {
                return false;
            }
            normalized[symbol] = weight as i32;
            tmp_total = end;
        }
    }

    true
}

fn old_normalize_counts(counts: &[usize], max_log: u8, avoid_0_numbit: bool) -> (Vec<i32>, u8) {
    let mut probs = alloc::vec![0i32; counts.len()];
    let acc_log = old_normalize_counts_into(counts, max_log, avoid_0_numbit, &mut probs);
    (probs, acc_log)
}

fn old_normalize_counts_into(
    counts: &[usize],
    max_log: u8,
    avoid_0_numbit: bool,
    probs: &mut [i32],
) -> u8 {
    debug_assert_eq!(counts.len(), probs.len());
    probs.fill(0);
    let mut min_count = 0;
    for (idx, count) in counts.iter().copied().enumerate() {
        probs[idx] = count as i32;
        if count > 0 && (count < min_count || min_count == 0) {
            min_count = count;
        }
    }

    min_count -= 1;
    let mut max_prob = 0i32;
    for prob in probs.iter_mut() {
        if *prob > 0 {
            *prob -= min_count as i32;
        }
        max_prob = max_prob.max(*prob);
    }

    if max_prob > 0 && max_prob as usize > probs.len() {
        let divisor = max_prob / (probs.len() as i32);
        for prob in probs.iter_mut() {
            if *prob > 0 {
                *prob = (*prob / divisor).max(1)
            }
        }
    }

    let sum = probs.iter().sum::<i32>();
    assert!(sum > 0);
    let sum = sum as usize;
    let acc_log = (sum.ilog2() as u8 + 1).max(5);
    let acc_log = u8::min(acc_log, max_log);

    if sum < 1 << acc_log {
        let diff = (1 << acc_log) - sum;
        let max = probs.iter_mut().max().unwrap();
        *max += diff as i32;
    } else {
        let mut diff = sum - (1 << acc_log);
        while diff > 0 {
            let min = probs.iter_mut().filter(|prob| **prob > 1).min().unwrap();
            let decrease = usize::min(*min as usize - 1, diff);
            diff -= decrease;
            *min -= decrease as i32;
        }
    }
    let max = probs.iter_mut().max().unwrap();
    if avoid_0_numbit && *max > 1 << (acc_log - 1) {
        let redistribute = *max - (1 << (acc_log - 1));
        *max -= redistribute;
        let max = *max;

        let second_max = *probs.iter_mut().filter(|x| **x != max).max().unwrap();
        let second_max = probs.iter_mut().find(|x| **x == second_max).unwrap();
        *second_max += redistribute;
        assert!(*second_max <= max);
    }

    acc_log
}
