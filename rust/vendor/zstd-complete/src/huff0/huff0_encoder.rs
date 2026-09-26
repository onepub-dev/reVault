use alloc::vec::Vec;
use core::convert::TryFrom;
#[cfg(feature = "std")]
use std::sync::OnceLock;

use crate::{
    bit_io::BitWriter,
    workspace::{Arena, ArenaError, ArenaSize, ReusableVec},
};

mod lengths;
mod metrics;
mod stream;
mod table;
#[cfg(test)]
mod tests;
mod tree;
mod weights;

pub use table::HuffmanTable;
#[cfg(test)]
pub(crate) use tests::four_stream_counts;

const MAX_HUFFMAN_BITS: usize = 11;
const HUFFMAN_NODE_NONE: u16 = u16::MAX;
const HUFFMAN_RANK_NONE: usize = usize::MAX;

#[derive(Clone, Default)]
pub(crate) struct HuffmanBuildScratch {
    nodes: ReusableVec<tree::HuffmanNode>,
    generated: crate::kernel::huff0::HuffmanBuildScratch,
    recycled_tables: ReusableVec<HuffmanTable>,
    workspace_backed: bool,
}

impl HuffmanBuildScratch {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) const fn is_workspace_backed(&self) -> bool {
        self.workspace_backed
    }

    pub(crate) fn add_workspace_size(size: &mut ArenaSize) -> Result<(), ArenaError> {
        size.add::<tree::HuffmanNode>(513)?;
        crate::kernel::huff0::HuffmanBuildScratch::add_workspace_size(size)?;
        size.add::<HuffmanTable>(2)?;
        size.add::<(u32, u8)>(2 * 256)?;
        size.add::<u8>(2 * 512)
    }

    pub(crate) fn new_in(arena: &mut Arena<'_>) -> Result<Self, ArenaError> {
        let mut recycled_tables = arena.allocate_reusable_vec(2)?;
        for _ in 0..2 {
            recycled_tables.push(HuffmanTable {
                codes: arena.allocate_reusable_vec(256)?,
                max_num_bits: 0,
                table_description: arena.allocate_reusable_vec(512)?,
            });
        }
        Ok(Self {
            nodes: arena.allocate_reusable_vec(513)?,
            generated: crate::kernel::huff0::HuffmanBuildScratch::new_in(arena)?,
            recycled_tables,
            workspace_backed: true,
        })
    }

    #[cfg(test)]
    pub(crate) fn retained_generated_node_capacity(&self) -> usize {
        self.generated.retained_node_capacity()
    }
}

#[cfg(feature = "std")]
static C_GENERATED_HUFFMAN_TABLE: OnceLock<bool> = OnceLock::new();
#[cfg(feature = "std")]
static C_REUSE_FAST_HUFFMAN_SCRATCH: OnceLock<bool> = OnceLock::new();
#[cfg(feature = "std")]
static C_RECYCLE_FAST_HUFFMAN_TABLES: OnceLock<bool> = OnceLock::new();
#[cfg(feature = "std")]
static C_REUSE_HUFFMAN_WEIGHT_FSE_SCRATCH: OnceLock<bool> = OnceLock::new();

fn uses_generated_huffman_table() -> bool {
    #[cfg(feature = "std")]
    {
        *C_GENERATED_HUFFMAN_TABLE.get_or_init(|| {
            std::env::var("RUZSTD_TUNE_C_GENERATED_HUFFMAN_TABLE")
                .map(|value| !matches!(value.as_str(), "0" | "false" | "FALSE" | "off" | "OFF"))
                .unwrap_or(true)
        })
    }
    #[cfg(not(feature = "std"))]
    {
        true
    }
}

pub(crate) fn reuses_fast_huffman_scratch() -> bool {
    #[cfg(feature = "std")]
    {
        *C_REUSE_FAST_HUFFMAN_SCRATCH.get_or_init(|| {
            std::env::var("RUZSTD_TUNE_C_REUSE_FAST_HUFFMAN_SCRATCH")
                .map(|value| !matches!(value.as_str(), "0" | "false" | "FALSE" | "off" | "OFF"))
                .unwrap_or(true)
        })
    }
    #[cfg(not(feature = "std"))]
    {
        true
    }
}

