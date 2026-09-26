use crate::bit_io::BitWriter;
use alloc::vec::Vec;
#[cfg(feature = "std")]
use std::sync::OnceLock;

mod normalize;
mod table;

pub use normalize::build_table_from_data;
#[cfg(test)]
use normalize::normalize_counts;
#[cfg(test)]
use normalize::normalize_u32_counts_with_table_log;
pub(crate) use normalize::{
    build_huffman_weight_table_from_data, build_huffman_weight_table_from_data_with_scratch,
    build_table_from_data_with_scratch, normalize_counts_with_table_log,
    normalize_counts_with_table_log_into, normalize_u32_counts_with_table_log_into,
    normalized_probabilities_from_counts, normalized_probabilities_from_counts_into,
    optimal_table_log, optimal_table_log_u32,
};
pub use table::FSETable;
pub(crate) use table::{
    build_probability_table_for_estimate, build_probability_table_for_estimate_with_scratch,
    build_rle_table, build_table_from_probabilities, build_table_from_probabilities_with_scratch,
    default_ll_table, default_ll_table_with_scratch, default_ml_table,
    default_ml_table_with_scratch, default_of_table, default_of_table_with_scratch,
    FSETableBuildScratch, SharedFSETable,
};
#[cfg(test)]
use table::{LL_DIST, ML_DIST, OF_DIST};

#[cfg(feature = "std")]
static C_REUSE_FAST_FSE_BUILD_SCRATCH: OnceLock<bool> = OnceLock::new();
#[cfg(feature = "std")]
static C_RECYCLE_FAST_FSE_TABLES: OnceLock<bool> = OnceLock::new();

pub(crate) fn reuses_fast_fse_build_scratch() -> bool {
    #[cfg(feature = "std")]
    {
        *C_REUSE_FAST_FSE_BUILD_SCRATCH.get_or_init(|| {
            std::env::var("RUZSTD_TUNE_C_REUSE_FAST_FSE_BUILD_SCRATCH")
                .map(|value| !matches!(value.as_str(), "0" | "false" | "FALSE" | "off" | "OFF"))
                .unwrap_or(true)
        })
    }
    #[cfg(not(feature = "std"))]
    {
        true
    }
}

pub(crate) fn recycles_fast_fse_tables() -> bool {
    #[cfg(feature = "std")]
    {
        *C_RECYCLE_FAST_FSE_TABLES.get_or_init(|| {
            std::env::var("RUZSTD_TUNE_C_RECYCLE_FAST_FSE_TABLES")
                .map(|value| !matches!(value.as_str(), "0" | "false" | "FALSE" | "off" | "OFF"))
                .unwrap_or(true)
        })
    }
    #[cfg(not(feature = "std"))]
    {
        true
    }
}

pub(crate) struct FSEEncoder<'output, V: AsMut<Vec<u8>>> {
    pub(super) table: FSETable,
    writer: &'output mut BitWriter<V>,
}

