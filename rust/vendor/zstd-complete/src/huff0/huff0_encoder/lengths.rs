use alloc::vec::Vec;

use super::{HUFFMAN_RANK_NONE, MAX_HUFFMAN_BITS};

pub(super) fn limit_code_lengths(
    lengths: &mut [usize],
    symbols: &mut [LengthLimitedSymbol],
    max_bits: usize,
) -> bool {
    if symbols.len() <= 1 {
        return false;
    }

    let largest_bits = symbols.iter().map(|symbol| symbol.len).max().unwrap_or(0);
    if largest_bits <= max_bits {
        return length_units(lengths, max_bits) == 1usize << max_bits;
    }

    let Some(shift) = largest_bits.checked_sub(max_bits) else {
        return false;
    };
    if shift >= usize::BITS as usize {
        return false;
    }

    let base_cost = 1usize << shift;
    let mut total_cost = 0isize;
    let mut last_below_target = symbols.len() as isize - 1;

    while last_below_target >= 0 && symbols[last_below_target as usize].len > max_bits {
        let len = symbols[last_below_target as usize].len;
        let Some(rank_cost) = 1usize.checked_shl((largest_bits - len) as u32) else {
            return false;
        };
        total_cost += base_cost.saturating_sub(rank_cost) as isize;
        symbols[last_below_target as usize].len = max_bits;
        last_below_target -= 1;
    }

    while last_below_target >= 0 && symbols[last_below_target as usize].len == max_bits {
        last_below_target -= 1;
    }

    total_cost >>= shift;
    if total_cost <= 0 {
        return false;
    }

    let mut rank_last = [HUFFMAN_RANK_NONE; MAX_HUFFMAN_BITS + 2];
    let mut current_bits = max_bits;
    for pos in (0..=last_below_target).rev() {
        let pos = pos as usize;
        let len = symbols[pos].len;
        if len >= current_bits {
            continue;
        }
        current_bits = len;
        rank_last[max_bits - current_bits] = pos;
    }

    while total_cost > 0 {
        let mut bits_to_decrease = highest_bit_set(total_cost as usize);
        for candidate_bits in (2..=bits_to_decrease).rev() {
            let high_pos = rank_last[candidate_bits];
            let low_pos = rank_last[candidate_bits - 1];
            if high_pos == HUFFMAN_RANK_NONE {
                bits_to_decrease -= 1;
                continue;
            }
            if low_pos == HUFFMAN_RANK_NONE {
                break;
            }
            if symbols[high_pos].count <= symbols[low_pos].count.saturating_mul(2) {
                break;
            }
            bits_to_decrease -= 1;
        }

        while bits_to_decrease <= max_bits && rank_last[bits_to_decrease] == HUFFMAN_RANK_NONE {
            bits_to_decrease += 1;
        }
        if bits_to_decrease > max_bits {
            return false;
        }

        total_cost -= 1isize << (bits_to_decrease - 1);
        let pos = rank_last[bits_to_decrease];
        if pos == HUFFMAN_RANK_NONE {
            return false;
        }
        symbols[pos].len += 1;

        if rank_last[bits_to_decrease - 1] == HUFFMAN_RANK_NONE {
            rank_last[bits_to_decrease - 1] = pos;
        }

        if pos == 0 {
            rank_last[bits_to_decrease] = HUFFMAN_RANK_NONE;
        } else {
            let previous = pos - 1;
            if symbols[previous].len == max_bits - bits_to_decrease {
                rank_last[bits_to_decrease] = previous;
            } else {
                rank_last[bits_to_decrease] = HUFFMAN_RANK_NONE;
            }
        }
    }

    while total_cost < 0 {
        if rank_last[1] == HUFFMAN_RANK_NONE {
            while last_below_target >= 0 && symbols[last_below_target as usize].len == max_bits {
                last_below_target -= 1;
            }
            let pos = (last_below_target + 1) as usize;
            if pos >= symbols.len() || symbols[pos].len == 0 {
                return false;
            }
            symbols[pos].len -= 1;
            rank_last[1] = pos;
            total_cost += 1;
            continue;
        }

        let pos = rank_last[1] + 1;
        if pos >= symbols.len() || symbols[pos].len == 0 {
            return false;
        }
        symbols[pos].len -= 1;
        rank_last[1] = pos;
        total_cost += 1;
    }

    lengths.fill(0);
    for symbol in symbols {
        lengths[symbol.symbol] = symbol.len;
    }

    length_units(lengths, max_bits) == 1usize << max_bits
}

