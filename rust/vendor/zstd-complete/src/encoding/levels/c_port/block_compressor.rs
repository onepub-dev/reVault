//! Block-compressor routing ported from `ZSTD_selectBlockCompressor()`.
//!
//! The C implementation chooses a block compressor from the strategy, resolved
//! row-matchfinder switch, and dictionary mode. Keeping that selection explicit
//! makes the dictionary-mode port testable before every variant has a dedicated
//! Rust implementation.

use super::{cctx_params::ParamSwitch, params::Strategy};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum DictionaryMode {
    NoDict,
    ExtDict,
    DictMatchState,
    DedicatedDictSearch,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct BlockCompressorSelection {
    pub(crate) strategy: Strategy,
    pub(crate) dictionary_mode: DictionaryMode,
    pub(crate) row_match_finder: bool,
}

pub(crate) fn select_block_compressor(
    strategy: Strategy,
    row_matchfinder_mode: ParamSwitch,
    dictionary_mode: DictionaryMode,
) -> Option<BlockCompressorSelection> {
    let row_match_finder = row_match_finder_used(strategy, row_matchfinder_mode);
    let strategy = selected_strategy(strategy, dictionary_mode)?;

    Some(BlockCompressorSelection {
        strategy,
        dictionary_mode,
        row_match_finder,
    })
}

fn selected_strategy(strategy: Strategy, dictionary_mode: DictionaryMode) -> Option<Strategy> {
    match dictionary_mode {
        DictionaryMode::NoDict => Some(strategy),
        DictionaryMode::ExtDict | DictionaryMode::DictMatchState => match strategy {
            Strategy::BtUltra2 => Some(Strategy::BtUltra),
            strategy => Some(strategy),
        },
        DictionaryMode::DedicatedDictSearch => match strategy {
            Strategy::Greedy | Strategy::Lazy | Strategy::Lazy2 => Some(strategy),
            _ => None,
        },
    }
}

fn row_match_finder_used(strategy: Strategy, mode: ParamSwitch) -> bool {
    mode == ParamSwitch::Enable && (Strategy::Greedy..=Strategy::Lazy2).contains(&strategy)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_dict_keeps_btultra2_like_c() {
        let selected = select_block_compressor(
            Strategy::BtUltra2,
            ParamSwitch::Disable,
            DictionaryMode::NoDict,
        )
        .unwrap();

        assert_eq!(selected.strategy, Strategy::BtUltra2);
        assert_eq!(selected.dictionary_mode, DictionaryMode::NoDict);
        assert!(!selected.row_match_finder);
    }

    #[test]
    fn dictionary_modes_route_btultra2_to_btultra_like_c() {
        for dictionary_mode in [DictionaryMode::ExtDict, DictionaryMode::DictMatchState] {
            let selected =
                select_block_compressor(Strategy::BtUltra2, ParamSwitch::Disable, dictionary_mode)
                    .unwrap();

            assert_eq!(selected.strategy, Strategy::BtUltra);
            assert_eq!(selected.dictionary_mode, dictionary_mode);
        }
    }

    #[test]
    fn row_matchfinder_only_applies_to_c_row_strategies() {
        let selected = select_block_compressor(
            Strategy::Lazy2,
            ParamSwitch::Enable,
            DictionaryMode::DictMatchState,
        )
        .unwrap();

        assert!(selected.row_match_finder);

        let bt_selected = select_block_compressor(
            Strategy::BtLazy2,
            ParamSwitch::Enable,
            DictionaryMode::NoDict,
        )
        .unwrap();

        assert!(!bt_selected.row_match_finder);
    }

    #[test]
    fn dedicated_dictionary_search_matches_c_supported_strategies() {
        for strategy in [Strategy::Greedy, Strategy::Lazy, Strategy::Lazy2] {
            assert!(select_block_compressor(
                strategy,
                ParamSwitch::Disable,
                DictionaryMode::DedicatedDictSearch,
            )
            .is_some());
        }

        for strategy in [
            Strategy::Fast,
            Strategy::DFast,
            Strategy::BtLazy2,
            Strategy::BtOpt,
            Strategy::BtUltra,
            Strategy::BtUltra2,
        ] {
            assert!(select_block_compressor(
                strategy,
                ParamSwitch::Disable,
                DictionaryMode::DedicatedDictSearch,
            )
            .is_none());
        }
    }
}