impl<V: AsMut<Vec<u8>>> FSEEncoder<'_, V> {
    pub fn new(table: FSETable, writer: &mut BitWriter<V>) -> FSEEncoder<'_, V> {
        FSEEncoder { table, writer }
    }

    pub(crate) fn into_table(self) -> FSETable {
        self.table
    }

    /// Encode data with the table and prepend its serialized description.
    #[cfg(any(test, feature = "fuzz_exports"))]
    pub fn encode(&mut self, data: &[u8]) {
        self.write_table();

        let mut state_index = self.table.c_start_state_index(data[data.len() - 1]);
        for symbol in data[0..data.len() - 1].iter().rev().copied() {
            let (bits, num_bits, next_state) = self.table.encode_symbol(symbol, state_index);
            self.writer
                .write_bits(u64::from(bits), usize::from(num_bits));
            state_index = next_state;
        }
        self.writer.write_bits(
            u64::from(self.table.state_bits(state_index)),
            self.acc_log() as usize,
        );

        self.write_end_marker();
    }

    /// Encode data using two interleaved FSE states.
    pub fn encode_interleaved(&mut self, data: &[u8]) {
        self.write_table();

        let mut ip = data.len();
        let mut state_1;
        let mut state_2;

        if data.len() & 1 != 0 {
            ip -= 1;
            state_1 = self.table.c_start_state_index(data[ip]);
            ip -= 1;
            state_2 = self.table.c_start_state_index(data[ip]);
            ip -= 1;
            Self::encode_symbol(&self.table, self.writer, &mut state_1, data[ip]);
        } else {
            ip -= 1;
            state_2 = self.table.c_start_state_index(data[ip]);
            ip -= 1;
            state_1 = self.table.c_start_state_index(data[ip]);
        }

        if (data.len() - 2) & 2 != 0 {
            let mut bits = 0u64;
            let mut num_bits = 0usize;
            ip -= 1;
            Self::batch_symbol(
                &self.table,
                &mut state_2,
                data[ip],
                &mut bits,
                &mut num_bits,
            );
            ip -= 1;
            Self::batch_symbol(
                &self.table,
                &mut state_1,
                data[ip],
                &mut bits,
                &mut num_bits,
            );
            self.writer.write_bits_64(bits, num_bits);
        }

        while ip > 0 {
            debug_assert!(ip >= 4);
            let mut bits = 0u64;
            let mut num_bits = 0usize;
            ip -= 1;
            Self::batch_symbol(
                &self.table,
                &mut state_2,
                data[ip],
                &mut bits,
                &mut num_bits,
            );
            ip -= 1;
            Self::batch_symbol(
                &self.table,
                &mut state_1,
                data[ip],
                &mut bits,
                &mut num_bits,
            );
            ip -= 1;
            Self::batch_symbol(
                &self.table,
                &mut state_2,
                data[ip],
                &mut bits,
                &mut num_bits,
            );
            ip -= 1;
            Self::batch_symbol(
                &self.table,
                &mut state_1,
                data[ip],
                &mut bits,
                &mut num_bits,
            );
            debug_assert!(num_bits <= 48);
            self.writer.write_bits_64(bits, num_bits);
        }

        let acc_log = self.acc_log() as usize;
        let final_states = u64::from(self.table.state_bits(state_2))
            | (u64::from(self.table.state_bits(state_1)) << acc_log);
        self.writer.write_bits_64(final_states, acc_log * 2);

        self.write_end_marker();
    }

    #[inline(always)]
    fn encode_symbol<VV: AsMut<Vec<u8>>>(
        table: &FSETable,
        writer: &mut BitWriter<VV>,
        state_index: &mut u32,
        symbol: u8,
    ) {
        let (bits, num_bits, next_state) = table.encode_symbol(symbol, *state_index);
        writer.write_bits(u64::from(bits), usize::from(num_bits));
        *state_index = next_state;
    }

    #[inline(always)]
    fn batch_symbol(
        table: &FSETable,
        state_index: &mut u32,
        symbol: u8,
        bits: &mut u64,
        num_bits: &mut usize,
    ) {
        let (symbol_bits, symbol_num_bits, next_state) = table.encode_symbol(symbol, *state_index);
        *bits |= u64::from(symbol_bits) << *num_bits;
        *num_bits += usize::from(symbol_num_bits);
        *state_index = next_state;
    }

    fn write_table(&mut self) {
        self.table.write_table(self.writer);
    }

    fn write_end_marker(&mut self) {
        let bits_to_fill = self.writer.misaligned();
        if bits_to_fill == 0 {
            self.writer.write_bits(1u32, 8);
        } else {
            self.writer.write_bits(1u32, bits_to_fill);
        }
    }

    pub(super) fn acc_log(&self) -> u8 {
        self.table.acc_log()
    }
}

fn write_normalized_probabilities<V: AsMut<Vec<u8>>>(
    probabilities: impl ExactSizeIterator<Item = i32>,
    acc_log: u8,
    writer: &mut BitWriter<V>,
) {
    writer.append_aligned_with(|output| {
        write_normalized_probabilities_bytes(probabilities, acc_log, output)
    });
}

