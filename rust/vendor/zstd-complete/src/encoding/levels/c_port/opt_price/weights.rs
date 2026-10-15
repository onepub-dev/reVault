use crate::encoding::levels::c_port::hash_chain_match::highbit32;

use super::{OptLevel, BITCOST_ACCURACY, BITCOST_MULTIPLIER};

impl OptLevel {
    #[inline(always)]
    fn accurate_weights(self) -> bool {
        matches!(self, Self::BtUltra)
    }

    #[inline(always)]
    pub(super) fn favors_small_offsets(self) -> bool {
        matches!(self, Self::BtOpt)
    }
}

#[inline(always)]
pub(super) fn weight(stat: u32, opt_level: OptLevel) -> u32 {
    if opt_level.accurate_weights() {
        frac_weight(stat)
    } else {
        bit_weight(stat)
    }
}

#[inline(always)]
fn bit_weight(stat: u32) -> u32 {
    highbit32(stat + 1) * BITCOST_MULTIPLIER
}

#[inline(always)]
fn frac_weight(raw_stat: u32) -> u32 {
    let stat = raw_stat + 1;
    let high_bit = highbit32(stat);
    let base_weight = high_bit * BITCOST_MULTIPLIER;
    let frac_weight = (stat << BITCOST_ACCURACY) >> high_bit;
    base_weight + frac_weight
}

#[inline(always)]
pub(super) fn price_delta(price: u32, previous_price: u32) -> i32 {
    (i64::from(price) - i64::from(previous_price)) as i32
}

#[inline(always)]
pub(super) fn literal_length_code_transition(lit_length: u32) -> Option<(usize, usize)> {
    match lit_length {
        1..=16 => Some(((lit_length - 1) as usize, lit_length as usize)),
        18 => Some((16, 17)),
        20 => Some((17, 18)),
        22 => Some((18, 19)),
        24 => Some((19, 20)),
        28 => Some((20, 21)),
        32 => Some((21, 22)),
        40 => Some((22, 23)),
        48 => Some((23, 24)),
        64 => Some((24, 25)),
        128 => Some((25, 26)),
        256 => Some((26, 27)),
        512 => Some((27, 28)),
        1024 => Some((28, 29)),
        2048 => Some((29, 30)),
        4096 => Some((30, 31)),
        8192 => Some((31, 32)),
        16384 => Some((32, 33)),
        32768 => Some((33, 34)),
        65536 => Some((34, 35)),
        _ => None,
    }
}

pub(super) fn scale_stats<const N: usize>(table: &mut [u32; N], log_target: u32) -> u32 {
    let previous_sum = sum(table);
    let factor = previous_sum >> log_target;
    if factor <= 1 {
        return previous_sum;
    }
    downscale_stats(table, highbit32(factor), true)
}

pub(super) fn downscale_stats<const N: usize>(
    table: &mut [u32; N],
    shift: u32,
    base_one: bool,
) -> u32 {
    let mut sum = 0;
    for stat in table {
        let base = u32::from(base_one || *stat > 0);
        let new_stat = base + (*stat >> shift);
        *stat = new_stat;
        sum += new_stat;
    }
    sum
}

pub(super) fn sum(table: &[u32]) -> u32 {
    table.iter().sum()
}
