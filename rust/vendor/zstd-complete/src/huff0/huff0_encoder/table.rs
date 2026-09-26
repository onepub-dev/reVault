use alloc::vec::Vec;
use core::convert::TryFrom;

use crate::fse::fse_encoder::FSETableBuildScratch;

use super::{
    lengths::{
        high_bit, highest_bit_set, invalid_huffman_tree, length_units, rank_limited_weights,
        rank_limited_weights_from_sorted_symbols, rank_limited_weights_into,
    },
    tree::{
        base_code_lengths, build_huffman_tree, is_flat_distribution, length_limited_code_lengths,
        length_limited_code_lengths_from_base, limit_huffman_tree_height, HuffmanNode,
    },
    weights::{
        table_description_bytes_from_weights, table_description_bytes_from_weights_reusing,
        table_description_bytes_from_weights_reusing_with_fse_scratch, weights_from_codes,
    },
    HuffmanBuildScratch, MAX_HUFFMAN_BITS,
};
use crate::workspace::ReusableVec;

#[cfg(test)]
use super::tests::four_stream_counts;

#[derive(Clone)]
pub struct HuffmanTable {
    /// Index is the symbol, values are the bitstring in the lower bits of the u32 and the amount of bits in the u8
    pub(super) codes: ReusableVec<(u32, u8)>,
    pub(super) max_num_bits: u8,
    pub(super) table_description: ReusableVec<u8>,
}

impl HuffmanTable {
    #[cfg(feature = "dict_builder")]
    pub(crate) fn table_description(&self) -> &[u8] {
        &self.table_description
    }

    #[cfg(any(test, feature = "fuzz_exports"))]
    pub fn build_from_data(data: &[u8]) -> Self {
        let mut counts = [0; 256];
        let mut max = 0;
        for x in data {
            counts[*x as usize] += 1;
            max = max.max(*x);
        }

        Self::build_from_counts(&counts[..=max as usize])
    }

    pub fn build_from_counts(counts: &[usize]) -> Self {
        let mut scratch = HuffmanBuildScratch::new();
        Self::build_from_counts_with_scratch(counts, &mut scratch)
    }

    pub(crate) fn build_from_counts_with_scratch(
        counts: &[usize],
        scratch: &mut HuffmanBuildScratch,
    ) -> Self {
        Self::build_from_counts_with_max_bits_and_scratch(counts, MAX_HUFFMAN_BITS, scratch)
    }

    pub(crate) fn build_from_counts_with_max_bits(counts: &[usize], max_bits: usize) -> Self {
        let mut scratch = HuffmanBuildScratch::new();
        Self::build_from_counts_with_max_bits_and_scratch(counts, max_bits, &mut scratch)
    }

    pub(crate) fn build_from_counts_with_max_bits_and_scratch(
        counts: &[usize],
        max_bits: usize,
        scratch: &mut HuffmanBuildScratch,
    ) -> Self {
        Self::build_from_counts_with_max_bits_and_workspaces_impl(counts, max_bits, scratch, None)
    }

    pub(crate) fn build_from_counts_with_max_bits_and_workspaces(
        counts: &[usize],
        max_bits: usize,
        scratch: &mut HuffmanBuildScratch,
        fse_scratch: &mut FSETableBuildScratch,
    ) -> Self {
        Self::build_from_counts_with_max_bits_and_workspaces_impl(
            counts,
            max_bits,
            scratch,
            Some(fse_scratch),
        )
    }

    fn build_from_counts_with_max_bits_and_workspaces_impl(
        counts: &[usize],
        max_bits: usize,
        scratch: &mut HuffmanBuildScratch,
        mut fse_scratch: Option<&mut FSETableBuildScratch>,
    ) -> Self {
        assert!(counts.len() <= 256);
        let max_bits = max_bits.clamp(1, MAX_HUFFMAN_BITS);
        let c_table = if scratch.workspace_backed
            || (super::uses_generated_huffman_table() && super::recycles_fast_huffman_tables())
        {
            Self::build_c_table_generated_reusing(
                counts,
                max_bits,
                scratch,
                fse_scratch.as_deref_mut(),
            )
        } else if super::uses_generated_huffman_table() {
            Self::build_c_table_generated(counts, max_bits, &mut scratch.generated)
        } else {
            Self::build_c_table_with_nodes(counts, max_bits, &mut scratch.nodes)
        };
        if let Some(table) = c_table {
            return table;
        }
        if scratch.workspace_backed {
            let mut weights = [0usize; 256];
            let weights = rank_limited_weights_into(counts, &mut weights);
            Self::build_from_weights_reusing(weights, scratch, fse_scratch)
        } else {
            Self::build_from_weights(&rank_limited_weights(counts))
        }
    }

