//! Repeat-offset match collection for the optimal parser.

use super::{OptMatch, OptMatchBounds, OptMatchTable};
use crate::encoding::levels::c_port::{
    hash_chain_match::{count_match_no_dict, equal_min_match},
    sequence_store::{OffBase, RepeatCode},
};

#[allow(clippy::too_many_arguments)]
#[inline(always)]
pub(in crate::encoding::levels::c_port) fn collect_repcode_matches(
    matches: &mut OptMatchTable,
    src: &[u8],
    ip: usize,
    block_end: usize,
    rep: [u32; 3],
    ll0: bool,
    min_match: u32,
    bounds: OptMatchBounds,
    window_low: usize,
    sufficient_len: usize,
    best_length: &mut usize,
) {
    if !bounds.is_ext_dict() && bounds.attached_dictionary().is_none() {
        collect_repcode_matches_no_dict(
            matches,
            src,
            ip,
            block_end,
            rep,
            ll0,
            min_match,
            window_low,
            sufficient_len,
            best_length,
        );
        return;
    }

    let first_rep = usize::from(ll0);
    let last_rep = 3 + usize::from(ll0);
    for rep_code in first_rep..last_rep {
        let rep_offset = if rep_code == 3 {
            rep[0].saturating_sub(1)
        } else {
            rep[rep_code]
        } as usize;
        let Some(rep_len) =
            bounds.rep_match_length(src, ip, rep_offset, min_match, block_end, window_low)
        else {
            continue;
        };
        if rep_len > *best_length {
            *best_length = rep_len;
            matches.push(OptMatch {
                off_base: repcode_to_off_base(rep_code - first_rep + 1),
                len: rep_len as u32,
            });
            if (rep_len > sufficient_len) | (ip + rep_len == block_end) {
                break;
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
#[inline(always)]
pub(in crate::encoding::levels::c_port) fn collect_repcode_matches_no_dict(
    matches: &mut OptMatchTable,
    src: &[u8],
    ip: usize,
    block_end: usize,
    offsets: [u32; 3],
    ll0: bool,
    min_match: u32,
    window_low: usize,
    sufficient_len: usize,
    best_length: &mut usize,
) {
    let max_rep_distance = ip - window_low;
    if ll0 {
        if try_repcode_match_no_dict(
            matches,
            src,
            ip,
            block_end,
            offsets[1],
            1,
            min_match,
            max_rep_distance,
            sufficient_len,
            best_length,
        ) {
            return;
        }
        if try_repcode_match_no_dict(
            matches,
            src,
            ip,
            block_end,
            offsets[2],
            2,
            min_match,
            max_rep_distance,
            sufficient_len,
            best_length,
        ) {
            return;
        }
        let _ = try_repcode_match_no_dict(
            matches,
            src,
            ip,
            block_end,
            offsets[0].wrapping_sub(1),
            3,
            min_match,
            max_rep_distance,
            sufficient_len,
            best_length,
        );
    } else {
        if try_repcode_match_no_dict(
            matches,
            src,
            ip,
            block_end,
            offsets[0],
            1,
            min_match,
            max_rep_distance,
            sufficient_len,
            best_length,
        ) {
            return;
        }
        if try_repcode_match_no_dict(
            matches,
            src,
            ip,
            block_end,
            offsets[1],
            2,
            min_match,
            max_rep_distance,
            sufficient_len,
            best_length,
        ) {
            return;
        }
        let _ = try_repcode_match_no_dict(
            matches,
            src,
            ip,
            block_end,
            offsets[2],
            3,
            min_match,
            max_rep_distance,
            sufficient_len,
            best_length,
        );
    }
}

pub(in crate::encoding::levels::c_port) fn should_stop_after_best_match(
    matches: &OptMatchTable,
    ip: usize,
    block_end: usize,
    sufficient_len: usize,
) -> bool {
    if matches.is_empty() {
        return false;
    }
    let best = matches.get(matches.len() - 1);
    (best.len as usize > sufficient_len) | (ip + best.len as usize == block_end)
}

#[allow(clippy::too_many_arguments)]
#[inline(always)]
fn try_repcode_match_no_dict(
    matches: &mut OptMatchTable,
    src: &[u8],
    ip: usize,
    block_end: usize,
    rep_offset: u32,
    repcode: usize,
    min_match: u32,
    max_rep_distance: usize,
    sufficient_len: usize,
    best_length: &mut usize,
) -> bool {
    let rep_offset = rep_offset as usize;
    if rep_offset.wrapping_sub(1) >= max_rep_distance {
        return false;
    }

    let rep_index = ip - rep_offset;
    if !equal_min_match(src, ip, rep_index, min_match) {
        return false;
    }

    let rep_len = count_match_no_dict(
        src,
        ip + min_match as usize,
        rep_index + min_match as usize,
        block_end,
    ) + min_match as usize;
    if rep_len > *best_length {
        *best_length = rep_len;
        matches.push(OptMatch {
            off_base: repcode_to_off_base(repcode),
            len: rep_len as u32,
        });
        return (rep_len > sufficient_len) | (ip + rep_len == block_end);
    }
    false
}

#[inline(always)]
fn repcode_to_off_base(code: usize) -> u32 {
    match code {
        1 => OffBase::Repeat(RepeatCode::First),
        2 => OffBase::Repeat(RepeatCode::Second),
        3 => OffBase::Repeat(RepeatCode::Third),
        _ => unreachable!("C repcode value is between 1 and 3"),
    }
    .to_c_value()
}
