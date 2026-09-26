//! Canonical Huffman table construction kernel.

use alloc::{vec, vec::Vec};
use core::convert::TryFrom;

use super::HuffmanCode;
use crate::workspace::{Arena, ArenaError, ArenaSize, ReusableVec};

const MAX_HUFFMAN_BITS: usize = 11;
const NODE_NONE: u16 = u16::MAX;
const RANK_NONE: usize = usize::MAX;

#[derive(Clone, Default)]
pub struct HuffmanBuildScratch {
    nodes: ReusableVec<Node>,
}

impl HuffmanBuildScratch {
    pub(crate) fn add_workspace_size(size: &mut ArenaSize) -> Result<(), ArenaError> {
        size.add::<Node>(513)
    }

    pub(crate) fn new_in(arena: &mut Arena<'_>) -> Result<Self, ArenaError> {
        Ok(Self {
            nodes: arena.allocate_reusable_vec(513)?,
        })
    }

    #[doc(hidden)]
    #[cfg(test)]
    pub fn retained_node_capacity(&self) -> usize {
        self.nodes.capacity()
    }
}

#[cfg(test)]
pub struct BuiltHuffmanTable {
    pub codes: Vec<HuffmanCode>,
    pub max_num_bits: u8,
    pub weights: Vec<u8>,
}

pub struct DescribedHuffmanTable {
    pub codes: Vec<HuffmanCode>,
    pub max_num_bits: u8,
    pub table_description: Vec<u8>,
}

/// Builds C's canonical compression table and consumes it into the weight
/// representation used by `HUF_writeCTable_wksp()`.
///
/// Keeping the sort, tree merge, height redistribution, canonical-code pass,
/// and bits-to-weight pass in one generated-code unit avoids returning an
/// intermediate tree or rebuilding rank metadata in the parent crate.
#[cfg_attr(target_vendor = "apple", link_section = "__TEXT,__rz_hb0")]
#[cfg_attr(target_family = "windows", link_section = ".text$010.rz.hb0")]
#[cfg_attr(
    all(
        not(target_vendor = "apple"),
        not(target_family = "windows"),
        not(target_family = "wasm")
    ),
    link_section = ".text.sorted.010.ruzstd.huffman.build"
)]
#[cfg(test)]
pub fn build_huffman_table(
    counts: &[usize],
    max_bits: usize,
    scratch: &mut HuffmanBuildScratch,
) -> Option<BuiltHuffmanTable> {
    let (codes, max_num_bits) = build_codes(counts, max_bits, scratch)?;
    let weights = with_weights(&codes, max_num_bits, <[u8]>::to_vec);
    Some(BuiltHuffmanTable {
        codes,
        max_num_bits,
        weights,
    })
}

/// Builds the complete canonical table and immediately consumes its bit
/// lengths into the serialized table description. The callback boundary is
/// invoked once per new table and lets the parent reuse its existing exact FSE
/// weight serializer while the hot tree/rank/weight transaction remains in
/// this isolated codegen unit.
#[cfg_attr(target_vendor = "apple", link_section = "__TEXT,__rz_hb1")]
#[cfg_attr(target_family = "windows", link_section = ".text$011.rz.hb1")]
#[cfg_attr(
    all(
        not(target_vendor = "apple"),
        not(target_family = "windows"),
        not(target_family = "wasm")
    ),
    link_section = ".text.sorted.011.ruzstd.huffman.build_described"
)]
pub fn build_described_huffman_table(
    counts: &[usize],
    max_bits: usize,
    scratch: &mut HuffmanBuildScratch,
    describe_weights: fn(&[u8]) -> Vec<u8>,
) -> Option<DescribedHuffmanTable> {
    let (codes, max_num_bits) = build_codes(counts, max_bits, scratch)?;
    let table_description = with_weights(&codes, max_num_bits, describe_weights);
    Some(DescribedHuffmanTable {
        codes,
        max_num_bits,
        table_description,
    })
}

/// Builds the same complete described table while refilling caller-owned
/// output allocations. C alternates current and next entropy tables inside
/// the compression context; this entry point lets the Rust frame model the
/// same lifetime without embedding a fixed 2 KiB table in return values.
#[cfg_attr(target_vendor = "apple", link_section = "__TEXT,__rz_hb2")]
#[cfg_attr(target_family = "windows", link_section = ".text$012.rz.hb2")]
#[cfg_attr(
    all(
        not(target_vendor = "apple"),
        not(target_family = "windows"),
        not(target_family = "wasm")
    ),
    link_section = ".text.sorted.012.ruzstd.huffman.build_described_reusing"
)]
pub fn build_described_huffman_table_reusing(
    counts: &[usize],
    max_bits: usize,
    scratch: &mut HuffmanBuildScratch,
    codes: &mut Vec<HuffmanCode>,
    table_description: &mut Vec<u8>,
    describe_weights: fn(&[u8], &mut Vec<u8>),
) -> Option<u8> {
    let max_num_bits = build_codes_reusing(counts, max_bits, scratch, codes)?;
    with_weights(codes, max_num_bits, |weights| {
        describe_weights(weights, table_description)
    });
    Some(max_num_bits)
}

