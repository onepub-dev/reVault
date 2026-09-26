//! Dictionary metadata handling ported from `ZSTD_compress_insertDictionary()`.

use alloc::{rc::Rc, vec::Vec};
use core::{convert::TryFrom, fmt};

use crate::decoding::dictionary::{Dictionary, MAGIC_NUM};
use crate::encoding::frame_compressor::FseTables;
use crate::fse::fse_encoder;
use crate::huff0::huff0_encoder;

use super::{
    hash_chain_match::highbit32, opt_price::DictionaryPriceSeeds, sequence_store::RepeatOffsets,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum DictionaryContentType {
    Auto,
    RawContent,
    FullDict,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum DictionaryKind {
    RawContent,
    Full,
}

#[derive(Clone)]
pub(crate) struct ParsedDictionary<'a> {
    pub(crate) kind: DictionaryKind,
    pub(crate) dict_id: u32,
    pub(crate) raw_size: usize,
    pub(crate) content: &'a [u8],
    pub(crate) repeat_offsets: RepeatOffsets,
    entropy: Option<Rc<DictionaryEntropy>>,
}

impl ParsedDictionary<'_> {
    pub(crate) fn initial_fse_tables(&self) -> FseTables {
        self.entropy
            .as_ref()
            .map(|entropy| entropy.fse_tables.clone())
            .unwrap_or_else(FseTables::new)
    }

    pub(crate) fn initial_huffman_table(&self) -> Option<huff0_encoder::HuffmanTable> {
        self.entropy
            .as_ref()
            .map(|entropy| entropy.huffman_table.clone())
    }

    pub(crate) fn initial_huffman_table_is_valid(&self) -> bool {
        self.entropy
            .as_ref()
            .is_some_and(|entropy| entropy.huffman_repeat_valid)
    }

    pub(crate) fn initial_opt_price_seeds(&self) -> Option<DictionaryPriceSeeds> {
        self.entropy
            .as_ref()
            .map(|entropy| entropy.opt_price_seeds.clone())
    }
}

impl fmt::Debug for ParsedDictionary<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ParsedDictionary")
            .field("kind", &self.kind)
            .field("dict_id", &self.dict_id)
            .field("raw_size", &self.raw_size)
            .field("content", &self.content)
            .field("repeat_offsets", &self.repeat_offsets)
            .field("has_entropy", &self.entropy.is_some())
            .finish()
    }
}

impl PartialEq for ParsedDictionary<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind
            && self.dict_id == other.dict_id
            && self.raw_size == other.raw_size
            && self.content == other.content
            && self.repeat_offsets == other.repeat_offsets
    }
}

impl Eq for ParsedDictionary<'_> {}

#[derive(Clone)]
struct DictionaryEntropy {
    huffman_table: huff0_encoder::HuffmanTable,
    huffman_repeat_valid: bool,
    fse_tables: FseTables,
    opt_price_seeds: DictionaryPriceSeeds,
}

/// Owned dictionary metadata produced once for the prepared-dictionary API.
///
/// This is the safe Rust counterpart of the immutable content and entropy
/// state retained by `ZSTD_CDict`. Match-finder tables are strategy-specific
/// and are prepared by the frame path selected for each compression.
#[derive(Clone)]
pub(crate) struct PreparedDictionary {
    kind: DictionaryKind,
    dict_id: u32,
    raw_size: usize,
    content: Vec<u8>,
    repeat_offsets: RepeatOffsets,
    entropy: Option<Rc<DictionaryEntropy>>,
}

impl PreparedDictionary {
    pub(crate) fn from_bytes(dictionary: &[u8]) -> Result<Option<Self>, DictionaryParseError> {
        let Some(parsed) = parse_dictionary(dictionary, DictionaryContentType::Auto, false)? else {
            return Ok(None);
        };

        Ok(Some(Self {
            kind: parsed.kind,
            dict_id: parsed.dict_id,
            raw_size: parsed.raw_size,
            content: parsed.content.to_vec(),
            repeat_offsets: parsed.repeat_offsets,
            entropy: parsed.entropy,
        }))
    }

