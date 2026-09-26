//! Structures that wrap around various decoders to make decoding easier.

use super::super::blocks::sequence_section::Sequence;
use super::decode_buffer::DecodeBuffer;
use crate::decoding::dictionary::Dictionary;
use crate::decoding::dictionary::MAGIC_NUM;
use crate::decoding::errors::DictionaryDecodeError;
use crate::fse::FSETable;
use crate::huff0::HuffmanTable;
use crate::workspace::{Arena, ArenaError, ArenaSize, ReusableVec};

use crate::blocks::sequence_section::{
    MAX_LITERAL_LENGTH_CODE, MAX_MATCH_LENGTH_CODE, MAX_OFFSET_CODE,
};
use crate::common::MAX_BLOCK_SIZE;
use core::convert::TryInto;

// The three-byte sequence-count representation adds an unsigned 16-bit value
// to 0x7f00. Invalid streams can therefore request more records than a valid
// 128 KiB block could execute; static storage must still reject them safely
// without allowing Vec to reallocate caller-owned memory.
const MAX_ENCODED_SEQUENCES: usize = 0x7f00 + u16::MAX as usize;

/// A block level decoding buffer.
pub struct DecoderScratch {
    /// The decoder used for Huffman blocks.
    pub huf: HuffmanScratch,
    /// The decoder used for FSE blocks.
    pub fse: FSEScratch,

    pub buffer: DecodeBuffer,
    pub offset_hist: [u32; 3],

    pub literals_buffer: ReusableVec<u8>,
    pub sequences: ReusableVec<Sequence>,
    pub block_content_buffer: ReusableVec<u8>,
}

impl DecoderScratch {
    pub fn new(window_size: usize) -> DecoderScratch {
        DecoderScratch {
            huf: HuffmanScratch {
                table: HuffmanTable::new(),
            },
            fse: FSEScratch {
                offsets: FSETable::new(MAX_OFFSET_CODE),
                of_rle: None,
                literal_lengths: FSETable::new(MAX_LITERAL_LENGTH_CODE),
                ll_rle: None,
                match_lengths: FSETable::new(MAX_MATCH_LENGTH_CODE),
                ml_rle: None,
            },
            buffer: DecodeBuffer::new(window_size),
            offset_hist: [1, 4, 8],

            block_content_buffer: ReusableVec::new(),
            literals_buffer: ReusableVec::new(),
            sequences: ReusableVec::new(),
        }
    }

    pub(crate) fn new_in(
        arena: &mut Arena<'_>,
        max_window_size: usize,
        max_dictionary_size: usize,
    ) -> Result<Self, ArenaError> {
        let block_size = MAX_BLOCK_SIZE as usize;
        let history = arena.allocate_uninit_slice(
            max_window_size
                .checked_add(block_size)
                .and_then(|size| size.checked_add(1))
                .ok_or(ArenaError::SizeOverflow)?,
        )?;
        let dictionary = arena.allocate_reusable_vec(max_dictionary_size)?;
        Ok(Self {
            huf: HuffmanScratch {
                table: HuffmanTable::new_in(arena)?,
            },
            fse: FSEScratch {
                offsets: FSETable::new_in(arena, MAX_OFFSET_CODE, 8)?,
                of_rle: None,
                literal_lengths: FSETable::new_in(arena, MAX_LITERAL_LENGTH_CODE, 9)?,
                ll_rle: None,
                match_lengths: FSETable::new_in(arena, MAX_MATCH_LENGTH_CODE, 9)?,
                ml_rle: None,
            },
            buffer: DecodeBuffer::from_static_storage(history, dictionary),
            offset_hist: [1, 4, 8],
            block_content_buffer: arena.allocate_reusable_vec(block_size)?,
            literals_buffer: arena.allocate_reusable_vec(block_size)?,
            sequences: arena.allocate_reusable_vec(MAX_ENCODED_SEQUENCES)?,
        })
    }

    pub(crate) fn workspace_size(
        max_window_size: usize,
        max_dictionary_size: usize,
    ) -> Result<usize, ArenaError> {
        let block_size = MAX_BLOCK_SIZE as usize;
        let mut size = ArenaSize::new();
        size.add::<u8>(
            max_window_size
                .checked_add(block_size)
                .and_then(|value| value.checked_add(1))
                .ok_or(ArenaError::SizeOverflow)?,
        )?;
        size.add::<u8>(max_dictionary_size)?;
        HuffmanTable::add_workspace_size(&mut size)?;
        FSETable::add_workspace_size(&mut size, 8)?;
        FSETable::add_workspace_size(&mut size, 9)?;
        FSETable::add_workspace_size(&mut size, 9)?;
        size.add::<u8>(block_size)?;
        size.add::<u8>(block_size)?;
        size.add::<Sequence>(MAX_ENCODED_SEQUENCES)?;
        Ok(size.finish())
    }

    pub fn reset(&mut self, window_size: usize) {
        self.offset_hist = [1, 4, 8];
        self.literals_buffer.clear();
        self.sequences.clear();
        self.block_content_buffer.clear();

        self.buffer.reset(window_size);

        self.fse.literal_lengths.reset();
        self.fse.match_lengths.reset();
        self.fse.offsets.reset();
        self.fse.ll_rle = None;
        self.fse.ml_rle = None;
        self.fse.of_rle = None;

        self.huf.table.reset();
    }

