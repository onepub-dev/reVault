use alloc::vec::Vec;
use core::convert::TryFrom;

use super::{
    lengths::{
        high_bit, highest_bit_set, invalid_huffman_tree, limit_code_lengths, LengthLimitedSymbol,
    },
    HUFFMAN_NODE_NONE, HUFFMAN_RANK_NONE, MAX_HUFFMAN_BITS,
};

pub(super) fn is_flat_distribution(counts: &[usize]) -> bool {
    let mut nonzero = 0usize;
    let mut min = usize::MAX;
    let mut max = 0usize;
    for count in counts.iter().copied().filter(|count| *count > 0) {
        nonzero += 1;
        min = min.min(count);
        max = max.max(count);
    }

    nonzero > 128 && max <= min.saturating_mul(2)
}

pub(super) fn length_limited_code_lengths(counts: &[usize], max_bits: usize) -> Option<Vec<usize>> {
    let mut nodes = Vec::new();
    length_limited_code_lengths_with_nodes(counts, max_bits, &mut nodes)
}

pub(super) fn length_limited_code_lengths_with_nodes(
    counts: &[usize],
    max_bits: usize,
    nodes: &mut Vec<HuffmanNode>,
) -> Option<Vec<usize>> {
    let base = base_code_lengths_with_nodes(counts, nodes)?;
    length_limited_code_lengths_from_base(base.lengths, base.symbols, max_bits)
}

pub(super) fn length_limited_code_lengths_from_base(
    mut lengths: Vec<usize>,
    mut symbols: Vec<LengthLimitedSymbol>,
    max_bits: usize,
) -> Option<Vec<usize>> {
    limit_code_lengths(&mut lengths, &mut symbols, max_bits).then_some(lengths)
}

pub(super) fn base_code_lengths(counts: &[usize]) -> Option<BaseCodeLengths> {
    let mut nodes = Vec::new();
    base_code_lengths_with_nodes(counts, &mut nodes)
}

fn base_code_lengths_with_nodes(
    counts: &[usize],
    nodes: &mut Vec<HuffmanNode>,
) -> Option<BaseCodeLengths> {
    let leaf_count = build_huffman_tree(counts, nodes)?;

    let mut lengths = alloc::vec![0; counts.len()];
    let mut symbols = Vec::with_capacity(leaf_count);
    let mut largest_bits = 0usize;
    for idx in 1..=leaf_count {
        let parent = huffman_parent(&nodes[idx]);
        let len = nodes[parent].len + 1;
        let symbol = huffman_symbol(&nodes[idx]);
        nodes[idx].len = len;
        let len = usize::from(len);
        lengths[symbol] = len;
        largest_bits = largest_bits.max(len);
        symbols.push(LengthLimitedSymbol {
            symbol,
            count: usize::try_from(nodes[idx].count).expect("u32 count fits in usize"),
            len,
        });
    }

    Some(BaseCodeLengths {
        lengths,
        symbols,
        largest_bits,
    })
}

#[cfg_attr(target_vendor = "apple", link_section = "__TEXT,__rz_hut")]
#[cfg_attr(target_family = "windows", link_section = ".text$040.rz.hut")]
#[cfg_attr(
    all(
        not(target_vendor = "apple"),
        not(target_family = "windows"),
        not(target_family = "wasm")
    ),
    link_section = ".text.sorted.040.ruzstd.huffman.tree"
)]
pub(super) fn build_huffman_tree(counts: &[usize], nodes: &mut Vec<HuffmanNode>) -> Option<usize> {
    c_sorted_huffman_nodes_into(counts, nodes);

    let leaf_count = nodes.len().saturating_sub(1);
    if leaf_count <= 1 {
        return None;
    }

    let first_parent = leaf_count + 1;
    let root = 2 * leaf_count - 1;
    nodes.resize(
        2 * leaf_count,
        HuffmanNode {
            count: u32::MAX,
            symbol: 0,
            parent: HUFFMAN_NODE_NONE,
            len: 0,
        },
    );
    let mut smallest_leaf = leaf_count;
    let mut smallest_parent = first_parent;

    for parent in first_parent..=root {
        let left = pop_smallest_huffman_node(nodes, &mut smallest_leaf, &mut smallest_parent);
        let right = pop_smallest_huffman_node(nodes, &mut smallest_leaf, &mut smallest_parent);
        let parent_index = u16::try_from(parent).expect("Huffman parent index fits in u16");
        nodes[left].parent = parent_index;
        nodes[right].parent = parent_index;
        nodes[parent].count = nodes[left]
            .count
            .checked_add(nodes[right].count)
            .unwrap_or_else(|| invalid_huffman_tree());
    }

    nodes[root].len = 0;
    for idx in (first_parent..root).rev() {
        let parent = huffman_parent(&nodes[idx]);
        nodes[idx].len = nodes[parent].len + 1;
    }
    for idx in 1..=leaf_count {
        let parent = huffman_parent(&nodes[idx]);
        nodes[idx].len = nodes[parent].len + 1;
    }

    Some(leaf_count)
}

