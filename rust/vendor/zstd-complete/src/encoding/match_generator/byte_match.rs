use super::*;

impl MatchGenerator {
    pub(super) fn extend_match_backwards(
        &self,
        offset: usize,
        match_len: usize,
        context: &MatchCandidateContext<'_>,
    ) -> (usize, usize) {
        let mut start_idx = context.suffix_idx;
        let mut match_len = match_len;
        while start_idx > context.anchor_idx {
            let target_idx = start_idx - 1;
            let source_relative = target_idx as isize - offset as isize;
            let Some(source) = self
                .slice_at_relative(source_relative)
                .and_then(|source| source.first())
            else {
                break;
            };
            if *source != self.last_entry().data[target_idx] {
                break;
            }

            start_idx = target_idx;
            match_len += 1;
        }

        (start_idx, match_len)
    }

    #[cfg(test)]
    pub(super) fn match_len_at_offset(
        &self,
        offset: usize,
        context: &MatchCandidateContext<'_>,
    ) -> usize {
        if offset == 0 {
            return 0;
        }

        let mut len = 0usize;
        while len < context.data_slice.len() {
            let source_relative = context.suffix_idx as isize + len as isize - offset as isize;
            let Some(source) = self.slice_at_relative(source_relative) else {
                break;
            };

            let target = &context.data_slice[len..];
            let matched = Self::common_prefix_len(source, target);
            len += matched;
            if matched < source.len().min(target.len()) {
                break;
            }
        }
        len
    }

    #[cfg(test)]
    pub(super) fn has_min_match_at_offset(
        &self,
        offset: usize,
        context: &MatchCandidateContext<'_>,
    ) -> bool {
        self.verified_min_match_prefix_len(offset, context)
            .is_some()
    }

    #[inline(always)]
    pub(super) fn verified_min_match_prefix_len(
        &self,
        offset: usize,
        context: &MatchCandidateContext<'_>,
    ) -> Option<usize> {
        if offset == 0 {
            return None;
        }

        let source_relative = context.suffix_idx as isize - offset as isize;
        let source = self.slice_at_relative(source_relative)?;

        if source.len() < MIN_MATCH_LEN {
            return Some(0);
        }

        if source[..MIN_MATCH_LEN] == context.data_slice[..MIN_MATCH_LEN] {
            Some(MIN_MATCH_LEN)
        } else {
            None
        }
    }

    #[inline(always)]
    pub(super) fn match_len_at_offset_with_prefix(
        &self,
        offset: usize,
        context: &MatchCandidateContext<'_>,
        verified_prefix_len: usize,
    ) -> usize {
        if offset == 0 {
            return 0;
        }

        if offset <= context.suffix_idx {
            let source_start = context.suffix_idx - offset + verified_prefix_len;
            let source = &self.last_entry().data[source_start..];
            let target = &context.data_slice[verified_prefix_len..];
            return verified_prefix_len + Self::common_prefix_len(source, target);
        }

        let mut len = verified_prefix_len;
        while len < context.data_slice.len() {
            let source_relative = context.suffix_idx as isize + len as isize - offset as isize;
            let Some(source) = self.slice_at_relative(source_relative) else {
                break;
            };
            let target = &context.data_slice[len..];
            let matched = Self::common_prefix_len(source, target);
            len += matched;
            if matched < source.len().min(target.len()) {
                break;
            }
        }
        len
    }

    #[inline(always)]
    pub(super) fn slice_at_relative(&self, relative_to_current: isize) -> Option<&[u8]> {
        if relative_to_current >= 0 {
            return self.last_entry().data.get(relative_to_current as usize..);
        }

        let previous_entries = self.last_entry_index();
        for entry in self.window[..previous_entries].iter().rev() {
            let start = -(entry.base_offset as isize);
            let end = start + entry.data.len() as isize;
            if (start..end).contains(&relative_to_current) {
                return Some(&entry.data[(relative_to_current - start) as usize..]);
            }
        }

        None
    }

    /// Find the common prefix length between two byte slices.
    #[inline(always)]
    pub(super) fn common_prefix_len(a: &[u8], b: &[u8]) -> usize {
        Self::mismatch_chunks::<8>(a, b)
    }

    /// Find the common prefix length between two byte slices with a configurable chunk length.
    /// The chunked shape is easy for the optimizer to vectorize while staying in safe Rust.
    pub(super) fn mismatch_chunks<const N: usize>(xs: &[u8], ys: &[u8]) -> usize {
        let off = core::iter::zip(xs.chunks_exact(N), ys.chunks_exact(N))
            .take_while(|(x, y)| x == y)
            .count()
            * N;
        off + core::iter::zip(&xs[off..], &ys[off..])
            .take_while(|(x, y)| x == y)
            .count()
    }

    #[inline(always)]
    pub(super) fn local_offset_code(offset_value: u32) -> u8 {
        offset_value.ilog2() as u8
    }

    #[inline(always)]
    pub(super) fn local_literal_length_extra_bits(len: u32) -> usize {
        match len {
            0..=15 => 0,
            16..=19 => 1,
            20..=23 => 2,
            24..=31 => 3,
            32..=47 => 4,
            48..=63 => 5,
            64..=127 => 6,
            128..=255 => 7,
            256..=511 => 8,
            512..=1023 => 9,
            1024..=2047 => 10,
            2048..=4095 => 11,
            4096..=8191 => 12,
            8192..=16383 => 13,
            16384..=32767 => 14,
            32768..=65535 => 15,
            _ => 16,
        }
    }

    #[inline(always)]
    pub(super) fn local_match_length_extra_bits(len: u32) -> usize {
        match len {
            3..=34 => 0,
            35..=50 => 1,
            51..=66 => 2,
            67..=82 => 4,
            83..=98 => 4,
            99..=130 => 5,
            131..=258 => 7,
            259..=514 => 8,
            515..=1026 => 9,
            1027..=2050 => 10,
            2051..=4098 => 11,
            4099..=8194 => 12,
            8195..=16386 => 13,
            16387..=32770 => 14,
            32771..=65538 => 15,
            _ => 16,
        }
    }
}
