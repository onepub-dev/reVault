//! Shared setup for C-style dictionary frame encoders.

use alloc::vec::Vec;

use super::{
    c_frame_header::write_frame_header, cctx_params::CctxParameters,
    compress_bound::compress_bound, dictionary::ParsedDictionary, frame_state::FrameBlockState,
    params::CParamMode,
};

pub(crate) struct DictionaryFrameContext {
    pub(crate) combined: Vec<u8>,
    pub(crate) dict_len: usize,
    pub(crate) cctx: CctxParameters,
    pub(crate) output: Vec<u8>,
    pub(crate) frame_state: FrameBlockState,
    pub(crate) opt_price_seeds: Option<super::opt_price::DictionaryPriceSeeds>,
}

impl DictionaryFrameContext {
    pub(crate) fn new(src: &[u8], level: i32, dictionary: ParsedDictionary<'_>) -> Self {
        let original_dict_len = dictionary.content.len();
        let cctx = CctxParameters::for_level_with_mode(
            level,
            src.len() as u64,
            original_dict_len,
            CParamMode::NoAttachDict,
        );
        Self::new_with_cctx(src, dictionary, cctx)
    }

    pub(crate) fn new_with_cctx(
        src: &[u8],
        dictionary: ParsedDictionary<'_>,
        cctx: CctxParameters,
    ) -> Self {
        Self::new_with_cctx_and_dictionary_params(src, dictionary, cctx, cctx.compression)
    }

    /// Builds an active frame while sizing retained dictionary content from
    /// the dictionary match state rather than the active source tables.
    pub(crate) fn new_with_cctx_and_dictionary_params(
        src: &[u8],
        dictionary: ParsedDictionary<'_>,
        cctx: CctxParameters,
        dictionary_params: super::params::CompressionParameters,
    ) -> Self {
        cctx.assert_resolved();
        let params = cctx.compression;
        let loaded_dictionary = loaded_dictionary_content(dictionary_params, dictionary.content);
        let dict_len = loaded_dictionary.len();
        let mut combined = Vec::with_capacity(dict_len + src.len());
        combined.extend_from_slice(loaded_dictionary);
        combined.extend_from_slice(src);

        let dictionary_id = (dictionary.dict_id != 0).then_some(dictionary.dict_id);
        let opt_price_seeds = dictionary.initial_opt_price_seeds();
        let mut output = Vec::with_capacity(compress_bound(src.len()));
        write_frame_header(&mut output, src.len(), params, dictionary_id);
        let frame_state =
            FrameBlockState::with_dictionary(params, cctx.max_block_size, &dictionary);

        Self {
            combined,
            dict_len,
            cctx,
            output,
            frame_state,
            opt_price_seeds,
        }
    }

    pub(crate) fn src_end(&self) -> usize {
        self.combined.len()
    }

    pub(crate) fn loaded_dict_end_for_block(
        &self,
        block_end: usize,
        params: super::params::CompressionParameters,
    ) -> usize {
        let window_size = 1_usize << params.window_log;
        if block_end <= self.dict_len.saturating_add(window_size) {
            self.dict_len
        } else {
            0
        }
    }
}

fn loaded_dictionary_content(params: super::params::CompressionParameters, dict: &[u8]) -> &[u8] {
    let table_limited_size = 1_usize << (params.hash_log + 3).max(params.chain_log + 1).min(31);
    if dict.len() > table_limited_size {
        &dict[dict.len() - table_limited_size..]
    } else {
        dict
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::encoding::levels::c_port::dictionary::{parse_dictionary, DictionaryContentType};
    use alloc::vec;

    #[test]
    fn large_dictionary_loads_only_suffix_like_c() {
        let dictionary = (0..(256 * 1024 + 1024))
            .map(|value| value as u8)
            .collect::<Vec<_>>();
        let parsed = parse_dictionary(&dictionary, DictionaryContentType::Auto, false)
            .unwrap()
            .expect("raw dictionary");

        let context = DictionaryFrameContext::new(b"payload", 1, parsed);

        assert_eq!(context.dict_len, 128 * 1024);
        assert_eq!(
            &context.combined[..context.dict_len],
            &dictionary[dictionary.len() - context.dict_len..]
        );
        assert_eq!(&context.combined[context.dict_len..], b"payload");
    }

    #[test]
    fn loaded_dictionary_expires_after_block_crosses_window_like_c() {
        let dictionary = vec![b'a'; 4096];
        let parsed = parse_dictionary(&dictionary, DictionaryContentType::Auto, false)
            .unwrap()
            .expect("raw dictionary");
        let context = DictionaryFrameContext::new(&vec![b'b'; 4096], 1, parsed);
        let params = context.cctx.compression;
        let window_size = 1_usize << params.window_log;

        assert_eq!(
            context.loaded_dict_end_for_block(context.dict_len + window_size, params),
            context.dict_len
        );
        assert_eq!(
            context.loaded_dict_end_for_block(context.dict_len + window_size + 1, params),
            0
        );
    }

    #[test]
    fn attached_dictionary_retention_uses_dictionary_table_parameters() {
        let dictionary = vec![b'd'; 64 * 1024];
        let parsed = parse_dictionary(&dictionary, DictionaryContentType::Auto, false)
            .unwrap()
            .expect("raw dictionary");
        let active_params = super::super::params::CompressionParameters {
            window_log: 16,
            chain_log: 12,
            hash_log: 12,
            search_log: 6,
            min_match: 3,
            target_length: 128,
            strategy: super::super::params::Strategy::BtUltra,
        };
        let dictionary_params = super::super::params::CompressionParameters {
            chain_log: 17,
            hash_log: 17,
            ..active_params
        };
        let cctx = CctxParameters::from_compression_parameters(16, active_params, 2048);

        let active_limited =
            DictionaryFrameContext::new_with_cctx(b"payload", parsed.clone(), cctx);
        let dictionary_limited = DictionaryFrameContext::new_with_cctx_and_dictionary_params(
            b"payload",
            parsed,
            cctx,
            dictionary_params,
        );

        assert_eq!(active_limited.dict_len, 32 * 1024);
        assert_eq!(dictionary_limited.dict_len, dictionary.len());
    }
}