#[derive(Clone)]
pub(super) struct LengthLimitedSymbol {
    pub(super) symbol: usize,
    pub(super) count: usize,
    pub(super) len: usize,
}

pub(super) fn length_units(lengths: &[usize], max_bits: usize) -> usize {
    let mut units = 0usize;
    for len in lengths.iter().copied().filter(|len| *len > 0) {
        if len > max_bits {
            return usize::MAX;
        }
        let Some(unit) = 1usize.checked_shl((max_bits - len) as u32) else {
            return usize::MAX;
        };
        units = units.saturating_add(unit);
    }
    units
}

pub(super) fn rank_limited_weights(counts: &[usize]) -> Vec<usize> {
    let zeros = counts.iter().filter(|x| **x == 0).count();
    let weights = rank_limited_nonzero_weights(counts.len() - zeros);
    let mut next_weight = 0;
    let mut counts_sorted = counts
        .iter()
        .enumerate()
        .filter(|(_, count)| **count > 0)
        .collect::<Vec<_>>();
    counts_sorted.sort_unstable_by(|(left_idx, left_count), (right_idx, right_count)| {
        left_count
            .cmp(right_count)
            .then_with(|| left_idx.cmp(right_idx))
    });

    let mut weights_distributed = alloc::vec![0; counts.len()];
    for (idx, count) in counts_sorted {
        debug_assert!(*count > 0);
        weights_distributed[idx] = next_rank_limited_weight(&weights, &mut next_weight);
    }

    weights_distributed
}

/// Writes the rank-limited fallback into caller-provided storage.
///
/// Huffman alphabets contain at most 256 symbols, so prepared workspaces can
/// keep this uncommon fallback allocation-free without retaining another
/// heap vector in every encoder context.
pub(super) fn rank_limited_weights_into<'a>(
    counts: &[usize],
    output: &'a mut [usize; 256],
) -> &'a [usize] {
    #[derive(Clone, Copy, Default)]
    struct RankedSymbol {
        symbol: usize,
        count: usize,
    }

    debug_assert!(counts.len() <= output.len());
    let mut ranked = [RankedSymbol::default(); 256];
    let mut ranked_len = 0;
    for (symbol, &count) in counts.iter().enumerate() {
        if count != 0 {
            ranked[ranked_len] = RankedSymbol { symbol, count };
            ranked_len += 1;
        }
    }
    ranked[..ranked_len].sort_unstable_by(|left, right| {
        left.count
            .cmp(&right.count)
            .then_with(|| left.symbol.cmp(&right.symbol))
    });

    let mut weights = [0u8; 256];
    let weight_sum_log = distribute_weights_with_sum_log_into(ranked_len, &mut weights);
    let limit = ranked_len.ilog2() as usize + 2;
    redistribute_weights_with_sum_log(&mut weights[..ranked_len], limit, weight_sum_log);

    output[..counts.len()].fill(0);
    for (rank, symbol) in ranked[..ranked_len].iter().enumerate() {
        output[symbol.symbol] = usize::from(weights[rank]);
    }
    &output[..counts.len()]
}

pub(super) fn rank_limited_weights_from_sorted_symbols(
    count_len: usize,
    symbols_by_desc_count: &[LengthLimitedSymbol],
) -> Vec<usize> {
    let weights = rank_limited_nonzero_weights(symbols_by_desc_count.len());
    let mut next_weight = 0;
    let mut weights_distributed = alloc::vec![0; count_len];
    let mut group_end = symbols_by_desc_count.len();
    while group_end > 0 {
        let count = symbols_by_desc_count[group_end - 1].count;
        let mut group_start = group_end - 1;
        while group_start > 0 && symbols_by_desc_count[group_start - 1].count == count {
            group_start -= 1;
        }

        for symbol in &symbols_by_desc_count[group_start..group_end] {
            weights_distributed[symbol.symbol] =
                next_rank_limited_weight(&weights, &mut next_weight);
        }
        group_end = group_start;
    }

    weights_distributed
}

fn rank_limited_nonzero_weights(amount: usize) -> Vec<u8> {
    let (mut weights, weight_sum_log) = distribute_weights_with_sum_log(amount);
    let limit = weights.len().ilog2() as usize + 2;
    redistribute_weights_with_sum_log(&mut weights, limit, weight_sum_log);
    weights
}