    pub(crate) fn c_fast_table_log(src_size: usize, max_symbol_value: usize) -> usize {
        const FSE_MIN_TABLE_LOG: u32 = 5;
        const FSE_MAX_TABLE_LOG: u32 = 22;

        debug_assert!(src_size > 1);
        let max_bits_src = high_bit(src_size - 1).saturating_sub(1);
        let min_bits_src = high_bit(src_size) + 1;
        let min_bits_symbols = high_bit(max_symbol_value) + 2;
        let min_bits = min_bits_src.min(min_bits_symbols);
        u32::from(MAX_HUFFMAN_BITS as u8)
            .min(max_bits_src)
            .max(min_bits)
            .clamp(FSE_MIN_TABLE_LOG, FSE_MAX_TABLE_LOG)
            .min(MAX_HUFFMAN_BITS as u32) as usize
    }

    #[cfg(test)]
    pub(crate) fn build_smallest_from_counts(
        counts: &[usize],
        data: &[u8],
        four_streams: bool,
    ) -> Self {
        let stream_counts = four_streams.then(|| four_stream_counts(data));
        Self::build_smallest_from_counts_with_stream_counts(counts, stream_counts.as_ref())
    }

    pub(crate) fn build_smallest_from_counts_with_stream_counts(
        counts: &[usize],
        stream_counts: Option<&[[usize; 256]; 4]>,
    ) -> Self {
        if is_flat_distribution(counts) {
            return Self::build_from_counts(counts);
        }

        let base_lengths = base_code_lengths(counts);
        let base_largest_bits = base_lengths.as_ref().map(|base| base.largest_bits);
        let base_candidate = base_lengths
            .as_ref()
            .and_then(|base| match base_largest_bits {
                Some(largest_bits) if largest_bits <= MAX_HUFFMAN_BITS => {
                    debug_assert_eq!(
                        length_units(&base.lengths, MAX_HUFFMAN_BITS),
                        1usize << MAX_HUFFMAN_BITS
                    );
                    Some(Self::build_from_code_lengths(
                        &base.lengths,
                        MAX_HUFFMAN_BITS,
                    ))
                }
                _ => length_limited_code_lengths_from_base(
                    base.lengths.clone(),
                    base.symbols.clone(),
                    MAX_HUFFMAN_BITS,
                )
                .map(|lengths| Self::build_from_code_lengths(&lengths, MAX_HUFFMAN_BITS)),
            });

        let has_base_candidate = base_candidate.is_some();
        let cached_rank_limited_weights = match base_lengths.as_ref() {
            Some(base) => Some(rank_limited_weights_from_sorted_symbols(
                counts.len(),
                &base.symbols,
            )),
            None if base_candidate.is_none() => Some(rank_limited_weights(counts)),
            None => None,
        };

        let mut best = base_candidate.unwrap_or_else(|| {
            Self::build_from_weights(
                cached_rank_limited_weights
                    .as_ref()
                    .expect("rank-limited fallback is available"),
            )
        });

        let candidate_len = |table: &Self| {
            if let Some(stream_counts) = stream_counts {
                table.encoded_len_from_stream_counts(stream_counts, true)
            } else {
                table.encoded_len_from_counts(counts, true)
            }
        };
        let mut best_len = candidate_len(&best);

        if has_base_candidate {
            let rank_limited_weights =
                cached_rank_limited_weights.unwrap_or_else(|| rank_limited_weights(counts));
            let rank_limited = Self::build_from_weights(&rank_limited_weights);
            if rank_limited.can_encode_counts(counts) {
                let rank_limited_len = candidate_len(&rank_limited);
                if rank_limited_len < best_len {
                    best = rank_limited;
                    best_len = rank_limited_len;
                }
            }
        }

        let nonzero_count = base_lengths.as_ref().map_or_else(
            || counts.iter().filter(|count| **count > 0).count(),
            |base| base.symbols.len(),
        );
        let min_bits = nonzero_count.next_power_of_two().ilog2() as usize;

        let candidate_max_bits_end = base_largest_bits
            .map(|largest_bits| largest_bits.min(MAX_HUFFMAN_BITS))
            .unwrap_or(MAX_HUFFMAN_BITS);
        for max_bits in min_bits.max(1)..candidate_max_bits_end {
            let candidate_lengths = base_lengths
                .as_ref()
                .and_then(|base| {
                    length_limited_code_lengths_from_base(
                        base.lengths.clone(),
                        base.symbols.clone(),
                        max_bits,
                    )
                })
                .or_else(|| length_limited_code_lengths(counts, max_bits));
            let Some(candidate_lengths) = candidate_lengths else {
                continue;
            };
            let candidate = Self::build_from_code_lengths(&candidate_lengths, max_bits);
            let candidate_len = candidate_len(&candidate);
            if candidate_len < best_len
                || (candidate_len == best_len && candidate.max_num_bits < best.max_num_bits)
            {
                best = candidate;
                best_len = candidate_len;
            }
        }

        best
    }

