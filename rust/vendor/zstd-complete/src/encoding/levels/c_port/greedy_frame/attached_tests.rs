use super::*;

fn active_cctx(level: i32, src_size: usize, dictionary_size: usize) -> CctxParameters {
    CctxParameters::for_level_with_mode(
        level,
        src_size as u64,
        dictionary_size,
        CParamMode::NoAttachDict,
    )
}

#[test]
fn default_attach_routes_lazy_and_lazy2_row_strategies() {
    let src_size = 31_858;
    let dictionary_size = 51_962;

    assert!(attached_dict_cctx(
        6,
        src_size,
        dictionary_size,
        active_cctx(6, src_size, dictionary_size),
        LazyBlockStrategy::Lazy,
    )
    .is_some());
    assert!(attached_dict_cctx(
        8,
        src_size,
        dictionary_size,
        active_cctx(8, src_size, dictionary_size),
        LazyBlockStrategy::Lazy2,
    )
    .is_some());
}

#[test]
fn default_attach_routes_btlazy2_and_excludes_oversized_sources() {
    let dictionary_size = 51_962;
    assert!(attached_dict_cctx(
        11,
        31_858,
        dictionary_size,
        active_cctx(11, 31_858, dictionary_size),
        LazyBlockStrategy::BtLazy2,
    )
    .is_some());
    assert!(attached_dict_cctx(
        8,
        32 * 1024 + 1,
        dictionary_size,
        active_cctx(8, 32 * 1024 + 1, dictionary_size),
        LazyBlockStrategy::Lazy2,
    )
    .is_none());
}