fn next_rank_limited_weight(weights: &[u8], next_weight: &mut usize) -> usize {
    let Some(weight) = weights.get(*next_weight).copied() else {
        invalid_huffman_tree();
    };
    *next_weight += 1;
    usize::from(weight)
}

#[cold]
#[inline(never)]
pub(super) fn invalid_huffman_tree() -> ! {
    panic!("huffman tree construction invariant failed")
}

pub(super) fn high_bit(x: usize) -> u32 {
    if x == 0 {
        0
    } else {
        x.ilog2()
    }
}

/// Assert that the provided value is greater than zero, and returns index of the first set bit
pub(super) fn highest_bit_set(x: usize) -> usize {
    assert!(x > 0);
    usize::BITS as usize - x.leading_zeros() as usize
}

/// Distributes weights that add up to a clean power of two.
#[cfg(test)]
pub(super) fn distribute_weights(amount: usize) -> Vec<u8> {
    distribute_weights_with_sum_log(amount).0
}

fn distribute_weights_with_sum_log(amount: usize) -> (Vec<u8>, usize) {
    assert!((2..=256).contains(&amount));
    let mut weights = alloc::vec![1u8, 1];
    let mut target_weight = 1usize;
    let mut weight_counter = 2usize;
    while weights.len() < amount {
        let mut add_new = 1usize << (weight_counter - target_weight);
        let available_space = amount - weights.len();
        if add_new > available_space {
            target_weight = weight_counter;
            add_new = 1;
        }
        weights.extend(core::iter::repeat_n(target_weight as u8, add_new));
        weight_counter += 1;
    }
    let weight_sum_log = weights
        .iter()
        .copied()
        .map(|weight| 1usize << weight)
        .sum::<usize>()
        .ilog2() as usize;
    (weights, weight_sum_log)
}

fn distribute_weights_with_sum_log_into(amount: usize, weights: &mut [u8; 256]) -> usize {
    assert!((2..=256).contains(&amount));
    weights[0] = 1;
    weights[1] = 1;
    let mut len = 2;
    let mut target_weight = 1usize;
    let mut weight_counter = 2usize;
    while len < amount {
        let mut add_new = 1usize << (weight_counter - target_weight);
        let available_space = amount - len;
        if add_new > available_space {
            target_weight = weight_counter;
            add_new = 1;
        }
        weights[len..len + add_new].fill(target_weight as u8);
        len += add_new;
        weight_counter += 1;
    }
    weights[..amount]
        .iter()
        .copied()
        .map(|weight| 1usize << weight)
        .sum::<usize>()
        .ilog2() as usize
}

#[cfg(test)]
pub(super) fn redistribute_weights(weights: &mut [u8], max_num_bits: usize) {
    let weight_sum_log = weights
        .iter()
        .copied()
        .map(|weight| 1usize << weight)
        .sum::<usize>()
        .ilog2() as usize;
    redistribute_weights_with_sum_log(weights, max_num_bits, weight_sum_log);
}

fn redistribute_weights_with_sum_log(
    weights: &mut [u8],
    max_num_bits: usize,
    weight_sum_log: usize,
) {
    if weight_sum_log < max_num_bits {
        return;
    }
    let decrease_weights_by = weight_sum_log - max_num_bits + 1;
    let mut added_weights = 0usize;
    for weight in weights.iter_mut() {
        let weight_value = usize::from(*weight);
        if weight_value < decrease_weights_by {
            for add in weight_value..decrease_weights_by {
                added_weights += 1 << add;
            }
            *weight = decrease_weights_by as u8;
        }
    }
    while added_weights > 0 {
        let max_weight = added_weights.ilog2() as u8 + 1;
        let upper = weights.partition_point(|weight| *weight <= max_weight);
        if upper == 0 {
            invalid_huffman_tree();
        }
        let current_weight = weights[upper - 1];
        let current_idx = weights.partition_point(|weight| *weight < current_weight);
        let current_unit = 1usize << (usize::from(current_weight) - 1);
        added_weights -= current_unit;
        weights[current_idx] -= 1;
    }
    if weights[0] > 1 {
        let offset = weights[0] - 1;
        for weight in weights.iter_mut() {
            *weight -= offset;
        }
    }
}