    pub(crate) fn reserve_workspace(&mut self, max_window_size: usize, max_dictionary_size: usize) {
        let block_size = MAX_BLOCK_SIZE as usize;
        self.huf.table.reserve_workspace();
        self.fse.offsets.reserve_workspace(8);
        self.fse.literal_lengths.reserve_workspace(9);
        self.fse.match_lengths.reserve_workspace(9);
        let block_content = block_size.saturating_sub(self.block_content_buffer.len());
        self.block_content_buffer.reserve(block_content);
        let literals = block_size.saturating_sub(self.literals_buffer.len());
        self.literals_buffer.reserve(literals);
        let maximum_sequences = MAX_ENCODED_SEQUENCES;
        let sequences = maximum_sequences.saturating_sub(self.sequences.len());
        self.sequences.reserve(sequences);
        self.buffer.reserve_workspace(
            max_window_size.saturating_add(block_size),
            max_dictionary_size,
        );
    }

    pub fn init_from_dict(&mut self, dict: &Dictionary) {
        self.fse.reinit_from(&dict.fse);
        self.huf.table.reinit_from(&dict.huf.table);
        self.offset_hist = dict.offset_hist;
        self.buffer.dict_content.clear();
        self.buffer
            .dict_content
            .extend_from_slice(&dict.dict_content);
    }

    /// Parses a formatted dictionary directly into the reusable decoder
    /// tables, without constructing an owning [`Dictionary`].
    pub(crate) fn init_from_dict_bytes(
        &mut self,
        raw: &[u8],
    ) -> Result<u32, DictionaryDecodeError> {
        if raw.len() < 8 {
            return Err(DictionaryDecodeError::NotEnoughBytes);
        }
        let magic: [u8; 4] = raw[..4].try_into().expect("four-byte slice");
        if magic != MAGIC_NUM {
            return Err(DictionaryDecodeError::BadMagicNum { got: magic });
        }
        let dict_id = u32::from_le_bytes(raw[4..8].try_into().expect("four-byte slice"));
        let mut tables = &raw[8..];

        let huf_size = self.huf.table.build_decoder(tables)? as usize;
        tables = tables
            .get(huf_size..)
            .ok_or(DictionaryDecodeError::NotEnoughBytes)?;

        let of_size = self.fse.offsets.build_decoder(
            tables,
            crate::decoding::sequence_section_decoder::OF_MAX_LOG,
        )?;
        tables = tables
            .get(of_size..)
            .ok_or(DictionaryDecodeError::NotEnoughBytes)?;

        let ml_size = self.fse.match_lengths.build_decoder(
            tables,
            crate::decoding::sequence_section_decoder::ML_MAX_LOG,
        )?;
        tables = tables
            .get(ml_size..)
            .ok_or(DictionaryDecodeError::NotEnoughBytes)?;

        let ll_size = self.fse.literal_lengths.build_decoder(
            tables,
            crate::decoding::sequence_section_decoder::LL_MAX_LOG,
        )?;
        tables = tables
            .get(ll_size..)
            .ok_or(DictionaryDecodeError::NotEnoughBytes)?;

        let offsets = tables
            .get(..12)
            .ok_or(DictionaryDecodeError::NotEnoughBytes)?;
        self.offset_hist = [
            u32::from_le_bytes(offsets[0..4].try_into().expect("four-byte slice")),
            u32::from_le_bytes(offsets[4..8].try_into().expect("four-byte slice")),
            u32::from_le_bytes(offsets[8..12].try_into().expect("four-byte slice")),
        ];
        self.buffer.dict_content.clear();
        self.buffer.dict_content.extend_from_slice(&tables[12..]);
        Ok(dict_id)
    }

    pub(crate) fn init_from_raw_dictionary(&mut self, dictionary: &[u8]) {
        self.offset_hist = [1, 4, 8];
        self.buffer.dict_content.clear();
        self.buffer.dict_content.extend_from_slice(dictionary);
    }
}

pub struct HuffmanScratch {
    pub table: HuffmanTable,
}

impl HuffmanScratch {
    pub fn new() -> HuffmanScratch {
        HuffmanScratch {
            table: HuffmanTable::new(),
        }
    }
}

impl Default for HuffmanScratch {
    fn default() -> Self {
        Self::new()
    }
}

pub struct FSEScratch {
    pub offsets: FSETable,
    pub of_rle: Option<u8>,
    pub literal_lengths: FSETable,
    pub ll_rle: Option<u8>,
    pub match_lengths: FSETable,
    pub ml_rle: Option<u8>,
}

impl FSEScratch {
    pub fn new() -> FSEScratch {
        FSEScratch {
            offsets: FSETable::new(MAX_OFFSET_CODE),
            of_rle: None,
            literal_lengths: FSETable::new(MAX_LITERAL_LENGTH_CODE),
            ll_rle: None,
            match_lengths: FSETable::new(MAX_MATCH_LENGTH_CODE),
            ml_rle: None,
        }
    }

    pub fn reinit_from(&mut self, other: &Self) {
        self.offsets.reinit_from(&other.offsets);
        self.literal_lengths.reinit_from(&other.literal_lengths);
        self.match_lengths.reinit_from(&other.match_lengths);
        self.of_rle = other.of_rle;
        self.ll_rle = other.ll_rle;
        self.ml_rle = other.ml_rle;
    }
}

impl Default for FSEScratch {
    fn default() -> Self {
        Self::new()
    }
}