/// Builds the same caller-owned table while allowing its description callback
/// to borrow a caller-owned workspace. This keeps the generated Huffman
/// transaction independent of the parent's entropy implementation while
/// letting a compression context share C's sequential workspace lifetime.
#[cfg_attr(target_vendor = "apple", link_section = "__TEXT,__rz_hb3")]
#[cfg_attr(target_family = "windows", link_section = ".text$013.rz.hb3")]
#[cfg_attr(
    all(
        not(target_vendor = "apple"),
        not(target_family = "windows"),
        not(target_family = "wasm")
    ),
    link_section = ".text.sorted.013.ruzstd.huffman.build_described_reusing_context"
)]
pub fn build_described_huffman_table_reusing_with_context<Context>(
    counts: &[usize],
    max_bits: usize,
    scratch: &mut HuffmanBuildScratch,
    codes: &mut Vec<HuffmanCode>,
    table_description: &mut Vec<u8>,
    context: &mut Context,
    describe_weights: fn(&[u8], &mut Vec<u8>, &mut Context),
) -> Option<u8> {
    let max_num_bits = build_codes_reusing(counts, max_bits, scratch, codes)?;
    with_weights(codes, max_num_bits, |weights| {
        describe_weights(weights, table_description, context)
    });
    Some(max_num_bits)
}

fn build_codes_reusing(
    counts: &[usize],
    max_bits: usize,
    scratch: &mut HuffmanBuildScratch,
    codes: &mut Vec<HuffmanCode>,
) -> Option<u8> {
    if counts.len() > 256 {
        return None;
    }
    let max_bits = max_bits.clamp(1, MAX_HUFFMAN_BITS);
    if counts.iter().filter(|count| **count != 0).count() > 1usize << max_bits {
        return None;
    }
    let leaf_count = build_tree(counts, &mut scratch.nodes)?;
    let max_num_bits = limit_height(&mut scratch.nodes, leaf_count, max_bits)?;

    codes.clear();
    codes.resize(counts.len(), (0, 0));
    let mut counts_by_length = [0u16; 256];
    for node in &scratch.nodes[1..=leaf_count] {
        counts_by_length[usize::from(node.len)] += 1;
        codes[usize::from(node.symbol)].1 = node.len;
    }

    let mut values_by_length = [0u32; 256];
    let mut min_value = 0u32;
    for len in (1..=max_num_bits).rev() {
        values_by_length[len] = min_value;
        min_value += u32::from(counts_by_length[len]);
        min_value >>= 1;
    }
    for code in codes {
        let len = usize::from(code.1);
        if len != 0 {
            code.0 = values_by_length[len];
            values_by_length[len] += 1;
        }
    }

    u8::try_from(max_num_bits).ok()
}

fn build_codes(
    counts: &[usize],
    max_bits: usize,
    scratch: &mut HuffmanBuildScratch,
) -> Option<(Vec<HuffmanCode>, u8)> {
    if counts.len() > 256 {
        return None;
    }
    let max_bits = max_bits.clamp(1, MAX_HUFFMAN_BITS);
    if counts.iter().filter(|count| **count != 0).count() > 1usize << max_bits {
        return None;
    }
    let leaf_count = build_tree(counts, &mut scratch.nodes)?;
    let max_num_bits = limit_height(&mut scratch.nodes, leaf_count, max_bits)?;

    let mut codes = vec![(0, 0); counts.len()];
    let mut counts_by_length = [0u16; 256];
    for node in &scratch.nodes[1..=leaf_count] {
        counts_by_length[usize::from(node.len)] += 1;
        codes[usize::from(node.symbol)].1 = node.len;
    }

    let mut values_by_length = [0u32; 256];
    let mut min_value = 0u32;
    for len in (1..=max_num_bits).rev() {
        values_by_length[len] = min_value;
        min_value += u32::from(counts_by_length[len]);
        min_value >>= 1;
    }
    for code in &mut codes {
        let len = usize::from(code.1);
        if len != 0 {
            code.0 = values_by_length[len];
            values_by_length[len] += 1;
        }
    }

    let max_num_bits = u8::try_from(max_num_bits).ok()?;
    Some((codes, max_num_bits))
}

