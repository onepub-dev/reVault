use super::{
    params::{should_attach_dict_by_default, CParamMode, ZSTD_CONTENTSIZE_UNKNOWN},
    CompressionParameters, Strategy, MAX_COMPRESSION_LEVEL, MIN_COMPRESSION_LEVEL,
};

fn params(
    window_log: u32,
    chain_log: u32,
    hash_log: u32,
    search_log: u32,
    min_match: u32,
    target_length: u32,
    strategy: Strategy,
) -> CompressionParameters {
    CompressionParameters {
        window_log,
        chain_log,
        hash_log,
        search_log,
        min_match,
        target_length,
        strategy,
    }
}

#[test]
fn level_zero_uses_c_default_level_for_unknown_size() {
    assert_eq!(
        CompressionParameters::for_level(0, 0, 0),
        params(21, 16, 17, 1, 5, 0, Strategy::DFast)
    );
}

#[test]
fn unknown_size_uses_large_source_table() {
    assert_eq!(
        CompressionParameters::for_level(1, 0, 0),
        params(19, 13, 14, 1, 7, 0, Strategy::Fast)
    );
    assert_eq!(
        CompressionParameters::for_level(19, 0, 0),
        params(23, 24, 22, 7, 3, 256, Strategy::BtUltra2)
    );
}

#[test]
fn source_size_selects_c_size_tiers() {
    assert_eq!(
        CompressionParameters::for_level(3, 256 * 1024 + 1, 0),
        params(19, 16, 17, 1, 5, 0, Strategy::DFast)
    );
    assert_eq!(
        CompressionParameters::for_level(3, 256 * 1024, 0),
        params(18, 16, 16, 1, 4, 0, Strategy::DFast)
    );
    assert_eq!(
        CompressionParameters::for_level(3, 128 * 1024, 0),
        params(17, 15, 16, 2, 5, 0, Strategy::DFast)
    );
    assert_eq!(
        CompressionParameters::for_level(3, 16 * 1024, 0),
        params(14, 14, 15, 2, 4, 0, Strategy::DFast)
    );
}

#[test]
fn known_tiny_source_clamps_work_tables_like_c() {
    assert_eq!(
        CompressionParameters::for_level(3, 1, 0),
        params(10, 6, 7, 2, 4, 0, Strategy::DFast)
    );
}

#[test]
fn compression_level_is_clamped_to_c_bounds() {
    assert_eq!(
        CompressionParameters::for_level(MAX_COMPRESSION_LEVEL + 100, 0, 0),
        CompressionParameters::for_level(MAX_COMPRESSION_LEVEL, 0, 0)
    );
    assert_eq!(
        CompressionParameters::for_level(-5, 0, 0),
        params(19, 12, 13, 1, 6, 5, Strategy::Fast)
    );
    assert_eq!(
        CompressionParameters::for_level(MIN_COMPRESSION_LEVEL - 1, 0, 0),
        params(19, 12, 13, 1, 6, 128 * 1024, Strategy::Fast)
    );
}

#[test]
fn public_unknown_size_with_dictionary_uses_c_wrapping_row_size() {
    assert_eq!(
        CompressionParameters::for_level(3, 0, 1024),
        params(14, 14, 15, 2, 4, 0, Strategy::DFast)
    );
}

#[test]
fn attach_dict_mode_ignores_dictionary_size() {
    let with_dict =
        CompressionParameters::for_level_with_mode(3, 1, 1_000_000, CParamMode::AttachDict);
    let without_dict = CompressionParameters::for_level_with_mode(3, 1, 0, CParamMode::AttachDict);

    assert_eq!(with_dict, without_dict);
    assert_eq!(with_dict, params(10, 6, 7, 2, 4, 0, Strategy::DFast));
}

#[test]
fn create_cdict_mode_assumes_small_source_when_size_is_unknown() {
    assert_eq!(
        CompressionParameters::for_level_with_mode(
            3,
            ZSTD_CONTENTSIZE_UNKNOWN,
            1024,
            CParamMode::CreateCDict,
        ),
        params(11, 11, 12, 2, 4, 0, Strategy::DFast)
    );

    assert_eq!(
        CompressionParameters::for_level_with_mode(
            3,
            ZSTD_CONTENTSIZE_UNKNOWN,
            1024,
            CParamMode::NoAttachDict,
        ),
        params(14, 14, 15, 2, 4, 0, Strategy::DFast)
    );
}

#[test]
fn default_attach_dict_mode_uses_c_strategy_cutoffs() {
    assert!(should_attach_dict_by_default(Strategy::Fast, 8 * 1024));
    assert!(!should_attach_dict_by_default(Strategy::Fast, 8 * 1024 + 1));

    assert!(should_attach_dict_by_default(Strategy::DFast, 16 * 1024));
    assert!(!should_attach_dict_by_default(
        Strategy::DFast,
        16 * 1024 + 1
    ));

    for strategy in [
        Strategy::Greedy,
        Strategy::Lazy,
        Strategy::Lazy2,
        Strategy::BtLazy2,
        Strategy::BtOpt,
    ] {
        assert!(should_attach_dict_by_default(strategy, 32 * 1024));
        assert!(!should_attach_dict_by_default(strategy, 32 * 1024 + 1));
    }

    assert!(should_attach_dict_by_default(Strategy::BtUltra, 8 * 1024));
    assert!(!should_attach_dict_by_default(
        Strategy::BtUltra,
        8 * 1024 + 1
    ));
    assert!(should_attach_dict_by_default(Strategy::BtUltra2, 8 * 1024));
    assert!(!should_attach_dict_by_default(
        Strategy::BtUltra2,
        8 * 1024 + 1
    ));
}

#[test]
fn default_attach_dict_mode_accepts_unknown_size_and_focused_greedy_case() {
    assert!(should_attach_dict_by_default(
        Strategy::Fast,
        ZSTD_CONTENTSIZE_UNKNOWN
    ));
    assert!(should_attach_dict_by_default(Strategy::Greedy, 31_858));
}

#[test]
fn attach_adjustment_preserves_cdict_selected_strategy() {
    let cdict = CompressionParameters::for_level_with_mode(
        5,
        ZSTD_CONTENTSIZE_UNKNOWN,
        64 * 1024,
        CParamMode::CreateCDict,
    );
    let attached = cdict.adjusted_for_mode(31_858, 64 * 1024, CParamMode::AttachDict);

    assert_eq!(attached.strategy, cdict.strategy);
    assert_eq!(attached.strategy, Strategy::Greedy);
}
