//! Optimal-parser path reconstruction helpers ported from `zstd_opt.c`.

use alloc::vec::Vec;

use super::{
    opt_state::{OptBlockState, Optimal},
    sequence_store::{OffBase, RepeatOffsets},
};

pub(super) fn select_path(
    last_pos: usize,
    last_stretch: Option<Optimal>,
    rep: &mut [u32; 3],
    state: &OptBlockState,
    path: &mut Vec<Optimal>,
) {
    path.clear();
    let stretch = last_stretch.unwrap_or_else(|| state.opt[last_pos]);
    let mut cur = last_pos - stretch.mlen as usize;

    if stretch.litlen == 0 {
        *rep = update_reps(state.opt[cur].rep, stretch.off, state.opt[cur].litlen == 0);
    } else {
        *rep = stretch.rep;
        cur -= stretch.litlen as usize;
    }

    path.push(stretch);
    let mut stretch_pos = cur;
    loop {
        let next = state.opt[stretch_pos];
        let last = path.len() - 1;
        path[last].litlen = next.litlen;
        if next.mlen == 0 {
            break;
        }
        path.push(next);
        stretch_pos -= next.litlen as usize + next.mlen as usize;
    }

    path.reverse();
}

pub(super) fn update_reps(rep: [u32; 3], off_base: u32, previous_litlen_zero: bool) -> [u32; 3] {
    let mut repeat_offsets = RepeatOffsets::from_offsets(rep[0], rep[1], rep[2]);
    repeat_offsets.update(
        OffBase::from_c_value(off_base).expect("optimal parser rep offBase"),
        u32::from(!previous_litlen_zero),
    );
    repeat_offsets.as_offsets()
}