    pub(crate) fn as_parsed(&self) -> ParsedDictionary<'_> {
        ParsedDictionary {
            kind: self.kind,
            dict_id: self.dict_id,
            raw_size: self.raw_size,
            content: &self.content,
            repeat_offsets: self.repeat_offsets,
            entropy: self.entropy.clone(),
        }
    }

    pub(crate) fn raw_size(&self) -> usize {
        self.raw_size
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum DictionaryParseError {
    WrongDictionary,
    CorruptedDictionary,
}

pub(crate) fn parse_dictionary<'a>(
    dict: &'a [u8],
    content_type: DictionaryContentType,
    no_dict_id: bool,
) -> Result<Option<ParsedDictionary<'a>>, DictionaryParseError> {
    if dict.len() < 8 {
        return match content_type {
            DictionaryContentType::FullDict => Err(DictionaryParseError::WrongDictionary),
            DictionaryContentType::Auto | DictionaryContentType::RawContent => Ok(None),
        };
    }

    if matches!(content_type, DictionaryContentType::RawContent) {
        return Ok(Some(raw_dictionary(dict)));
    }

    if !has_zstd_dictionary_magic(dict) {
        return match content_type {
            DictionaryContentType::Auto => Ok(Some(raw_dictionary(dict))),
            DictionaryContentType::FullDict => Err(DictionaryParseError::WrongDictionary),
            DictionaryContentType::RawContent => unreachable!("raw content returned earlier"),
        };
    }

    parse_full_dictionary(dict, no_dict_id)
}

fn raw_dictionary(dict: &[u8]) -> ParsedDictionary<'_> {
    ParsedDictionary {
        kind: DictionaryKind::RawContent,
        dict_id: 0,
        raw_size: dict.len(),
        content: dict,
        repeat_offsets: RepeatOffsets::new(),
        entropy: None,
    }
}

fn parse_full_dictionary(
    dict: &[u8],
    no_dict_id: bool,
) -> Result<Option<ParsedDictionary<'_>>, DictionaryParseError> {
    let decoded =
        Dictionary::decode_dict(dict).map_err(|_| DictionaryParseError::CorruptedDictionary)?;
    let content_len = decoded.dict_content.len();
    let content_start = dict
        .len()
        .checked_sub(content_len)
        .ok_or(DictionaryParseError::CorruptedDictionary)?;
    let repeat_offsets = RepeatOffsets::from_offsets(
        decoded.offset_hist[0],
        decoded.offset_hist[1],
        decoded.offset_hist[2],
    );

    validate_repeat_offsets(repeat_offsets, content_len)?;

    Ok(Some(ParsedDictionary {
        kind: DictionaryKind::Full,
        dict_id: if no_dict_id { 0 } else { decoded.id },
        raw_size: dict.len(),
        content: &dict[content_start..],
        repeat_offsets,
        entropy: Some(Rc::new(dictionary_entropy(
            &decoded,
            dict.len() - content_start,
        ))),
    }))
}

fn dictionary_entropy(decoded: &Dictionary, content_len: usize) -> DictionaryEntropy {
    let mut fse_tables = FseTables::new();
    let ll_table = fse_encoder::SharedFSETable::new(fse_encoder::build_table_from_probabilities(
        decoded.fse.literal_lengths.symbol_probabilities(),
        decoded.fse.literal_lengths.accuracy_log,
    ));
    let ml_table = fse_encoder::SharedFSETable::new(fse_encoder::build_table_from_probabilities(
        decoded.fse.match_lengths.symbol_probabilities(),
        decoded.fse.match_lengths.accuracy_log,
    ));
    let of_table = fse_encoder::SharedFSETable::new(fse_encoder::build_table_from_probabilities(
        decoded.fse.offsets.symbol_probabilities(),
        decoded.fse.offsets.accuracy_log,
    ));
    fse_tables.ll_repeat_valid = fse_repeat_is_valid(&ll_table, 35);
    fse_tables.ml_repeat_valid = fse_repeat_is_valid(&ml_table, 52);
    fse_tables.of_repeat_valid =
        fse_repeat_is_valid(&of_table, dictionary_offset_max_symbol(content_len));
    fse_tables.ll_previous = Some(ll_table);
    fse_tables.ml_previous = Some(ml_table);
    fse_tables.of_previous = Some(of_table);

    let opt_price_seeds = dictionary_price_seeds(decoded);

    let huffman_weights = decoded.huf.table.encoder_weights();
    DictionaryEntropy {
        huffman_table: huff0_encoder::HuffmanTable::build_from_weights(&huffman_weights),
        huffman_repeat_valid: huffman_repeat_is_valid(&huffman_weights),
        fse_tables,
        opt_price_seeds,
    }
}

