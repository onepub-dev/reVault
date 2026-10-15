//! Optimal-parser price model ported from the no-dictionary path in
//! `zstd_opt.c`.

use super::hash_chain_match::highbit32;
use crate::encoding::blocks::{literal_length_code, match_length_code};

mod weights;

use weights::{
    downscale_stats, literal_length_code_transition, price_delta, scale_stats, sum, weight,
};

const BITCOST_ACCURACY: u32 = 8;
pub(super) const BITCOST_MULTIPLIER: u32 = 1 << BITCOST_ACCURACY;
pub(super) const ZSTD_MAX_PRICE: i32 = 1 << 30;
const ZSTD_BLOCKSIZE_MAX: u32 = 128 * 1024;
const ZSTD_PREDEF_THRESHOLD: usize = 8;
const ZSTD_LITFREQ_ADD: u32 = 2;
const MINMATCH: u32 = 3;

const MAX_LL: usize = 35;
const MAX_ML: usize = 52;
const MAX_OFF: usize = 31;
const MAX_LIT: usize = 255;

const LL_BITS: [u8; MAX_LL + 1] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 3, 3, 4, 6, 7, 8, 9, 10, 11,
    12, 13, 14, 15, 16,
];

const ML_BITS: [u8; MAX_ML + 1] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    1, 1, 1, 1, 2, 2, 3, 3, 4, 4, 5, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16,
];

const BASE_LL_FREQS: [u32; MAX_LL + 1] = [
    4, 2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 1, 1, 1,
];

