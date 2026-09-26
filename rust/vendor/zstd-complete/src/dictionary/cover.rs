//! An implementation of the local maximum coverage algorithm
//! described in the paper "Effective Construction of Relative Lempel-Ziv Dictionaries",
//! by Liao, Petri, Moffat, and Wirth, published under the University of Melbourne.
//!
//! See: <https://people.eng.unimelb.edu.au/ammoffat/abstracts/lpmw16www.pdf>
//!
//! Facebook's implementation was also used as a reference.
//! <https://github.com/facebook/zstd/tree/dev/lib/dictBuilder>

use super::DictParams;
use crate::dictionary::frequency::estimate_frequency;
use core::convert::TryInto;
use std::collections::HashMap;
use std::vec::Vec;

/// The size of each k-mer
pub(super) const K: usize = 16;

///As found under "4: Experiments - Varying k-mer Size" in the original paper,
/// "when k = 16, across all our text collections, there is a reasonable spread"
///
/// Reasonable range: [6, 16]
pub(super) type KMer = [u8; K];

pub struct Segment {
    /// The actual contents of the segment.
    pub raw: Vec<u8>,
    /// A measure of how "ideal" a given segment would be to include in the dictionary
    ///
    /// Higher is better, there's no upper limit. This number is determined by
    /// estimating the number of occurances in a given epoch
    pub score: usize,
}

impl Eq for Segment {}

impl PartialEq for Segment {
    fn eq(&self, other: &Self) -> bool {
        // We only really care about score in regards to heap order
        self.score == other.score
    }
}

impl PartialOrd for Segment {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Segment {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.score.cmp(&other.score)
    }
}

/// A re-usable allocation containing large allocations
/// that are used multiple times during dictionary construction (once per epoch)
pub struct Context {
    /// Keeps track of the number of occurances of a particular k-mer within an epoch.
    ///
    /// Reset for each epoch.
    pub frequencies: HashMap<KMer, usize>,
}

/// Returns the highest scoring segment from `epoch`.
///
/// Candidate segments come from the epoch being processed. Their k-mers are
/// scored against `collection_sample`, which is the reservoir sample of the
/// full training corpus.
pub fn pick_best_segment(
    params: &DictParams,
    ctx: &mut Context,
    epoch: &'_ [u8],
    collection_sample: &'_ [u8],
) -> Option<Segment> {
    let mut segments = epoch.chunks(params.segment_size as usize).peekable();
    let mut best_segment: &[u8] = segments.peek()?;
    let mut top_segment_score: usize = 0;
    // Iterate over segments and score each segment, keeping track of the best segment
    for segment in segments {
        let segment_score = score_segment(ctx, collection_sample, segment);
        if segment_score > top_segment_score {
            best_segment = segment;
            top_segment_score = segment_score;
        }
    }

    Some(Segment {
        raw: best_segment.into(),
        score: top_segment_score,
    })
}

/// Given a segment, compute the score (or usefulness) of that segment against the entire epoch.
///
/// `score_segment` modifies `ctx.frequencies`.
fn score_segment(ctx: &mut Context, collection_sample: &[u8], segment: &[u8]) -> usize {
    if segment.len() < K {
        return 0;
    }

    let mut segment_score = 0;
    // Determine the score of each overlapping k-mer
    for window in segment.windows(K) {
        let kmer: &KMer = window.try_into().expect("Failed to make kmer");
        // if the kmer is already in the pool, it recieves a score of zero
        if ctx.frequencies.contains_key(kmer) {
            continue;
        }
        let kmer_score = estimate_frequency(kmer, collection_sample);
        ctx.frequencies.insert(*kmer, kmer_score);
        segment_score += kmer_score;
    }

    segment_score
}

/// Computes the number of epochs and the size of each epoch.
///
/// Returns a (number of epochs, epoch size) tuple.
///
/// A translation of `COVER_epoch_info_t COVER_computeEpochs()` from facebook/zstd.
pub fn compute_epoch_info(
    params: &DictParams,
    max_dict_size: usize,
    num_kmers: usize,
) -> (usize, usize) {
    let min_epoch_size = 10_000; // 10 KiB
    let mut num_epochs: usize = usize::max(1, max_dict_size / params.segment_size as usize);
    let mut epoch_size: usize = num_kmers / num_epochs;
    if epoch_size >= min_epoch_size {
        assert!(epoch_size * num_epochs <= num_kmers);
        return (num_epochs, epoch_size);
    }
    epoch_size = usize::min(min_epoch_size, num_kmers);
    num_epochs = num_kmers / epoch_size;
    (num_epochs, epoch_size)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn best_segment_is_selected_from_epoch() {
        let params = DictParams { segment_size: 32 };
        let mut ctx = Context {
            frequencies: HashMap::new(),
        };
        let first_epoch_segment = [b'a'; 32];
        let second_epoch_segment = [b'b'; 32];
        let epoch = [first_epoch_segment, second_epoch_segment].concat();
        let collection_sample = [b'b'; 128];

        let best_segment =
            pick_best_segment(&params, &mut ctx, &epoch, &collection_sample).unwrap();

        assert_eq!(best_segment.raw, second_epoch_segment);
        assert!(best_segment.score > 0);
    }

    #[test]
    fn short_segments_do_not_underflow() {
        let params = DictParams { segment_size: 8 };
        let mut ctx = Context {
            frequencies: HashMap::new(),
        };
        let epoch = [b'a'; 8];
        let collection_sample = [b'a'; 32];

        let best_segment =
            pick_best_segment(&params, &mut ctx, &epoch, &collection_sample).unwrap();

        assert_eq!(best_segment.raw, epoch);
        assert_eq!(best_segment.score, 0);
    }
}