fn with_weights<R>(codes: &[HuffmanCode], max_num_bits: u8, consume: impl FnOnce(&[u8]) -> R) -> R {
    // This initialized fixed workspace is consumed before return. Its full
    // byte-indexed lookup mirrors C's bitsToWeight table without unsafe code.
    let mut bits_to_weight = [0u8; 256];
    for num_bits in 1..=max_num_bits {
        bits_to_weight[usize::from(num_bits)] = max_num_bits + 1 - num_bits;
    }
    let mut weights = [0u8; 256];
    for (weight, code) in weights.iter_mut().zip(codes) {
        *weight = bits_to_weight[usize::from(code.1)];
    }
    consume(&weights[..codes.len()])
}

fn build_tree(counts: &[usize], nodes: &mut Vec<Node>) -> Option<usize> {
    sort_leaves(counts, nodes)?;
    let leaf_count = nodes.len().saturating_sub(1);
    if leaf_count <= 1 {
        return None;
    }

    let first_parent = leaf_count + 1;
    let root = 2 * leaf_count - 1;
    nodes.resize(2 * leaf_count, Node::barrier());
    let mut smallest_leaf = leaf_count;
    let mut smallest_parent = first_parent;
    for parent in first_parent..=root {
        let left = pop_smallest(nodes, &mut smallest_leaf, &mut smallest_parent);
        let right = pop_smallest(nodes, &mut smallest_leaf, &mut smallest_parent);
        let parent_index = u16::try_from(parent).ok()?;
        nodes[left].parent = parent_index;
        nodes[right].parent = parent_index;
        nodes[parent].count = nodes[left].count.checked_add(nodes[right].count)?;
    }

    nodes[root].len = 0;
    for index in (first_parent..root).rev() {
        nodes[index].len = nodes[parent(nodes[index])].len + 1;
    }
    for index in 1..=leaf_count {
        nodes[index].len = nodes[parent(nodes[index])].len + 1;
    }
    Some(leaf_count)
}

fn limit_height(nodes: &mut [Node], leaf_count: usize, target_bits: usize) -> Option<usize> {
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
        last_below_target = last_below_target.checked_sub(1)?;
    }
    while usize::from(nodes[last_below_target].len) == target_bits {
        last_below_target = last_below_target.checked_sub(1)?;
    }

    total_cost >>= shift;
    if total_cost <= 0 {
        return None;
    }

    let mut rank_last = [RANK_NONE; MAX_HUFFMAN_BITS + 2];
    let mut current_bits = target_bits;
    for position in (1..=last_below_target).rev() {
        let len = usize::from(nodes[position].len);
        if len >= current_bits {
            continue;
        }
        current_bits = len;
        rank_last[target_bits - current_bits] = position;
    }

    while total_cost > 0 {
        let mut bits_to_decrease = highest_bit_set(total_cost as usize);
        for candidate_bits in (2..=bits_to_decrease).rev() {
            let high_position = rank_last[candidate_bits];
            let low_position = rank_last[candidate_bits - 1];
            if high_position == RANK_NONE {
                bits_to_decrease -= 1;
                continue;
            }
            if low_position == RANK_NONE {
                break;
            }
            if nodes[high_position].count <= nodes[low_position].count.saturating_mul(2) {
                break;
            }
            bits_to_decrease -= 1;
        }
        while bits_to_decrease <= target_bits && rank_last[bits_to_decrease] == RANK_NONE {
            bits_to_decrease += 1;
        }
        if bits_to_decrease > target_bits {
            return None;
        }

        total_cost -= 1isize << (bits_to_decrease - 1);
        let position = rank_last[bits_to_decrease];
        nodes[position].len += 1;
        if rank_last[bits_to_decrease - 1] == RANK_NONE {
            rank_last[bits_to_decrease - 1] = position;
        }
        rank_last[bits_to_decrease] = if position > 1
            && usize::from(nodes[position - 1].len) == target_bits - bits_to_decrease
        {
            position - 1
        } else {
            RANK_NONE
        };
    }

    while total_cost < 0 {
        if rank_last[1] == RANK_NONE {
            while usize::from(nodes[last_below_target].len) == target_bits {
                last_below_target = last_below_target.checked_sub(1)?;
            }
            let position = last_below_target + 1;
            nodes[position].len = nodes[position].len.checked_sub(1)?;
            rank_last[1] = position;
            total_cost += 1;
            continue;
        }
        let position = rank_last[1] + 1;
        if position > leaf_count {
            return None;
        }
        nodes[position].len = nodes[position].len.checked_sub(1)?;
        rank_last[1] = position;
        total_cost += 1;
    }
    Some(target_bits)
}

