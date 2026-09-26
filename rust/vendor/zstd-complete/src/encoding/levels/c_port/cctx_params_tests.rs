use super::{
    cctx_params::{CctxParameters, ParamSwitch},
    params::{CompressionParameters, Strategy},
};

#[test]
fn cctx_params_resolve_row_matchfinder_like_c() {
    let params = CctxParameters::for_level(5, 512 * 1024, 0);

    assert_eq!(params.compression.strategy, Strategy::Greedy);
    assert_eq!(params.use_row_match_finder, ParamSwitch::Enable);

    let tiny = CctxParameters::for_level(5, 16 * 1024, 0);
    assert_eq!(tiny.compression.strategy, Strategy::Lazy);
    assert_eq!(tiny.use_row_match_finder, ParamSwitch::Disable);
}

#[test]
fn cctx_params_resolve_post_block_splitter_like_c() {
    let level16 = CctxParameters::for_level(16, 512 * 1024, 0);
    let level15 = CctxParameters::for_level(15, 512 * 1024, 0);

    assert_eq!(level16.compression.strategy, Strategy::BtOpt);
    assert_eq!(level16.post_block_splitter, ParamSwitch::Enable);
    assert_eq!(level15.compression.strategy, Strategy::BtLazy2);
    assert_eq!(level15.post_block_splitter, ParamSwitch::Disable);
}

#[test]
fn cctx_params_resolve_ldm_for_large_level22_like_c() {
    let level22 = CctxParameters::for_level(22, 256 * 1024 * 1024, 0);

    assert_eq!(level22.compression.window_log, 27);
    assert_eq!(level22.compression.strategy, Strategy::BtUltra2);
    assert_eq!(level22.ldm.enable_ldm, ParamSwitch::Enable);
    assert_eq!(level22.ldm.window_log, 27);
    assert_eq!(level22.ldm.hash_rate_log, 4);
    assert_eq!(level22.ldm.hash_log, 23);
    assert_eq!(level22.ldm.min_match_length, 32);
    assert_eq!(level22.ldm.bucket_size_log, 8);
}

#[test]
fn cctx_params_keep_ldm_disabled_below_c_threshold() {
    let level21 = CctxParameters::for_level(21, 256 * 1024 * 1024, 0);
    let small_level22 = CctxParameters::for_level(22, 128 * 1024, 0);

    assert_eq!(level21.compression.window_log, 26);
    assert_eq!(level21.ldm.enable_ldm, ParamSwitch::Disable);
    assert_eq!(small_level22.compression.window_log, 17);
    assert_eq!(small_level22.ldm.enable_ldm, ParamSwitch::Disable);
}

#[test]
fn cctx_params_external_repcode_search_uses_c_level_gate() {
    assert_eq!(
        CctxParameters::for_level(9, 512 * 1024, 0).search_for_external_repcodes,
        ParamSwitch::Disable
    );
    assert_eq!(
        CctxParameters::for_level(10, 512 * 1024, 0).search_for_external_repcodes,
        ParamSwitch::Enable
    );
}

#[test]
fn cctx_params_default_target_c_block_size_is_disabled_like_c() {
    let params = CctxParameters::for_level(16, 512 * 1024, 0);

    assert_eq!(params.target_c_block_size, 0);
    assert!(!params.use_target_c_block_size());
}

#[test]
fn cctx_params_target_c_block_size_clamps_small_nonzero_values_like_c() {
    let mut params = CctxParameters::for_level(16, 512 * 1024, 0);

    assert!(params.set_target_c_block_size(1));
    assert_eq!(params.target_c_block_size, 1340);
    assert!(params.use_target_c_block_size());
}

#[test]
fn cctx_params_target_c_block_size_rejects_values_above_c_bound() {
    let mut params = CctxParameters::for_level(16, 512 * 1024, 0);

    assert!(!params.set_target_c_block_size(128 * 1024 + 1));
    assert_eq!(params.target_c_block_size, 0);
    assert!(!params.use_target_c_block_size());
}

#[test]
fn cctx_params_can_resolve_from_explicit_compression_params() {
    let compression = CompressionParameters {
        window_log: 27,
        chain_log: 27,
        hash_log: 25,
        search_log: 9,
        min_match: 3,
        target_length: 999,
        strategy: Strategy::BtUltra2,
    };
    let resolved = CctxParameters::from_compression_parameters(22, compression, 256 * 1024 * 1024);

    assert_eq!(resolved.ldm.enable_ldm, ParamSwitch::Enable);
    assert_eq!(resolved.post_block_splitter, ParamSwitch::Enable);
}

#[test]
fn cctx_params_resolve_block_size_from_window_and_pledged_size_like_c() {
    let mut compression = CompressionParameters::for_level(1, 4096, 0);
    compression.window_log = 10;

    let window_limited = CctxParameters::from_compression_parameters(1, compression, 4096);
    assert_eq!(window_limited.max_block_size, 1024);

    let pledged_limited = CctxParameters::from_compression_parameters(1, compression, 512);
    assert_eq!(pledged_limited.max_block_size, 512);

    let empty = CctxParameters::from_compression_parameters(1, compression, 0);
    assert_eq!(empty.max_block_size, 1);
}