fn fse_repeat_is_valid(table: &fse_encoder::FSETable, max_symbol: u8) -> bool {
    (0..=max_symbol).all(|symbol| table.can_encode_symbol(symbol))
}

fn dictionary_offset_max_symbol(content_len: usize) -> u8 {
    const BLOCK_SIZE: usize = 128 * 1024;
    const MAX_OFFSET_CODE: u32 = 31;

    content_len
        .checked_add(BLOCK_SIZE)
        .and_then(|max_offset| u32::try_from(max_offset).ok())
        .map_or(MAX_OFFSET_CODE, |max_offset| {
            highbit32(max_offset).min(MAX_OFFSET_CODE)
        }) as u8
}

fn huffman_repeat_is_valid(weights: &[usize]) -> bool {
    weights.len() == 256 && weights.iter().all(|weight| *weight != 0)
}

fn dictionary_price_seeds(decoded: &Dictionary) -> DictionaryPriceSeeds {
    let mut seeds = DictionaryPriceSeeds::new();
    let weights = decoded.huf.table.encoder_weights();
    let max_bits = usize::from(decoded.huf.table.max_num_bits);
    for symbol in 0..DictionaryPriceSeeds::LITERAL_COUNT {
        let weight = weights.get(symbol).copied().unwrap_or(0);
        let bit_cost = if weight == 0 {
            0
        } else {
            max_bits + 1 - weight
        };
        seeds.set_literal_freq(symbol, price_seed_frequency(bit_cost, 11));
    }

    for symbol in 0..DictionaryPriceSeeds::LIT_LENGTH_COUNT {
        seeds.set_lit_length_freq(
            symbol,
            fse_price_seed_frequency(
                decoded.fse.literal_lengths.symbol_probabilities(),
                decoded.fse.literal_lengths.accuracy_log,
                symbol,
            ),
        );
    }
    for symbol in 0..DictionaryPriceSeeds::MATCH_LENGTH_COUNT {
        seeds.set_match_length_freq(
            symbol,
            fse_price_seed_frequency(
                decoded.fse.match_lengths.symbol_probabilities(),
                decoded.fse.match_lengths.accuracy_log,
                symbol,
            ),
        );
    }
    for symbol in 0..DictionaryPriceSeeds::OFF_CODE_COUNT {
        seeds.set_off_code_freq(
            symbol,
            fse_price_seed_frequency(
                decoded.fse.offsets.symbol_probabilities(),
                decoded.fse.offsets.accuracy_log,
                symbol,
            ),
        );
    }

    seeds
}

fn fse_price_seed_frequency(probabilities: &[i32], table_log: u8, symbol: usize) -> u32 {
    let probability = probabilities.get(symbol).copied().unwrap_or(0);
    price_seed_frequency(fse_max_nb_bits(probability, table_log), 10)
}