    pub(crate) fn build_c_optimal_depth_from_counts(counts: &[usize]) -> Self {
        let symbol_cardinality = counts.iter().filter(|count| **count > 0).count();
        let min_table_log = high_bit(symbol_cardinality) as usize + 1;
        let Some(base) = base_code_lengths(counts) else {
            return Self::build_from_counts(counts);
        };
        let mut best_table: Option<Self> = None;
        let mut best_size = usize::MAX - 1;

        for table_log in min_table_log.max(1)..=MAX_HUFFMAN_BITS {
            let candidate_lengths = if base.largest_bits <= table_log {
                (length_units(&base.lengths, table_log) == 1usize << table_log)
                    .then(|| base.lengths.clone())
            } else {
                length_limited_code_lengths_from_base(
                    base.lengths.clone(),
                    base.symbols.clone(),
                    table_log,
                )
            };
            let Some(candidate_lengths) = candidate_lengths else {
                continue;
            };
            let candidate = Self::build_from_code_lengths(&candidate_lengths, table_log);
            if usize::from(candidate.max_num_bits) < table_log && table_log > min_table_log {
                break;
            }

            let new_size = candidate.estimated_compressed_size_from_counts(counts)
                + candidate.table_description_len();
            if new_size > best_size + 1 {
                break;
            }
            if new_size < best_size {
                best_size = new_size;
                best_table = Some(candidate);
            }
        }

        best_table.unwrap_or_else(|| Self::build_from_counts(counts))
    }

    pub fn build_from_weights(weights: &[usize]) -> Self {
        // Prepare huffman table with placeholders
        let mut codes = ReusableVec::with_capacity(weights.len());
        codes.resize(weights.len(), (0, 0));
        let mut table = HuffmanTable {
            codes,
            max_num_bits: 0,
            table_description: ReusableVec::new(),
        };

        // Determine the number of bits needed for codes with the lowest weight
        let mut weight_sum = 0usize;
        let mut max_weight = 0usize;
        let mut min_weight = usize::MAX;
        let mut weight_counts = [0usize; MAX_HUFFMAN_BITS + 1];
        for weight in weights.iter().copied() {
            if weight == 0 {
                continue;
            }
            if weight > MAX_HUFFMAN_BITS {
                invalid_huffman_tree();
            }
            weight_sum += 1 << (weight - 1);
            max_weight = max_weight.max(weight);
            min_weight = min_weight.min(weight);
            weight_counts[weight] += 1;
        }
        if !weight_sum.is_power_of_two() {
            panic!("This is an internal error");
        }
        let max_num_bits = highest_bit_set(weight_sum) - 1; // this is a log_2 of a clean power of two
        debug_assert!(min_weight != usize::MAX);
        table.max_num_bits = (max_num_bits - min_weight + 1) as u8;

        let mut next_code_by_weight = [0u32; MAX_HUFFMAN_BITS + 1];
        let mut current_code = 0u32;
        let mut current_weight = 0;
        for (weight, count) in weight_counts
            .iter()
            .copied()
            .enumerate()
            .take(max_weight + 1)
            .skip(1)
        {
            if count == 0 {
                continue;
            }
            current_code >>= weight - current_weight;
            next_code_by_weight[weight] = current_code;
            current_code += count as u32;
            current_weight = weight;
        }

        for (symbol, weight) in weights.iter().copied().enumerate() {
            if weight == 0 {
                continue;
            }
            let current_code = &mut next_code_by_weight[weight];
            let current_num_bits = (max_num_bits - weight + 1) as u8;
            table.codes[symbol] = (*current_code, current_num_bits);
            *current_code += 1;
        }
        table.table_description = ReusableVec::from_owned(table_description_bytes_from_weights(
            &weights_from_codes(&table.codes, table.max_num_bits),
        ));

        table
    }