pub(super) fn limit_huffman_tree_height(
    nodes: &mut [HuffmanNode],
    leaf_count: usize,
    target_bits: usize,
) -> Option<usize> {
    let largest_bits = usize::from(nodes[leaf_count].len);
    if largest_bits <= target_bits {
        return Some(largest_bits);
    }

    let shift = largest_bits.checked_sub(target_bits)?;
    let base_cost = 1isize.checked_shl(u32::try_from(shift).ok()?)?;
    let mut total_cost = 0isize;
    let mut last_below_target = leaf_count;
    while usize::from(nodes[last_below_target].len) > target_bits {
        let len = usize::from(nodes[last_below_target].len);
        let rank_cost = 1isize.checked_shl(u32::try_from(largest_bits - len).ok()?)?;
        total_cost += base_cost - rank_cost;
        nodes[last_below_target].len = u8::try_from(target_bits).ok()?;
        last_below_target -= 1;
    }
    while usize::from(nodes[last_below_target].len) == target_bits {
        last_below_target -= 1;
    }

    total_cost >>= shift;
    if total_cost <= 0 {
        return None;
    }

    let mut rank_last = [HUFFMAN_RANK_NONE; MAX_HUFFMAN_BITS + 2];
    let mut current_bits = target_bits;
    for pos in (1..=last_below_target).rev() {
        let len = usize::from(nodes[pos].len);
        if len >= current_bits {
            continue;
        }
        current_bits = len;
        rank_last[target_bits - current_bits] = pos;
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
            if nodes[high_pos].count <= nodes[low_pos].count.saturating_mul(2) {
                break;
            }
            bits_to_decrease -= 1;
        }

        while bits_to_decrease <= target_bits && rank_last[bits_to_decrease] == HUFFMAN_RANK_NONE {
            bits_to_decrease += 1;
        }
        if bits_to_decrease > target_bits {
            return None;
        }

        total_cost -= 1isize << (bits_to_decrease - 1);
        let pos = rank_last[bits_to_decrease];
        if pos == HUFFMAN_RANK_NONE {
            return None;
        }
        nodes[pos].len += 1;

        if rank_last[bits_to_decrease - 1] == HUFFMAN_RANK_NONE {
            rank_last[bits_to_decrease - 1] = pos;
        }
        if pos == 1 {
            rank_last[bits_to_decrease] = HUFFMAN_RANK_NONE;
        } else {
            let previous = pos - 1;
            rank_last[bits_to_decrease] =
                if usize::from(nodes[previous].len) == target_bits - bits_to_decrease {
                    previous
                } else {
                    HUFFMAN_RANK_NONE
                };
        }
    }

    while total_cost < 0 {
        if rank_last[1] == HUFFMAN_RANK_NONE {
            while usize::from(nodes[last_below_target].len) == target_bits {
                last_below_target -= 1;
            }
            let pos = last_below_target + 1;
            nodes[pos].len = nodes[pos].len.checked_sub(1)?;
            rank_last[1] = pos;
            total_cost += 1;
            continue;
        }

        let pos = rank_last[1] + 1;
        if pos > leaf_count {
            return None;
        }
        nodes[pos].len = nodes[pos].len.checked_sub(1)?;
        rank_last[1] = pos;
        total_cost += 1;
    }

    Some(target_bits)
}

pub(super) struct BaseCodeLengths {
    pub(super) lengths: Vec<usize>,
    pub(super) symbols: Vec<LengthLimitedSymbol>,
    pub(super) largest_bits: usize,
}

#[cfg(test)]
pub(super) fn c_sorted_huffman_nodes(counts: &[usize]) -> Vec<HuffmanNode> {
    let mut nodes = Vec::new();
    c_sorted_huffman_nodes_into(counts, &mut nodes);
    nodes
}