fn fse_max_nb_bits(probability: i32, table_log: u8) -> usize {
    let table_log = u32::from(table_log);
    let table_size = 1_u32 << table_log;
    let delta_nb_bits = match probability {
        0 => ((table_log + 1) << 16) - table_size,
        -1 | 1 => (table_log << 16) - table_size,
        probability => {
            debug_assert!(probability > 1);
            let probability = probability as u32;
            let max_bits_out = table_log - highbit32(probability - 1);
            let min_state_plus = probability << max_bits_out;
            (max_bits_out << 16) - min_state_plus
        }
    };
    ((delta_nb_bits + ((1 << 16) - 1)) >> 16) as usize
}

fn price_seed_frequency(bit_cost: usize, scale_log: usize) -> u32 {
    if bit_cost == 0 {
        1
    } else {
        debug_assert!(bit_cost <= scale_log);
        1_u32 << (scale_log - bit_cost)
    }
}

fn validate_repeat_offsets(
    repeat_offsets: RepeatOffsets,
    content_len: usize,
) -> Result<(), DictionaryParseError> {
    if repeat_offsets
        .as_offsets()
        .iter()
        .any(|&offset| offset == 0 || offset as usize > content_len)
    {
        return Err(DictionaryParseError::CorruptedDictionary);
    }
    Ok(())
}

fn has_zstd_dictionary_magic(dict: &[u8]) -> bool {
    dict.len() >= MAGIC_NUM.len() && dict[..MAGIC_NUM.len()] == MAGIC_NUM
}

#[cfg(test)]
mod tests {
    use alloc::vec::Vec;

    use super::*;

    #[test]
    fn short_auto_dictionary_is_ignored_like_c() {
        let parsed = parse_dictionary(b"abcdefg", DictionaryContentType::Auto, false).unwrap();

        assert_eq!(parsed, None);
    }

    #[test]
    fn short_full_dictionary_is_rejected_like_c() {
        let error = parse_dictionary(b"abcdefg", DictionaryContentType::FullDict, false)
            .expect_err("full dictionary requires a full zstd dictionary header");

        assert_eq!(error, DictionaryParseError::WrongDictionary);
    }

    #[test]
    fn auto_dictionary_without_magic_is_raw_content_like_c() {
        let raw = b"raw-dict-content";

        let parsed = parse_dictionary(raw, DictionaryContentType::Auto, false)
            .unwrap()
            .expect("raw dictionary");

        assert_eq!(parsed.kind, DictionaryKind::RawContent);
        assert_eq!(parsed.dict_id, 0);
        assert_eq!(parsed.content, raw);
        assert_eq!(parsed.repeat_offsets, RepeatOffsets::new());
    }

    #[test]
    fn raw_content_mode_ignores_zstd_magic_like_c() {
        let mut raw = MAGIC_NUM.to_vec();
        raw.extend_from_slice(b"raw-content");

        let parsed = parse_dictionary(&raw, DictionaryContentType::RawContent, false)
            .unwrap()
            .expect("raw dictionary");

        assert_eq!(parsed.kind, DictionaryKind::RawContent);
        assert_eq!(parsed.content, raw.as_slice());
    }

    #[test]
    fn full_dictionary_reports_id_content_and_repcodes() {
        let raw = full_dictionary_with_offsets([3, 10, 25]);

        let parsed = parse_dictionary(&raw, DictionaryContentType::Auto, false)
            .unwrap()
            .expect("full dictionary");

        assert_eq!(parsed.kind, DictionaryKind::Full);
        assert_eq!(parsed.dict_id, 0x4723_2101);
        assert_eq!(parsed.content, valid_full_dictionary_content());
        assert_eq!(
            parsed.repeat_offsets,
            RepeatOffsets::from_offsets(3, 10, 25)
        );
    }

    #[test]
    fn full_dictionary_seeds_encoder_entropy_tables_like_c() {
        let raw = full_dictionary_with_offsets([3, 10, 25]);

        let parsed = parse_dictionary(&raw, DictionaryContentType::Auto, false)
            .unwrap()
            .expect("full dictionary");
        let fse_tables = parsed.initial_fse_tables();

        assert!(parsed.initial_huffman_table().is_some());
        assert!(parsed.initial_opt_price_seeds().is_some());
        assert!(fse_tables.ll_previous.is_some());
        assert!(fse_tables.ml_previous.is_some());
        assert!(fse_tables.of_previous.is_some());
    }