    fn build_from_weights_reusing(
        weights: &[usize],
        scratch: &mut HuffmanBuildScratch,
        fse_scratch: Option<&mut FSETableBuildScratch>,
    ) -> Self {
        let mut table = scratch.take_recycled_table().unwrap_or_else(|| Self {
            codes: ReusableVec::new(),
            max_num_bits: 0,
            table_description: ReusableVec::new(),
        });
        table.codes.clear();
        table.codes.resize(weights.len(), (0, 0));

        let mut weight_sum = 0usize;
        let mut max_weight = 0usize;
        let mut min_weight = usize::MAX;
        let mut weight_counts = [0usize; MAX_HUFFMAN_BITS + 1];
        for weight in weights.iter().copied() {
            if weight == 0 {
                continue;
            }
            if weight > MAX_HUFFMAN_BITS {
                invalid_huffman_tree();
            }
            weight_sum += 1 << (weight - 1);
            max_weight = max_weight.max(weight);
            min_weight = min_weight.min(weight);
            weight_counts[weight] += 1;
        }
        if !weight_sum.is_power_of_two() {
            panic!("This is an internal error");
        }
        let max_num_bits = highest_bit_set(weight_sum) - 1;
        debug_assert!(min_weight != usize::MAX);
        table.max_num_bits = (max_num_bits - min_weight + 1) as u8;

        let mut next_code_by_weight = [0u32; MAX_HUFFMAN_BITS + 1];
        let mut current_code = 0u32;
        let mut current_weight = 0;
        for (weight, count) in weight_counts
            .iter()
            .copied()
            .enumerate()
            .take(max_weight + 1)
            .skip(1)
        {
            if count == 0 {
                continue;
            }
            current_code >>= weight - current_weight;
            next_code_by_weight[weight] = current_code;
            current_code += count as u32;
            current_weight = weight;
        }
        for (symbol, weight) in weights.iter().copied().enumerate() {
            if weight == 0 {
                continue;
            }
            let current_code = &mut next_code_by_weight[weight];
            let current_num_bits = (max_num_bits - weight + 1) as u8;
            table.codes[symbol] = (*current_code, current_num_bits);
            *current_code += 1;
        }

        let mut byte_weights = [0u8; 256];
        for (destination, &weight) in byte_weights.iter_mut().zip(weights) {
            *destination = weight as u8;
        }
        let (mut description, lease) = table.table_description.lease_vec();
        if let Some(fse_scratch) = fse_scratch {
            table_description_bytes_from_weights_reusing_with_fse_scratch(
                &byte_weights[..weights.len()],
                &mut description,
                fse_scratch,
            );
        } else {
            table_description_bytes_from_weights_reusing(
                &byte_weights[..weights.len()],
                &mut description,
            );
        }
        table.table_description = ReusableVec::recover_vec(description, lease);
        table
    }

    pub(super) fn build_from_code_lengths(lengths: &[usize], max_bits: usize) -> Self {
        let mut codes = ReusableVec::with_capacity(lengths.len());
        codes.resize(lengths.len(), (0, 0));
        let mut table = HuffmanTable {
            codes,
            max_num_bits: 0,
            table_description: ReusableVec::new(),
        };

        let mut weight_sum = 0usize;
        let mut max_weight = 0usize;
        let mut min_weight = usize::MAX;
        let mut weight_counts = [0usize; MAX_HUFFMAN_BITS + 1];
        for len in lengths.iter().copied() {
            if len == 0 {
                continue;
            }
            if len > max_bits {
                invalid_huffman_tree();
            }
            let weight = max_bits - len + 1;
            weight_sum += 1 << (weight - 1);
            max_weight = max_weight.max(weight);
            min_weight = min_weight.min(weight);
            weight_counts[weight] += 1;
        }
        if !weight_sum.is_power_of_two() {
            panic!("This is an internal error");
        }
        let max_num_bits = highest_bit_set(weight_sum) - 1;
        debug_assert!(min_weight != usize::MAX);
        table.max_num_bits = (max_num_bits - min_weight + 1) as u8;

        let mut next_code_by_weight = [0u32; MAX_HUFFMAN_BITS + 1];
        let mut current_code = 0u32;
        let mut current_weight = 0;
        for (weight, count) in weight_counts
            .iter()
            .copied()
            .enumerate()
            .take(max_weight + 1)
            .skip(1)
        {
            if count == 0 {
                continue;
            }
            current_code >>= weight - current_weight;
            next_code_by_weight[weight] = current_code;
            current_code += count as u32;
            current_weight = weight;
        }

        for (symbol, len) in lengths.iter().copied().enumerate() {
            if len == 0 {
                continue;
            }
            let weight = max_bits - len + 1;
            let current_code = &mut next_code_by_weight[weight];
            table.codes[symbol] = (*current_code, len as u8);
            *current_code += 1;
        }
        table.table_description = ReusableVec::from_owned(table_description_bytes_from_weights(
            &weights_from_codes(&table.codes, table.max_num_bits),
        ));

        table
    }

