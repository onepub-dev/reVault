//! Attached-dictionary setup for optimal compression strategies.

use super::OptFrameStrategy;
use crate::encoding::levels::c_port::{
    bt_match::load_attached_dictionary_binary_tree,
    cctx_params::CctxParameters,
    opt_match::OptAttachedDictionary,
    opt_state::OptBlockState,
    params::{should_attach_dict_by_default, CParamMode, CompressionParameters, Strategy},
};

#[cfg(test)]
use crate::encoding::levels::c_port::params::ZSTD_CONTENTSIZE_UNKNOWN;

const C_WINDOW_START_INDEX: usize = 2;

#[derive(Clone, Copy, Debug)]
pub(super) struct AttachedDictCctx {
    pub(super) active_cctx: CctxParameters,
    pub(super) dictionary_params: CompressionParameters,
}

pub(super) fn attached_dict_cctx(
    level: i32,
    src_size: usize,
    dictionary_size: usize,
    active_cctx: CctxParameters,
    frame_strategy: OptFrameStrategy,
) -> Option<AttachedDictCctx> {
    let dictionary_params = CompressionParameters::for_level_with_mode(
        level,
        super::super::params::ZSTD_CONTENTSIZE_UNKNOWN,
        dictionary_size,
        CParamMode::CreateCDict,
    );
    if !should_attach_dict_by_default(dictionary_params.strategy, src_size as u64) {
        return None;
    }

    let mut active_params = dictionary_params.adjusted_for_mode(
        src_size as u64,
        dictionary_size,
        CParamMode::AttachDict,
    );
    active_params.window_log = active_cctx.compression.window_log;
    let expected_strategy = match frame_strategy {
        OptFrameStrategy::BtOpt => Strategy::BtOpt,
        OptFrameStrategy::BtUltra => Strategy::BtUltra,
        OptFrameStrategy::BtUltra2 => Strategy::BtUltra2,
    };
    if active_params.strategy != active_cctx.compression.strategy
        || active_params.strategy != expected_strategy
    {
        return None;
    }

    let mut attached_cctx = active_cctx;
    attached_cctx.compression = active_params;
    Some(AttachedDictCctx {
        active_cctx: attached_cctx,
        dictionary_params,
    })
}

pub(super) fn initialize_attached_dictionary(
    dictionary_src: &[u8],
    dictionary_params: CompressionParameters,
    active_state: &mut OptBlockState,
) -> OptAttachedDictionary {
    let mut dictionary_state = super::super::greedy::GreedyMatchState::new();
    dictionary_state.ensure_tables(dictionary_params);
    dictionary_state.next_to_update = C_WINDOW_START_INDEX;
    load_attached_dictionary_binary_tree(
        dictionary_src,
        dictionary_src.len().saturating_sub(8),
        C_WINDOW_START_INDEX,
        dictionary_params,
        dictionary_params.min_match.clamp(3, 6),
        &mut dictionary_state,
    );
    dictionary_state.next_to_update = dictionary_src.len() + C_WINDOW_START_INDEX;
    dictionary_state.next_to_update3 = dictionary_state.next_to_update;

    active_state.match_state.next_to_update = dictionary_src.len();
    active_state.match_state.next_to_update3 = dictionary_src.len();
    active_state.attached_match_state = Some(dictionary_state);

    OptAttachedDictionary::new(
        0,
        dictionary_src.len(),
        dictionary_params,
        C_WINDOW_START_INDEX,
        dictionary_src.len() + C_WINDOW_START_INDEX,
        dictionary_src.len(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn prepared_active_cctx(level: i32, src_size: usize, dictionary_size: usize) -> CctxParameters {
        let dictionary_params = CompressionParameters::for_level_with_mode(
            level,
            ZSTD_CONTENTSIZE_UNKNOWN,
            dictionary_size,
            CParamMode::CreateCDict,
        );
        let active_params = dictionary_params.adjusted_for_mode(
            src_size as u64,
            dictionary_size,
            CParamMode::AttachDict,
        );
        CctxParameters::from_compression_parameters(level, active_params, src_size as u64)
    }

    #[test]
    fn btopt_uses_c_default_attach_cutoff() {
        let dictionary_size = 51_962;
        let btopt_source_size = 31_858;
        assert!(attached_dict_cctx(
            15,
            btopt_source_size,
            dictionary_size,
            prepared_active_cctx(15, btopt_source_size, dictionary_size),
            OptFrameStrategy::BtOpt,
        )
        .is_some());
        assert!(attached_dict_cctx(
            15,
            32 * 1024 + 1,
            dictionary_size,
            prepared_active_cctx(15, 32 * 1024 + 1, dictionary_size),
            OptFrameStrategy::BtOpt,
        )
        .is_none());
    }

    #[test]
    fn btultra2_uses_c_default_attach_cutoff() {
        let dictionary_size = 4096;
        assert!(attached_dict_cctx(
            19,
            8 * 1024,
            dictionary_size,
            prepared_active_cctx(19, 8 * 1024, dictionary_size),
            OptFrameStrategy::BtUltra2,
        )
        .is_some());
        assert!(attached_dict_cctx(
            19,
            8 * 1024 + 1,
            dictionary_size,
            prepared_active_cctx(19, 8 * 1024 + 1, dictionary_size),
            OptFrameStrategy::BtUltra2,
        )
        .is_none());
    }
}
