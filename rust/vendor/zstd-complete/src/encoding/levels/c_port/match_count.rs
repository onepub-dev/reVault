//! Match-length counter shared by the C-port match finders.

#![forbid(unsafe_code)]

use super::unaligned::{read16, read32, read64};

pub(super) fn count_match(
    src: &[u8],
    mut pos: usize,
    mut match_pos: usize,
    match_limit: usize,
) -> usize {
    let start = pos;

    while pos + 8 <= match_limit && match_pos + 8 <= src.len() {
        let diff = read64(src, pos) ^ read64(src, match_pos);
        if diff != 0 {
            return pos - start + common_prefix_bytes(diff);
        }
        pos += 8;
        match_pos += 8;
    }

    if pos + 4 <= match_limit
        && match_pos + 4 <= src.len()
        && read32(src, pos) == read32(src, match_pos)
    {
        pos += 4;
        match_pos += 4;
    }

    if pos + 2 <= match_limit
        && match_pos + 2 <= src.len()
        && read16(src, pos) == read16(src, match_pos)
    {
        pos += 2;
        match_pos += 2;
    }

    if pos < match_limit && match_pos < src.len() && src[pos] == src[match_pos] {
        pos += 1;
    }

    pos - start
}

#[inline(always)]
pub(super) fn count_match_no_dict(
    src: &[u8],
    mut pos: usize,
    mut match_pos: usize,
    match_limit: usize,
) -> usize {
    debug_assert!(match_pos <= pos);
    debug_assert!(match_limit <= src.len());

    let start = pos;
    let loop_limit = match_limit.saturating_sub(7);

    if pos < loop_limit {
        let diff = read64(src, pos) ^ read64(src, match_pos);
        if diff != 0 {
            return common_prefix_bytes(diff);
        }
        pos += 8;
        match_pos += 8;

        while pos < loop_limit {
            let diff = read64(src, pos) ^ read64(src, match_pos);
            if diff != 0 {
                return pos - start + common_prefix_bytes(diff);
            }
            pos += 8;
            match_pos += 8;
        }
    }

    if pos + 4 <= match_limit && read32(src, pos) == read32(src, match_pos) {
        pos += 4;
        match_pos += 4;
    }

    if pos + 2 <= match_limit && read16(src, pos) == read16(src, match_pos) {
        pos += 2;
        match_pos += 2;
    }

    if pos < match_limit && src[pos] == src[match_pos] {
        pos += 1;
    }

    pos - start
}

#[inline(always)]
pub(super) fn count_match_behind(
    src: &[u8],
    mut pos: usize,
    mut match_pos: usize,
    match_limit: usize,
) -> usize {
    debug_assert!(match_pos <= pos);
    debug_assert!(match_limit <= src.len());

    let start = pos;

    while pos + 8 <= match_limit {
        let diff = read64(src, pos) ^ read64(src, match_pos);
        if diff != 0 {
            return pos - start + common_prefix_bytes(diff);
        }
        pos += 8;
        match_pos += 8;
    }

    if pos + 4 <= match_limit && read32(src, pos) == read32(src, match_pos) {
        pos += 4;
        match_pos += 4;
    }

    if pos + 2 <= match_limit && read16(src, pos) == read16(src, match_pos) {
        pos += 2;
        match_pos += 2;
    }

    if pos < match_limit && src[pos] == src[match_pos] {
        pos += 1;
    }

    pos - start
}

#[inline(always)]
fn common_prefix_bytes(diff: u64) -> usize {
    (diff.trailing_zeros() >> 3) as usize
}

#[cfg(test)]
mod tests {
    use super::{count_match, count_match_behind, count_match_no_dict};

    #[test]
    fn counts_full_match_to_limit() {
        let data = b"abcdefghabcdefgh_tail";
        assert_eq!(count_match(data, 0, 8, 8), 8);
    }

    #[test]
    fn stops_inside_first_word() {
        let data = b"abcxefghabcdzzzz";
        assert_eq!(count_match(data, 0, 8, 8), 3);
    }

    #[test]
    fn no_dict_counter_matches_general_counter() {
        let data = b"0123456789abcdef0123456789abcx";
        assert_eq!(
            count_match_no_dict(data, 16, 0, data.len()),
            count_match(data, 16, 0, data.len())
        );
    }

    #[test]
    fn counts_tail_after_word_chunks() {
        let data = b"abcdefghijkXabcdefghijkY";
        assert_eq!(count_match(data, 0, 12, 12), 11);
    }

    #[test]
    fn respects_match_limit() {
        let data = b"abcdefghijklmnop";
        assert_eq!(count_match(data, 0, 0, 5), 5);
    }

    #[test]
    fn behind_counter_counts_full_match_to_limit() {
        let data = b"abcdefghabcdefgh_tail";
        assert_eq!(count_match_behind(data, 8, 0, 16), 8);
    }
}