    pub(super) fn build_c_table_with_nodes(
        counts: &[usize],
        max_bits: usize,
        nodes: &mut Vec<HuffmanNode>,
    ) -> Option<Self> {
        let leaf_count = build_huffman_tree(counts, nodes)?;
        let max_num_bits = limit_huffman_tree_height(nodes, leaf_count, max_bits)?;

        let mut codes = ReusableVec::with_capacity(counts.len());
        codes.resize(counts.len(), (0, 0));
        // C indexes compact rank tables after trusting the validated tree depth.
        // Covering every `u8` value makes the same access statically safe and
        // lets LLVM remove the per-node bounds checks without unchecked indexing.
        let mut counts_by_length = [0u16; 256];
        for node in &nodes[1..=leaf_count] {
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
        let max_num_bits = u8::try_from(max_num_bits).expect("Huffman table log fits in u8");
        let table_description =
            table_description_bytes_from_weights(&weights_from_codes(&codes, max_num_bits));
        Some(Self {
            codes,
            max_num_bits,
            table_description: ReusableVec::from_owned(table_description),
        })
    }

    fn build_c_table_generated(
        counts: &[usize],
        max_bits: usize,
        scratch: &mut crate::kernel::huff0::HuffmanBuildScratch,
    ) -> Option<Self> {
        let generated = crate::kernel::huff0::build_described_huffman_table(
            counts,
            max_bits,
            scratch,
            table_description_bytes_from_weights,
        )?;
        Some(Self {
            codes: ReusableVec::from_owned(generated.codes),
            max_num_bits: generated.max_num_bits,
            table_description: ReusableVec::from_owned(generated.table_description),
        })
    }

    fn build_c_table_generated_reusing(
        counts: &[usize],
        max_bits: usize,
        scratch: &mut HuffmanBuildScratch,
        fse_scratch: Option<&mut FSETableBuildScratch>,
    ) -> Option<Self> {
        let recycled = scratch.take_recycled_table().unwrap_or_else(|| Self {
            codes: ReusableVec::new(),
            max_num_bits: 0,
            table_description: ReusableVec::new(),
        });
        let Self {
            codes,
            table_description,
            ..
        } = recycled;
        let (mut codes, codes_lease) = codes.lease_vec();
        let (mut table_description, description_lease) = table_description.lease_vec();
        let max_num_bits = if scratch.workspace_backed || super::reuses_huffman_weight_fse_scratch()
        {
            if let Some(fse_scratch) = fse_scratch {
                crate::kernel::huff0::build_described_huffman_table_reusing_with_context(
                    counts,
                    max_bits,
                    &mut scratch.generated,
                    &mut codes,
                    &mut table_description,
                    fse_scratch,
                    table_description_bytes_from_weights_reusing_with_fse_scratch,
                )
            } else {
                crate::kernel::huff0::build_described_huffman_table_reusing(
                    counts,
                    max_bits,
                    &mut scratch.generated,
                    &mut codes,
                    &mut table_description,
                    table_description_bytes_from_weights_reusing,
                )
            }
        } else {
            crate::kernel::huff0::build_described_huffman_table_reusing(
                counts,
                max_bits,
                &mut scratch.generated,
                &mut codes,
                &mut table_description,
                table_description_bytes_from_weights_reusing,
            )
        };
        let codes = ReusableVec::recover_vec(codes, codes_lease);
        let table_description = ReusableVec::recover_vec(table_description, description_lease);
        let table = Self {
            codes,
            max_num_bits: max_num_bits.unwrap_or(0),
            table_description,
        };
        if max_num_bits.is_some() {
            Some(table)
        } else {
            scratch.recycle_table(table);
            None
        }
    }
}

impl HuffmanBuildScratch {
    pub(crate) fn recycle_table(&mut self, table: HuffmanTable) {
        let retains_table = if self.workspace_backed {
            self.recycled_tables.len() < self.recycled_tables.capacity()
        } else {
            super::recycles_fast_huffman_tables() && self.recycled_tables.is_empty()
        };
        if retains_table {
            self.recycled_tables.push(table);
        }
    }

    fn take_recycled_table(&mut self) -> Option<HuffmanTable> {
        self.recycled_tables.pop()
    }

    #[cfg(test)]
    pub(crate) fn recycled_table_count(&self) -> usize {
        self.recycled_tables.len()
    }
}