fn recycles_fast_huffman_tables() -> bool {
    #[cfg(feature = "std")]
    {
        *C_RECYCLE_FAST_HUFFMAN_TABLES.get_or_init(|| {
            std::env::var("RUZSTD_TUNE_C_RECYCLE_FAST_HUFFMAN_TABLES")
                .map(|value| !matches!(value.as_str(), "0" | "false" | "FALSE" | "off" | "OFF"))
                .unwrap_or(true)
        })
    }
    #[cfg(not(feature = "std"))]
    {
        true
    }
}

fn reuses_huffman_weight_fse_scratch() -> bool {
    #[cfg(feature = "std")]
    {
        *C_REUSE_HUFFMAN_WEIGHT_FSE_SCRATCH.get_or_init(|| {
            std::env::var("RUZSTD_TUNE_C_REUSE_HUFFMAN_WEIGHT_FSE_SCRATCH")
                .map(|value| !matches!(value.as_str(), "0" | "false" | "FALSE" | "off" | "OFF"))
                .unwrap_or(true)
        })
    }
    #[cfg(not(feature = "std"))]
    {
        true
    }
}

pub(crate) struct HuffmanEncoder<'output, 'table, V: AsMut<Vec<u8>>> {
    table: &'table HuffmanTable,
    writer: &'output mut BitWriter<V>,
}

impl<V: AsMut<Vec<u8>>> HuffmanEncoder<'_, '_, V> {
    pub fn new<'o, 't>(
        table: &'t HuffmanTable,
        writer: &'o mut BitWriter<V>,
    ) -> HuffmanEncoder<'o, 't, V> {
        HuffmanEncoder { table, writer }
    }

    /// Encodes the data using the provided table
    /// Writes
    /// * Table description
    /// * Encoded data
    /// * Padding bits to fill up last byte
    pub fn encode(&mut self, data: &[u8], with_table: bool) {
        if with_table {
            self.write_table();
        }
        stream::encode_stream(
            self.writer,
            &self.table.codes,
            self.table.max_num_bits,
            data,
        );
    }

    /// Encodes the data using the provided table in 4 concatenated streams
    /// Writes
    /// * Table description
    /// * Jumptable
    /// * Encoded data in 4 streams, each padded to fill the last byte
    pub fn encode4x(&mut self, data: &[u8], with_table: bool) {
        assert!(data.len() >= 4);

        let split_size = data.len().div_ceil(4);
        let src1 = &data[..split_size];
        let src2 = &data[split_size..split_size * 2];
        let src3 = &data[split_size * 2..split_size * 3];
        let src4 = &data[split_size * 3..];

        if with_table {
            self.write_table();
        }

        if let Ok(codes) = <&[(u32, u8); 256]>::try_from(self.table.codes.as_slice()) {
            let max_num_bits = self.table.max_num_bits;
            self.writer.append_aligned_with(move |output| {
                // SAFETY: this encoder built the canonical table, split input
                // is at least four bytes, and the BMI2 flag comes from CPUID.
                unsafe {
                    crate::kernel::huff0::encode_four_streams(
                        output,
                        codes,
                        max_num_bits,
                        data,
                        crate::cpu::bmi2_supported(),
                    )
                };
            });
            return;
        }

        let size_idx = self.writer.index();
        self.writer.write_bits(0u16, 16);
        self.writer.write_bits(0u16, 16);
        self.writer.write_bits(0u16, 16);

        let index_before = self.writer.index();
        stream::encode_stream(
            self.writer,
            &self.table.codes,
            self.table.max_num_bits,
            src1,
        );
        let size1 = (self.writer.index() - index_before) / 8;

        let index_before = self.writer.index();
        stream::encode_stream(
            self.writer,
            &self.table.codes,
            self.table.max_num_bits,
            src2,
        );
        let size2 = (self.writer.index() - index_before) / 8;

        let index_before = self.writer.index();
        stream::encode_stream(
            self.writer,
            &self.table.codes,
            self.table.max_num_bits,
            src3,
        );
        let size3 = (self.writer.index() - index_before) / 8;

        stream::encode_stream(
            self.writer,
            &self.table.codes,
            self.table.max_num_bits,
            src4,
        );

        assert!(size1 <= u16::MAX as usize);
        assert!(size2 <= u16::MAX as usize);
        assert!(size3 <= u16::MAX as usize);

        self.writer.change_bits(size_idx, size1 as u16, 16);
        self.writer.change_bits(size_idx + 16, size2 as u16, 16);
        self.writer.change_bits(size_idx + 32, size3 as u16, 16);
    }

    fn write_table(&mut self) {
        for byte in &self.table.table_description {
            self.writer.write_bits(*byte, 8);
        }
    }
}