    #[test]
    fn raw_dictionary_does_not_seed_encoder_entropy_tables_like_c() {
        let parsed = parse_dictionary(b"raw-dict-content", DictionaryContentType::Auto, false)
            .unwrap()
            .expect("raw dictionary");
        let fse_tables = parsed.initial_fse_tables();

        assert!(parsed.initial_huffman_table().is_none());
        assert!(parsed.initial_opt_price_seeds().is_none());
        assert!(fse_tables.ll_previous.is_none());
        assert!(fse_tables.ml_previous.is_none());
        assert!(fse_tables.of_previous.is_none());
    }

    #[test]
    fn dictionary_huffman_repeat_validity_requires_all_symbols_like_c() {
        assert!(huffman_repeat_is_valid(&[1; 256]));
        assert!(!huffman_repeat_is_valid(&[1; 255]));
        let mut weights = [1; 256];
        weights[137] = 0;
        assert!(!huffman_repeat_is_valid(&weights));
    }

    #[test]
    fn dictionary_offset_repeat_validity_uses_reachable_codes_like_c() {
        assert_eq!(dictionary_offset_max_symbol(25), 17);
        assert_eq!(
            dictionary_offset_max_symbol((u32::MAX as usize) - 128 * 1024),
            31
        );
        assert_eq!(dictionary_offset_max_symbol(u32::MAX as usize), 31);
    }

    #[test]
    fn full_dictionary_can_hide_dict_id_like_c() {
        let raw = full_dictionary_with_offsets([3, 10, 25]);

        let parsed = parse_dictionary(&raw, DictionaryContentType::Auto, true)
            .unwrap()
            .expect("full dictionary");

        assert_eq!(parsed.dict_id, 0);
    }

    #[test]
    fn full_dictionary_rejects_offsets_past_content_like_c() {
        let raw = full_dictionary_with_offsets([3, 10, 0x00AB_CDEF]);

        let error = parse_dictionary(&raw, DictionaryContentType::Auto, false)
            .expect_err("fixture offsets intentionally exceed dictionary content length");

        assert_eq!(error, DictionaryParseError::CorruptedDictionary);
    }

    fn full_dictionary_with_offsets(offsets: [u32; 3]) -> Vec<u8> {
        let mut raw = Vec::new();
        raw.extend_from_slice(&MAGIC_NUM);
        raw.extend_from_slice(&0x4723_2101_u32.to_le_bytes());
        raw.extend_from_slice(valid_full_dictionary_tables());
        for offset in offsets {
            raw.extend_from_slice(&offset.to_le_bytes());
        }
        raw.extend_from_slice(valid_full_dictionary_content());
        raw
    }

    fn valid_full_dictionary_tables() -> &'static [u8] {
        &[
            54, 16, 192, 155, 4, 0, 207, 59, 239, 121, 158, 116, 220, 93, 114, 229, 110, 41, 249,
            95, 165, 255, 83, 202, 254, 68, 74, 159, 63, 161, 100, 151, 137, 21, 184, 183, 189,
            100, 235, 209, 251, 174, 91, 75, 91, 185, 19, 39, 75, 146, 98, 177, 249, 14, 4, 35, 0,
            0, 0, 40, 40, 20, 10, 12, 204, 37, 196, 1, 173, 122, 0, 4, 0, 128, 1, 2, 2, 25, 32, 27,
            27, 22, 24, 26, 18, 12, 12, 15, 16, 11, 69, 37, 225, 48, 20, 12, 6, 2, 161, 80, 40, 20,
            44, 137, 145, 204, 46, 0, 0, 0, 0, 0, 116, 253, 16, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0,
        ]
    }

    fn valid_full_dictionary_content() -> &'static [u8] {
        &[
            1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 2, 1, 1, 123, 3, 234, 23, 234, 34, 23, 234, 34, 34, 234,
            234,
        ]
    }
}
