use crate::encoding::levels::c_port::{
    ldm::opt::LdmOptCursor,
    opt_match::{OptMatch, OptMatchBounds},
    opt_path::update_reps,
    opt_price::{OptLevel, OptPriceState, BITCOST_MULTIPLIER, ZSTD_MAX_PRICE},
    opt_state::{ForwardResult, LiteralPriceCache, OptBlockState, Optimal, ZSTD_OPT_NUM},
    params::CompressionParameters,
};

pub(super) fn seed_parser_root<const ULTRA: bool>(
    ip: usize,
    anchor: usize,
    rep: [u32; 3],
    state: &mut OptBlockState,
) {
    let opt_level = opt_level::<ULTRA>();
    let litlen = (ip - anchor) as u32;
    state.opt[0] = Optimal {
        price: price_i32(
            state
                .price_state
                .dynamic_lit_length_price(litlen, opt_level),
        ),
        off: 0,
        mlen: 0,
        litlen,
        rep,
    };
}

#[inline(always)]
pub(super) fn seed_match_prices<const ULTRA: bool>(
    min_match: u32,
    match_count: usize,
    state: &mut OptBlockState,
) -> (usize, i32) {
    let opt_level = opt_level::<ULTRA>();
    let litlen = state.opt[0].litlen;
    for pos in 1..min_match as usize {
        state.opt[pos].price = ZSTD_MAX_PRICE;
        state.opt[pos].mlen = 0;
        state.opt[pos].litlen = litlen + pos as u32;
    }

    let mut last_len = min_match;
    let zero_literal_length_price =
        price_i32(state.price_state.dynamic_lit_length_price(0, opt_level));
    let base_price = state.opt[0].price + zero_literal_length_price;
    for match_index in 0..match_count {
        let OptMatch { off_base, len } = state.matches.get(match_index);
        let offset_price = state
            .price_state
            .dynamic_match_offset_price(off_base, opt_level);
        for pos in last_len..=len {
            state.opt[pos as usize] = Optimal {
                price: base_price
                    + price_i32(
                        offset_price + state.price_state.dynamic_match_length_price(pos, opt_level),
                    ),
                off: off_base,
                mlen: pos,
                litlen: 0,
                ..state.opt[pos as usize]
            };
        }
        last_len = len + 1;
    }

    let last_pos = last_len.saturating_sub(1) as usize;
    state.opt[last_pos + 1].price = ZSTD_MAX_PRICE;
    (last_pos, zero_literal_length_price)
}

#[allow(clippy::too_many_arguments)]
#[cfg_attr(target_vendor = "apple", link_section = "__TEXT,__rz_optf")]
#[cfg_attr(target_family = "windows", link_section = ".text$031.rz.optf")]
#[cfg_attr(
    all(
        not(target_vendor = "apple"),
        not(target_family = "windows"),
        not(target_family = "wasm")
    ),
    link_section = ".text.sorted.031.ruzstd.opt.forward"
)]
pub(super) fn forward_pass<
    const MLS: u32,
    const ULTRA: bool,
    const WITH_LDM: bool,
    const EXT_DICT: bool,
    const LOADED_DICT: bool,
    const ATTACHED_DICT: bool,
>(
    src: &[u8],
    ip: usize,
    block_end: usize,
    ilimit: usize,
    mut last_pos: usize,
    min_match: u32,
    sufficient_len: u32,
    zero_literal_length_price: i32,
    params: CompressionParameters,
    state: &mut OptBlockState,
    block_start: usize,
    mut ldm_cursor: Option<&mut LdmOptCursor<'_>>,
    bounds: OptMatchBounds,
) -> ForwardResult {
    let mut last_stretch = None;
    state.literal_price_cache.begin_pass();
    let one_literal_increment = if ULTRA {
        ll_increment_price(1, opt_level::<ULTRA>(), &state.price_state)
    } else {
        0
    };
    let mut cur = 1_usize;

    while cur <= last_pos && cur <= ZSTD_OPT_NUM {
        update_literal_price::<ULTRA>(
            src,
            ip,
            block_end,
            cur,
            one_literal_increment,
            &mut last_pos,
            state,
        );
        refresh_node_reps(cur, state);

        let inr = ip + cur;
        // C still lets later priced positions collapse into literals here.
        if inr > ilimit {
            cur += 1;
            continue;
        }
        if cur == last_pos {
            break;
        }
        if !ULTRA
            && state.opt[cur + 1].price <= state.opt[cur].price + price_i32(BITCOST_MULTIPLIER / 2)
        {
            cur += 1;
            continue;
        }

        let rep = state.opt[cur].rep;
        let ll0 = state.opt[cur].litlen == 0;
        let match_count = if WITH_LDM {
            super::collect_matches_with_ldm_mls::<MLS, EXT_DICT, LOADED_DICT, ATTACHED_DICT>(
                src,
                inr,
                block_end,
                rep,
                ll0,
                min_match,
                params,
                state,
                block_start,
                ldm_cursor
                    .as_deref_mut()
                    .expect("LDM specialization requires a cursor"),
                bounds,
            )
        } else {
            super::collect_matches_no_ldm_mls::<MLS, EXT_DICT, LOADED_DICT, ATTACHED_DICT>(
                src, inr, block_end, rep, ll0, min_match, params, state, bounds,
            )
        };
        if match_count == 0 {
            cur += 1;
            continue;
        }

        let longest = state.matches.get(match_count - 1);
        if longest.len > sufficient_len
            || cur + longest.len as usize >= ZSTD_OPT_NUM
            || inr + longest.len as usize >= block_end
        {
            last_pos = cur + longest.len as usize;
            last_stretch = Some(Optimal {
                price: state.opt[cur].price,
                off: longest.off_base,
                mlen: longest.len,
                litlen: 0,
                rep,
            });
            break;
        }

        update_match_prices::<ULTRA>(
            cur,
            min_match,
            match_count,
            zero_literal_length_price,
            &mut last_pos,
            state,
        );
        // C refreshes the sentinel after each match-price update so stale
        // prices beyond the current frontier cannot influence later literals.
        state.opt[last_pos + 1].price = ZSTD_MAX_PRICE;
        cur += 1;
    }

    ForwardResult {
        last_pos,
        last_stretch,
    }
}