fn sort_leaves(counts: &[usize], nodes: &mut Vec<Node>) -> Option<()> {
    const TABLE_SIZE: usize = 192;
    const MAX_COUNT_LOG: usize = 32;
    const LOG_BUCKETS_BEGIN: usize = (TABLE_SIZE - 1) - MAX_COUNT_LOG - 1;
    const DISTINCT_CUTOFF: usize = LOG_BUCKETS_BEGIN + 7;

    #[derive(Clone, Copy, Default)]
    struct RankPosition {
        base: usize,
        current: usize,
    }

    fn rank_index(count: usize) -> usize {
        if count < DISTINCT_CUTOFF {
            count
        } else {
            high_bit(count) + LOG_BUCKETS_BEGIN
        }
    }

    let mut positions = [RankPosition::default(); TABLE_SIZE];
    let mut nonzero_count = 0usize;
    for &count in counts {
        nonzero_count += usize::from(count != 0);
        let rank = rank_index(count);
        if rank >= TABLE_SIZE - 1 {
            return None;
        }
        positions[rank].base += 1;
    }
    for rank in (1..TABLE_SIZE).rev() {
        positions[rank - 1].base += positions[rank].base;
        positions[rank - 1].current = positions[rank - 1].base;
    }

    nodes.clear();
    nodes.resize(nonzero_count + 1, Node::barrier());
    for (symbol, &count) in counts.iter().enumerate() {
        if count == 0 {
            continue;
        }
        let rank = rank_index(count) + 1;
        let position = positions[rank].current;
        positions[rank].current += 1;
        nodes[position + 1] = Node {
            count: u32::try_from(count).ok()?,
            symbol: u8::try_from(symbol).ok()?,
            parent: NODE_NONE,
            len: 0,
        };
    }
    for rank in &positions[DISTINCT_CUTOFF..TABLE_SIZE - 1] {
        let start = rank.base.min(nonzero_count) + 1;
        let end = rank.current.min(nonzero_count) + 1;
        if end.saturating_sub(start) > 1 {
            quick_sort(&mut nodes[start..end]);
        }
    }
    Some(())
}

fn quick_sort(nodes: &mut [Node]) {
    const INSERTION_THRESHOLD: usize = 8;
    if nodes.len() <= 1 {
        return;
    }
    if nodes.len() - 1 < INSERTION_THRESHOLD {
        insertion_sort(nodes);
        return;
    }
    let mut low = 0usize;
    let mut high = nodes.len() - 1;
    while low < high {
        let split = partition(nodes, low, high);
        if split - low < high - split {
            if low < split {
                quick_sort(&mut nodes[low..split]);
            }
            low = split + 1;
        } else {
            if split < high {
                quick_sort(&mut nodes[split + 1..=high]);
            }
            if split == 0 {
                break;
            }
            high = split - 1;
        }
    }
}

fn insertion_sort(nodes: &mut [Node]) {
    for index in 1..nodes.len() {
        let key = nodes[index];
        let mut position = index;
        while position > 0 && nodes[position - 1].count < key.count {
            nodes[position] = nodes[position - 1];
            position -= 1;
        }
        nodes[position] = key;
    }
}

fn partition(nodes: &mut [Node], low: usize, high: usize) -> usize {
    let pivot = nodes[high].count;
    let mut boundary = low;
    for index in low..high {
        if nodes[index].count > pivot {
            nodes.swap(boundary, index);
            boundary += 1;
        }
    }
    nodes.swap(boundary, high);
    boundary
}

fn pop_smallest(nodes: &[Node], leaf: &mut usize, parent: &mut usize) -> usize {
    if nodes[*leaf].count < nodes[*parent].count {
        let index = *leaf;
        *leaf -= 1;
        index
    } else {
        let index = *parent;
        *parent += 1;
        index
    }
}

fn parent(node: Node) -> usize {
    usize::from(node.parent)
}

fn high_bit(value: usize) -> usize {
    usize::BITS as usize - 1 - value.leading_zeros() as usize
}

fn highest_bit_set(value: usize) -> usize {
    usize::BITS as usize - value.leading_zeros() as usize
}

#[derive(Clone, Copy)]
struct Node {
    count: u32,
    parent: u16,
    symbol: u8,
    len: u8,
}

impl Node {
    const fn barrier() -> Self {
        Self {
            count: u32::MAX,
            parent: NODE_NONE,
            symbol: 0,
            len: 0,
        }
    }
}
