use super::*;

impl MatchGenerator {
    pub(super) fn min_non_repeat_match_len_for_text(
        &self,
        data: &[u8],
        is_text_block: bool,
    ) -> usize {
        if is_text_block {
            if !self.use_text_repeat_pipeline && Self::likely_short_line_text(data) {
                if Self::likely_code_like_short_text(data) {
                    if matches!(self.file_type_hint, CompressionFileType::CodeText)
                        && data.len() <= SMALL_CODE_TEXT_MIN_NON_REPEAT_MAX_BLOCK_LEN
                    {
                        SMALL_TEXT_MIN_NON_REPEAT_MATCH_LEN
                    } else {
                        CODE_LIKE_SHORT_TEXT_MIN_NON_REPEAT_MATCH_LEN
                    }
                } else {
                    SMALL_TEXT_MIN_NON_REPEAT_MATCH_LEN
                }
            } else {
                TEXT_MIN_NON_REPEAT_MATCH_LEN
            }
        } else {
            MIN_MATCH_LEN
        }
    }

    pub(super) fn likely_text(data: &[u8]) -> bool {
        const SAMPLE_COUNT: usize = 256;
        const MIN_TEXT_BYTES: usize = 1024;

        if data.len() < MIN_TEXT_BYTES {
            return false;
        }

        let step = (data.len() / SAMPLE_COUNT).max(1);
        let mut printable = 0usize;
        let mut total = 0usize;
        for idx in (0..data.len()).step_by(step).take(SAMPLE_COUNT) {
            total += 1;
            let byte = data[idx];
            if byte == b'\n'
                || byte == b'\r'
                || byte == b'\t'
                || byte.is_ascii_graphic()
                || byte == b' '
            {
                printable += 1;
            }
        }

        printable * 100 >= total * 90
    }

    pub(super) fn likely_short_line_text(data: &[u8]) -> bool {
        let mut short_lines = 0usize;
        let mut nonempty_lines = 0usize;
        let mut current_len = 0usize;

        for &byte in data {
            if byte == b'\n' {
                if current_len != 0 {
                    nonempty_lines += 1;
                    if current_len <= SHORT_TEXT_LINE_LEN_LIMIT {
                        short_lines += 1;
                    }
                }
                current_len = 0;
            } else if byte != b'\r' {
                current_len += 1;
            }
        }

        if current_len != 0 {
            nonempty_lines += 1;
            if current_len <= SHORT_TEXT_LINE_LEN_LIMIT {
                short_lines += 1;
            }
        }

        nonempty_lines != 0
            && short_lines * 100 >= nonempty_lines * SHORT_TEXT_LINE_FRACTION_PERCENT
    }

    pub(super) fn likely_lockfile_text(data: &[u8]) -> bool {
        likely_lockfile_text(data)
    }

    pub(super) fn likely_composer_dictionary_text(data: &[u8]) -> bool {
        likely_composer_lockfile_text(data)
    }

    pub(super) fn likely_structured_json_config_text(data: &[u8]) -> bool {
        let Some(first_non_ws) = data
            .iter()
            .copied()
            .find(|byte| !matches!(byte, b' ' | b'\t' | b'\r' | b'\n'))
        else {
            return false;
        };
        if first_non_ws != b'{' {
            return false;
        }

        let mut keyed_lines = 0usize;
        let mut content_lines = 0usize;
        for line in data.split(|&byte| byte == b'\n').take(256) {
            let line = line
                .iter()
                .skip_while(|&&byte| matches!(byte, b' ' | b'\t'))
                .copied()
                .collect::<Vec<u8>>();
            let line = line.as_slice();
            if line.is_empty() {
                continue;
            }
            if matches!(line, b"{" | b"}" | b"}," | b"[" | b"]" | b"],") {
                continue;
            }
            content_lines += 1;
            if line.starts_with(b"\"") && line.contains(&b':') {
                keyed_lines += 1;
            }
        }

        content_lines >= 4 && keyed_lines * 100 >= content_lines * 60
    }

    pub(super) fn likely_tsconfig_json_config_text(data: &[u8]) -> bool {
        let Some(first_non_ws) = data
            .iter()
            .copied()
            .find(|byte| !matches!(byte, b' ' | b'\t' | b'\r' | b'\n'))
        else {
            return false;
        };
        if first_non_ws != b'{' {
            return false;
        }

        let sample = &data[..data.len().min(16 * 1024)];
        let mut compiler_options = false;
        let mut paths = false;
        let mut include_or_exclude = 0usize;
        let mut feature_aliases = 0usize;

        for line in sample.split(|&byte| byte == b'\n').take(512) {
            let line = line
                .iter()
                .skip_while(|&&byte| matches!(byte, b' ' | b'\t'))
                .copied()
                .collect::<Vec<u8>>();
            let line = line.as_slice();
            if line.is_empty() {
                continue;
            }

            if line.starts_with(br#""compilerOptions":"#)
                || line.starts_with(br#""compilerOptions": {"#)
            {
                compiler_options = true;
            } else if line.starts_with(br#""paths":"#) || line.starts_with(br#""paths": {"#) {
                paths = true;
            } else if line.starts_with(br#""include":"#)
                || line.starts_with(br#""include": ["#)
                || line.starts_with(br#""exclude":"#)
                || line.starts_with(br#""exclude": ["#)
                || line.starts_with(br#""references":"#)
                || line.starts_with(br#""references": ["#)
            {
                include_or_exclude += 1;
            } else if line.starts_with(br#""@feature/"#) {
                feature_aliases += 1;
            }
        }

        compiler_options && paths && (include_or_exclude >= 1 || feature_aliases >= 8)
    }

    pub(super) fn likely_code_like_short_text(data: &[u8]) -> bool {
        let mut nonempty_lines = 0usize;
        let mut semicolons = 0usize;
        let mut braces = 0usize;
        let mut current_len = 0usize;

        for &byte in data {
            match byte {
                b';' => {
                    semicolons += 1;
                    current_len += 1;
                }
                b'{' | b'}' => {
                    braces += 1;
                    current_len += 1;
                }
                b'\n' => {
                    if current_len != 0 {
                        nonempty_lines += 1;
                    }
                    current_len = 0;
                }
                b'\r' => {}
                _ => current_len += 1,
            }
        }

        if current_len != 0 {
            nonempty_lines += 1;
        }

        nonempty_lines != 0
            && (semicolons * 100 >= nonempty_lines * CODE_LIKE_SEMI_PER_100_LINES
                || braces * 100 >= nonempty_lines * CODE_LIKE_BRACES_PER_100_LINES)
    }

    pub(super) fn active_window_size_for_text_kind(&self, is_text_block: bool) -> usize {
        if is_text_block {
            self.max_window_size
        } else {
            self.fast_window_size
        }
    }
}