fn c_sorted_huffman_nodes_into(counts: &[usize], nodes: &mut Vec<HuffmanNode>) {
    const RANK_POSITION_TABLE_SIZE: usize = 192;
    const RANK_POSITION_MAX_COUNT_LOG: usize = 32;
    const RANK_POSITION_LOG_BUCKETS_BEGIN: usize =
        (RANK_POSITION_TABLE_SIZE - 1) - RANK_POSITION_MAX_COUNT_LOG - 1;
    const RANK_POSITION_DISTINCT_COUNT_CUTOFF: usize = RANK_POSITION_LOG_BUCKETS_BEGIN + 7;

    #[derive(Clone, Copy, Default)]
    struct RankPosition {
        base: usize,
        curr: usize,
    }

    fn rank_index(count: usize) -> usize {
        if count < RANK_POSITION_DISTINCT_COUNT_CUTOFF {
            count
        } else {
            high_bit(count) as usize + RANK_POSITION_LOG_BUCKETS_BEGIN
        }
    }

    let mut rank_position = [RankPosition::default(); RANK_POSITION_TABLE_SIZE];
    let mut nonzero_count = 0usize;
    for count in counts.iter().copied() {
        nonzero_count += usize::from(count != 0);
        let lower_rank = rank_index(count);
        debug_assert!(lower_rank < RANK_POSITION_TABLE_SIZE - 1);
        rank_position[lower_rank].base += 1;
    }

    for rank in (1..RANK_POSITION_TABLE_SIZE).rev() {
        rank_position[rank - 1].base += rank_position[rank].base;
        rank_position[rank - 1].curr = rank_position[rank - 1].base;
    }

    nodes.clear();
    nodes.resize(
        nonzero_count + 1,
        HuffmanNode {
            count: u32::MAX,
            symbol: 0,
            parent: HUFFMAN_NODE_NONE,
            len: 0,
        },
    );
    for (symbol, count) in counts.iter().copied().enumerate() {
        if count == 0 {
            continue;
        }
        let rank = rank_index(count) + 1;
        let pos = rank_position[rank].curr;
        rank_position[rank].curr += 1;
        debug_assert!(pos < nonzero_count);
        nodes[pos + 1] = HuffmanNode {
            count: u32::try_from(count).unwrap_or_else(|_| invalid_huffman_tree()),
            symbol: u8::try_from(symbol).expect("Huffman symbol fits in u8"),
            parent: HUFFMAN_NODE_NONE,
            len: 0,
        };
    }

    for rank in &rank_position[RANK_POSITION_DISTINCT_COUNT_CUTOFF..RANK_POSITION_TABLE_SIZE - 1] {
        let bucket_start = rank.base.min(nonzero_count) + 1;
        let bucket_end = rank.curr.min(nonzero_count) + 1;
        if bucket_end.saturating_sub(bucket_start) > 1 {
            c_huf_simple_quick_sort(&mut nodes[bucket_start..bucket_end]);
        }
    }
}

fn c_huf_simple_quick_sort(nodes: &mut [HuffmanNode]) {
    const INSERTION_SORT_THRESHOLD: usize = 8;
    if nodes.len() <= 1 {
        return;
    }
    if nodes.len() - 1 < INSERTION_SORT_THRESHOLD {
        c_huf_insertion_sort(nodes);
        return;
    }

    let mut low = 0usize;
    let mut high = nodes.len() - 1;
    while low < high {
        let idx = c_huf_quick_sort_partition(nodes, low, high);
        if idx - low < high - idx {
            if low < idx {
                c_huf_simple_quick_sort(&mut nodes[low..idx]);
            }
            low = idx + 1;
        } else {
            if idx < high {
                c_huf_simple_quick_sort(&mut nodes[idx + 1..=high]);
            }
            if idx == 0 {
                break;
            }
            high = idx - 1;
        }
    }
}

fn c_huf_insertion_sort(nodes: &mut [HuffmanNode]) {
    for idx in 1..nodes.len() {
        let key = nodes[idx];
        let mut pos = idx;
        while pos > 0 && nodes[pos - 1].count < key.count {
            nodes[pos] = nodes[pos - 1];
            pos -= 1;
        }
        nodes[pos] = key;
    }
}

fn c_huf_quick_sort_partition(nodes: &mut [HuffmanNode], low: usize, high: usize) -> usize {
    let pivot = nodes[high].count;
    let mut boundary = low;
    for idx in low..high {
        if nodes[idx].count > pivot {
            nodes.swap(boundary, idx);
            boundary += 1;
        }
    }
    nodes.swap(boundary, high);
    boundary
}

fn pop_smallest_huffman_node(
    nodes: &[HuffmanNode],
    smallest_leaf: &mut usize,
    smallest_parent: &mut usize,
) -> usize {
    if nodes[*smallest_leaf].count < nodes[*smallest_parent].count {
        let idx = *smallest_leaf;
        *smallest_leaf -= 1;
        idx
    } else {
        let idx = *smallest_parent;
        *smallest_parent += 1;
        idx
    }
}

/// Safe equivalent of C's 8-byte `nodeElt` Huffman workspace entry.
/// `symbol` is read only from real leaves; barrier and parent entries use zero.
#[derive(Clone, Copy)]
pub(super) struct HuffmanNode {
    pub(super) count: u32,
    parent: u16,
    pub(super) symbol: u8,
    pub(super) len: u8,
}

fn huffman_parent(node: &HuffmanNode) -> usize {
    if node.parent == HUFFMAN_NODE_NONE {
        invalid_huffman_tree()
    }
    usize::from(node.parent)
}

fn huffman_symbol(node: &HuffmanNode) -> usize {
    usize::from(node.symbol)
}