/// Ports `FSE_writeNCount_generic()`'s safe-buffer path. The accumulator never
/// contains more than 31 live bits, so its C `U32` representation is explicit.
fn write_normalized_probabilities_bytes(
    probabilities: impl ExactSizeIterator<Item = i32>,
    acc_log: u8,
    output: &mut Vec<u8>,
) {
    let alphabet_size = probabilities.len();
    let mut probabilities = probabilities.peekable();
    let mut bit_stream = u32::from(acc_log - 5);
    let mut bit_count = 4u32;
    let mut remaining = (1i32 << acc_log) + 1;
    let mut threshold = 1i32 << acc_log;
    let mut nb_bits = u32::from(acc_log) + 1;
    let mut symbol = 0usize;
    let mut previous_is_zero = false;

    while symbol < alphabet_size && remaining > 1 {
        if previous_is_zero {
            let mut start = symbol;
            while probabilities.next_if_eq(&0).is_some() {
                symbol += 1;
            }
            assert!(
                symbol < alphabet_size,
                "normalized probabilities must end non-zero"
            );

            while symbol >= start + 24 {
                start += 24;
                bit_stream |= 0xffff << bit_count;
                output.extend_from_slice(&(bit_stream as u16).to_le_bytes());
                bit_stream >>= 16;
            }
            while symbol >= start + 3 {
                start += 3;
                bit_stream |= 3 << bit_count;
                bit_count += 2;
            }
            bit_stream |= ((symbol - start) as u32) << bit_count;
            bit_count += 2;
            if bit_count > 16 {
                output.extend_from_slice(&(bit_stream as u16).to_le_bytes());
                bit_stream >>= 16;
                bit_count -= 16;
            }
        }

        let mut count = probabilities
            .next()
            .expect("normalized probabilities must sum to the table size");
        symbol += 1;
        let max = 2 * threshold - 1 - remaining;
        remaining -= count.abs();
        count += 1;
        if count >= threshold {
            count += max;
        }
        bit_stream |= (count as u32) << bit_count;
        bit_count += nb_bits;
        bit_count -= u32::from(count < max);
        previous_is_zero = count == 1;
        assert!(
            remaining >= 1,
            "normalized probabilities exceed the table size"
        );
        while remaining < threshold {
            nb_bits -= 1;
            threshold >>= 1;
        }

        if bit_count > 16 {
            output.extend_from_slice(&(bit_stream as u16).to_le_bytes());
            bit_stream >>= 16;
            bit_count -= 16;
        }
    }

    assert_eq!(
        remaining, 1,
        "normalized probabilities do not fill the table"
    );
    let final_bytes = bit_count.div_ceil(8) as usize;
    output.extend_from_slice(&bit_stream.to_le_bytes()[..final_bytes]);
}

pub(crate) fn ncount_cost_from_probabilities(probs: &[i32], acc_log: u8) -> usize {
    let mut probabilities = probs.iter().copied();
    let mut bit_count = 4usize;
    let mut probability_counter = 0usize;
    let probability_sum = 1 << acc_log;
    let mut pending_zero_probability = None;

    while probability_counter < probability_sum {
        let prob = pending_zero_probability.take().unwrap_or_else(|| {
            probabilities
                .next()
                .expect("normalized probabilities must sum to the table size")
        });
        let max_remaining_value = probability_sum - probability_counter + 1;
        let bits_to_write = max_remaining_value.ilog2() as usize + 1;
        let low_threshold = ((1 << bits_to_write) - 1) - max_remaining_value;

        let value = (prob + 1) as u32;
        bit_count += if value < low_threshold as u32 {
            bits_to_write - 1
        } else {
            bits_to_write
        };

        if prob == -1 {
            probability_counter += 1;
        } else if prob > 0 {
            probability_counter += prob as usize;
        } else {
            let mut zeros = 0u8;
            loop {
                let next = probabilities
                    .next()
                    .expect("normalized probabilities must contain the next non-zero symbol");
                if next != 0 {
                    pending_zero_probability = Some(next);
                    break;
                }
                zeros += 1;
                if zeros == 3 {
                    bit_count += 2;
                    zeros = 0;
                }
            }
            bit_count += 2;
        }
    }

    bit_count.next_multiple_of(8)
}

#[cfg(test)]
mod tests;