#[inline(always)]
fn update_literal_price<const ULTRA: bool>(
    src: &[u8],
    ip: usize,
    block_end: usize,
    cur: usize,
    one_literal_increment: i32,
    last_pos: &mut usize,
    state: &mut OptBlockState,
) {
    let opt_level = opt_level::<ULTRA>();
    let previous_price = state.opt[cur - 1].price;
    let litlen = state.opt[cur - 1].litlen + 1;
    let litlen_increment = ll_increment_price(litlen, opt_level, &state.price_state);
    let literal = src[ip + cur - 1];
    let price = previous_price
        + price_i32(raw_literal_cost(
            literal,
            opt_level,
            &state.price_state,
            &mut state.literal_price_cache,
        ))
        + litlen_increment;

    if price <= state.opt[cur].price {
        let previous = state.opt[cur - 1];
        let previous_match = if ULTRA
            && state.opt[cur].litlen == 0
            && one_literal_increment < 0
            && ip + cur < block_end
        {
            Some(state.opt[cur])
        } else {
            None
        };
        state.opt[cur] = Optimal {
            price,
            litlen,
            ..previous
        };

        if let Some(prev_match) = previous_match {
            let next_literal_cost = price_i32(raw_literal_cost(
                src[ip + cur],
                opt_level,
                &state.price_state,
                &mut state.literal_price_cache,
            ));
            let with_one_literal = prev_match.price + next_literal_cost + one_literal_increment;
            let with_more_literals = price
                + next_literal_cost
                + ll_increment_price(litlen + 1, opt_level, &state.price_state);
            if with_one_literal < with_more_literals && with_one_literal < state.opt[cur + 1].price
            {
                let prev = cur - prev_match.mlen as usize;
                state.opt[cur + 1] = Optimal {
                    price: with_one_literal,
                    litlen: 1,
                    rep: update_reps(
                        state.opt[prev].rep,
                        prev_match.off,
                        state.opt[prev].litlen == 0,
                    ),
                    ..prev_match
                };
                *last_pos = (*last_pos).max(cur + 1);
            }
        }
    }
}

#[inline(always)]
fn raw_literal_cost(
    literal: u8,
    opt_level: OptLevel,
    price_state: &OptPriceState,
    cache: &mut LiteralPriceCache,
) -> u32 {
    if let Some(price) = cache.lookup(literal) {
        return price;
    }
    let price = price_state.dynamic_raw_literal_cost(literal, opt_level);
    cache.insert(literal, price);
    price
}

fn refresh_node_reps(cur: usize, state: &mut OptBlockState) {
    let current = state.opt[cur];
    if current.litlen != 0 {
        return;
    }

    let previous_index = cur - current.mlen as usize;
    let previous = state.opt[previous_index];
    state.opt[cur].rep = update_reps(previous.rep, current.off, previous.litlen == 0);
}

#[inline(always)]
fn update_match_prices<const ULTRA: bool>(
    cur: usize,
    min_match: u32,
    match_count: usize,
    zero_literal_length_price: i32,
    last_pos: &mut usize,
    state: &mut OptBlockState,
) {
    let opt_level = opt_level::<ULTRA>();
    let base_price = state.opt[cur].price + zero_literal_length_price;
    let mut previous_len = min_match;

    for match_index in 0..match_count {
        let OptMatch { off_base, len } = state.matches.get(match_index);
        let offset_price = state
            .price_state
            .dynamic_match_offset_price(off_base, opt_level);
        let start_len = previous_len;
        let mut match_len = len;
        while match_len >= start_len {
            let pos = cur + match_len as usize;
            let price = base_price
                + price_i32(
                    offset_price
                        + state
                            .price_state
                            .dynamic_match_length_price(match_len, opt_level),
                );
            if pos > *last_pos || price < state.opt[pos].price {
                while *last_pos < pos {
                    *last_pos += 1;
                    state.opt[*last_pos].price = ZSTD_MAX_PRICE;
                    state.opt[*last_pos].litlen = u32::MAX;
                }
                state.opt[pos] = Optimal {
                    price,
                    off: off_base,
                    mlen: match_len,
                    litlen: 0,
                    ..state.opt[pos]
                };
            } else if !ULTRA {
                break;
            }

            if match_len == start_len {
                break;
            }
            match_len -= 1;
        }
        previous_len = len + 1;
    }
}

#[inline(always)]
fn ll_increment_price(litlen: u32, opt_level: OptLevel, price_state: &OptPriceState) -> i32 {
    price_state.dynamic_lit_length_increment_price(litlen, opt_level)
}

#[inline(always)]
fn price_i32(price: u32) -> i32 {
    debug_assert!(price <= ZSTD_MAX_PRICE as u32);
    price as i32
}

#[inline(always)]
fn opt_level<const ULTRA: bool>() -> OptLevel {
    if ULTRA {
        OptLevel::BtUltra
    } else {
        OptLevel::BtOpt
    }
}