const BASE_OFF_FREQS: [u32; MAX_OFF + 1] = [
    6, 2, 1, 1, 2, 3, 4, 4, 4, 3, 2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum OptLevel {
    BtOpt,
    BtUltra,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum PriceType {
    Predefined,
    Dynamic,
}

#[derive(Clone, Debug)]
pub(crate) struct DictionaryPriceSeeds {
    lit_freq: [u32; MAX_LIT + 1],
    lit_length_freq: [u32; MAX_LL + 1],
    match_length_freq: [u32; MAX_ML + 1],
    off_code_freq: [u32; MAX_OFF + 1],
}

impl DictionaryPriceSeeds {
    pub(crate) const LITERAL_COUNT: usize = MAX_LIT + 1;
    pub(crate) const LIT_LENGTH_COUNT: usize = MAX_LL + 1;
    pub(crate) const MATCH_LENGTH_COUNT: usize = MAX_ML + 1;
    pub(crate) const OFF_CODE_COUNT: usize = MAX_OFF + 1;

    pub(crate) fn new() -> Self {
        Self {
            lit_freq: [0; MAX_LIT + 1],
            lit_length_freq: [0; MAX_LL + 1],
            match_length_freq: [0; MAX_ML + 1],
            off_code_freq: [0; MAX_OFF + 1],
        }
    }

    pub(crate) fn set_literal_freq(&mut self, symbol: usize, freq: u32) {
        self.lit_freq[symbol] = freq;
    }

    pub(crate) fn set_lit_length_freq(&mut self, symbol: usize, freq: u32) {
        self.lit_length_freq[symbol] = freq;
    }

    pub(crate) fn set_match_length_freq(&mut self, symbol: usize, freq: u32) {
        self.match_length_freq[symbol] = freq;
    }

    pub(crate) fn set_off_code_freq(&mut self, symbol: usize, freq: u32) {
        self.off_code_freq[symbol] = freq;
    }
}

#[derive(Clone, Debug)]
pub(super) struct OptPriceState {
    lit_freq: [u32; MAX_LIT + 1],
    lit_length_freq: [u32; MAX_LL + 1],
    match_length_freq: [u32; MAX_ML + 1],
    off_code_freq: [u32; MAX_OFF + 1],
    lit_sum: u32,
    lit_length_sum: u32,
    match_length_sum: u32,
    off_code_sum: u32,
    lit_sum_base_price: u32,
    lit_price_max: u32,
    lit_length_sum_base_price: u32,
    match_length_sum_base_price: u32,
    off_code_sum_base_price: u32,
    price_type: PriceType,
    compressed_literals: bool,
    dictionary_seeds: Option<DictionaryPriceSeeds>,
}

impl OptPriceState {
    pub(super) fn new() -> Self {
        Self {
            lit_freq: [0; MAX_LIT + 1],
            lit_length_freq: [0; MAX_LL + 1],
            match_length_freq: [0; MAX_ML + 1],
            off_code_freq: [0; MAX_OFF + 1],
            lit_sum: 0,
            lit_length_sum: 0,
            match_length_sum: 0,
            off_code_sum: 0,
            lit_sum_base_price: 0,
            lit_price_max: 0,
            lit_length_sum_base_price: 0,
            match_length_sum_base_price: 0,
            off_code_sum_base_price: 0,
            price_type: PriceType::Dynamic,
            compressed_literals: true,
            dictionary_seeds: None,
        }
    }

    pub(super) fn set_dictionary_seeds(&mut self, seeds: DictionaryPriceSeeds) {
        self.dictionary_seeds = Some(seeds);
    }

    pub(super) fn reset_for_frame(&mut self) {
        self.lit_sum = 0;
        self.lit_length_sum = 0;
        self.match_length_sum = 0;
        self.off_code_sum = 0;
        self.lit_sum_base_price = 0;
        self.lit_price_max = 0;
        self.lit_length_sum_base_price = 0;
        self.match_length_sum_base_price = 0;
        self.off_code_sum_base_price = 0;
        self.price_type = PriceType::Dynamic;
        self.compressed_literals = true;
    }

    pub(super) fn rescale_freqs(&mut self, src: &[u8], opt_level: OptLevel) {
        self.price_type = PriceType::Dynamic;

        if self.lit_length_sum == 0 {
            if src.len() <= ZSTD_PREDEF_THRESHOLD {
                self.price_type = PriceType::Predefined;
            }

            if let Some(seeds) = self.dictionary_seeds.take() {
                self.price_type = PriceType::Dynamic;
                self.apply_dictionary_seeds(&seeds);
            } else {
                if self.compressed_literals {
                    self.lit_freq = [0; MAX_LIT + 1];
                    for &literal in src {
                        self.lit_freq[literal as usize] += 1;
                    }
                    self.lit_sum = downscale_stats(&mut self.lit_freq, 8, false);
                }

                self.lit_length_freq = BASE_LL_FREQS;
                self.lit_length_sum = sum(&self.lit_length_freq);
                self.match_length_freq = [1; MAX_ML + 1];
                self.match_length_sum = (MAX_ML + 1) as u32;
                self.off_code_freq = BASE_OFF_FREQS;
                self.off_code_sum = sum(&self.off_code_freq);
            }
        } else {
            if self.compressed_literals {
                self.lit_sum = scale_stats(&mut self.lit_freq, 12);
            }
            self.lit_length_sum = scale_stats(&mut self.lit_length_freq, 11);
            self.match_length_sum = scale_stats(&mut self.match_length_freq, 11);
            self.off_code_sum = scale_stats(&mut self.off_code_freq, 11);
        }

        self.set_base_prices(opt_level);
    }

    fn apply_dictionary_seeds(&mut self, seeds: &DictionaryPriceSeeds) {
        if self.compressed_literals {
            self.lit_freq = seeds.lit_freq;
            self.lit_sum = sum(&self.lit_freq);
        }
        self.lit_length_freq = seeds.lit_length_freq;
        self.lit_length_sum = sum(&self.lit_length_freq);
        self.match_length_freq = seeds.match_length_freq;
        self.match_length_sum = sum(&self.match_length_freq);
        self.off_code_freq = seeds.off_code_freq;
        self.off_code_sum = sum(&self.off_code_freq);
    }

    pub(super) fn raw_literals_cost(&self, literals: &[u8], opt_level: OptLevel) -> u32 {
        if literals.is_empty() {
            return 0;
        }

        if !self.compressed_literals {
            return ((literals.len() as u32) << 3) * BITCOST_MULTIPLIER;
        }

        if self.price_type == PriceType::Predefined {
            return (literals.len() as u32 * 6) * BITCOST_MULTIPLIER;
        }

        let mut price = self.lit_sum_base_price * literals.len() as u32;
        for &literal in literals {
            let lit_price =
                weight(self.lit_freq[literal as usize], opt_level).min(self.lit_price_max);
            price -= lit_price;
        }
        price
    }

    #[inline(always)]
    pub(super) fn raw_literal_cost(&self, literal: u8, opt_level: OptLevel) -> u32 {
        if !self.compressed_literals {
            return 8 * BITCOST_MULTIPLIER;
        }

        if self.price_type == PriceType::Predefined {
            return 6 * BITCOST_MULTIPLIER;
        }

        self.lit_sum_base_price
            - weight(self.lit_freq[literal as usize], opt_level).min(self.lit_price_max)
    }

    #[inline(always)]
    pub(super) fn dynamic_raw_literal_cost(&self, literal: u8, opt_level: OptLevel) -> u32 {
        debug_assert!(self.compressed_literals);
        debug_assert_eq!(self.price_type, PriceType::Dynamic);

        self.lit_sum_base_price
            - weight(self.lit_freq[literal as usize], opt_level).min(self.lit_price_max)
    }

    #[inline(always)]
    pub(super) fn lit_length_price(&self, lit_length: u32, opt_level: OptLevel) -> u32 {
        debug_assert!(lit_length <= ZSTD_BLOCKSIZE_MAX);
        if self.price_type == PriceType::Predefined {
            return weight(lit_length, opt_level);
        }

        self.dynamic_lit_length_price(lit_length, opt_level)
    }

    #[inline(always)]
    pub(super) fn dynamic_lit_length_price(&self, lit_length: u32, opt_level: OptLevel) -> u32 {
        debug_assert!(lit_length <= ZSTD_BLOCKSIZE_MAX);
        debug_assert_eq!(self.price_type, PriceType::Dynamic);

        if lit_length == ZSTD_BLOCKSIZE_MAX {
            return BITCOST_MULTIPLIER
                + self.dynamic_lit_length_price(ZSTD_BLOCKSIZE_MAX - 1, opt_level);
        }

        let ll_code = literal_length_code(lit_length) as usize;
        u32::from(LL_BITS[ll_code]) * BITCOST_MULTIPLIER + self.lit_length_sum_base_price
            - weight(self.lit_length_freq[ll_code], opt_level)
    }

    #[inline(always)]
    pub(super) fn lit_length_increment_price(&self, lit_length: u32, opt_level: OptLevel) -> i32 {
        debug_assert!(lit_length > 0);
        debug_assert!(lit_length <= ZSTD_BLOCKSIZE_MAX);

        if self.price_type == PriceType::Predefined {
            return price_delta(
                weight(lit_length, opt_level),
                weight(lit_length - 1, opt_level),
            );
        }

        self.dynamic_lit_length_increment_price(lit_length, opt_level)
    }

    #[inline(always)]
    pub(super) fn dynamic_lit_length_increment_price(
        &self,
        lit_length: u32,
        opt_level: OptLevel,
    ) -> i32 {
        debug_assert!(lit_length > 0);
        debug_assert!(lit_length <= ZSTD_BLOCKSIZE_MAX);
        debug_assert_eq!(self.price_type, PriceType::Dynamic);

        self.dynamic_lit_length_increment_price_unchecked(lit_length, opt_level)
    }

    #[inline(always)]
    fn dynamic_lit_length_increment_price_unchecked(
        &self,
        lit_length: u32,
        opt_level: OptLevel,
    ) -> i32 {
        if lit_length == ZSTD_BLOCKSIZE_MAX {
            return BITCOST_MULTIPLIER as i32;
        }

        if lit_length <= 15 {
            let ll_code = lit_length as usize;
            return price_delta(
                weight(self.lit_length_freq[ll_code - 1], opt_level),
                weight(self.lit_length_freq[ll_code], opt_level),
            );
        }

        if lit_length == 16 {
            return self.dynamic_lit_length_code_delta(15, 16, opt_level);
        }

        let Some((previous_code, ll_code)) = literal_length_code_transition(lit_length) else {
            return 0;
        };

        self.dynamic_lit_length_code_delta(previous_code, ll_code, opt_level)
    }

    #[inline(always)]
    fn dynamic_lit_length_code_delta(
        &self,
        previous_code: usize,
        ll_code: usize,
        opt_level: OptLevel,
    ) -> i32 {
        let price = u32::from(LL_BITS[ll_code]) * BITCOST_MULTIPLIER
            + self.lit_length_sum_base_price
            - weight(self.lit_length_freq[ll_code], opt_level);
        let previous_price = u32::from(LL_BITS[previous_code]) * BITCOST_MULTIPLIER
            + self.lit_length_sum_base_price
            - weight(self.lit_length_freq[previous_code], opt_level);
        price_delta(price, previous_price)
    }

    #[inline(always)]
    pub(super) fn match_price(&self, off_base: u32, match_length: u32, opt_level: OptLevel) -> u32 {
        self.match_offset_price(off_base, opt_level)
            + self.match_length_price(match_length, opt_level)
    }

    #[inline(always)]
    pub(super) fn match_offset_price(&self, off_base: u32, opt_level: OptLevel) -> u32 {
        let off_code = highbit32(off_base);

        if self.price_type == PriceType::Predefined {
            return (16 + off_code) * BITCOST_MULTIPLIER;
        }

        self.dynamic_match_offset_price_from_code(off_code, opt_level)
    }

    #[inline(always)]
    pub(super) fn dynamic_match_offset_price(&self, off_base: u32, opt_level: OptLevel) -> u32 {
        debug_assert_eq!(self.price_type, PriceType::Dynamic);
        let off_code = highbit32(off_base);
        self.dynamic_match_offset_price_from_code(off_code, opt_level)
    }

    #[inline(always)]
    fn dynamic_match_offset_price_from_code(&self, off_code: u32, opt_level: OptLevel) -> u32 {
        let mut price = off_code * BITCOST_MULTIPLIER + self.off_code_sum_base_price
            - weight(self.off_code_freq[off_code as usize], opt_level);
        if opt_level.favors_small_offsets() && off_code >= 20 {
            price += (off_code - 19) * 2 * BITCOST_MULTIPLIER;
        }

        price
    }

    #[inline(always)]
    pub(super) fn match_length_price(&self, match_length: u32, opt_level: OptLevel) -> u32 {
        debug_assert!(match_length >= MINMATCH);
        let ml_base = match_length - MINMATCH;

        if self.price_type == PriceType::Predefined {
            return weight(ml_base, opt_level);
        }

        self.dynamic_match_length_price_from_base(ml_base, opt_level)
    }

    #[inline(always)]
    pub(super) fn dynamic_match_length_price(&self, match_length: u32, opt_level: OptLevel) -> u32 {
        debug_assert!(match_length >= MINMATCH);
        debug_assert_eq!(self.price_type, PriceType::Dynamic);
        let ml_base = match_length - MINMATCH;
        self.dynamic_match_length_price_from_base(ml_base, opt_level)
    }

    #[inline(always)]
    fn dynamic_match_length_price_from_base(&self, ml_base: u32, opt_level: OptLevel) -> u32 {
        let match_length = ml_base + MINMATCH;
        let ml_code = match_length_code(match_length) as usize;
        u32::from(ML_BITS[ml_code]) * BITCOST_MULTIPLIER + self.match_length_sum_base_price
            - weight(self.match_length_freq[ml_code], opt_level)
            + BITCOST_MULTIPLIER / 5
    }

    pub(super) fn update_stats(
        &mut self,
        lit_length: u32,
        literals: &[u8],
        off_base: u32,
        match_length: u32,
    ) {
        debug_assert!(literals.len() >= lit_length as usize);

        if self.compressed_literals && lit_length != 0 {
            for &literal in &literals[..lit_length as usize] {
                self.lit_freq[literal as usize] += ZSTD_LITFREQ_ADD;
            }
            self.lit_sum += lit_length * ZSTD_LITFREQ_ADD;
        }

        let ll_code = literal_length_code(lit_length) as usize;
        self.lit_length_freq[ll_code] += 1;
        self.lit_length_sum += 1;

        let off_code = highbit32(off_base) as usize;
        self.off_code_freq[off_code] += 1;
        self.off_code_sum += 1;

        let ml_code = match_length_code(match_length) as usize;
        self.match_length_freq[ml_code] += 1;
        self.match_length_sum += 1;
    }

    #[inline(always)]
    pub(super) fn refresh_base_prices(&mut self, opt_level: OptLevel) {
        self.set_base_prices(opt_level);
    }

    #[cfg(test)]
    pub(super) fn frequency_snapshot(
        &self,
        ll_code: usize,
        ml_code: usize,
        off_code: usize,
    ) -> (u32, u32, u32) {
        (
            self.lit_length_freq[ll_code],
            self.match_length_freq[ml_code],
            self.off_code_freq[off_code],
        )
    }

    #[inline(always)]
    fn set_base_prices(&mut self, opt_level: OptLevel) {
        if self.compressed_literals {
            self.lit_sum_base_price = weight(self.lit_sum, opt_level);
            self.lit_price_max = self.lit_sum_base_price - BITCOST_MULTIPLIER;
        }
        self.lit_length_sum_base_price = weight(self.lit_length_sum, opt_level);
        self.match_length_sum_base_price = weight(self.match_length_sum, opt_level);
        self.off_code_sum_base_price = weight(self.off_code_sum, opt_level);
    }
}
