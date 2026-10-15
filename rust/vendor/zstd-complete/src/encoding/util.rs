use alloc::vec::Vec;

/// Returns the minimum number of bytes needed to represent this value, as
/// either 1, 2, 4, or 8 bytes. A value of 0 will still return one byte.
///
/// Used for variable length fields like `Dictionary_ID` or `Frame_Content_Size`.
pub fn find_min_size(val: u64) -> usize {
    if val == 0 {
        return 1;
    }
    if val >> 8 == 0 {
        return 1;
    }
    if val >> 16 == 0 {
        return 2;
    }
    if val >> 32 == 0 {
        return 4;
    }
    8
}

/// Returns the same value, but represented using the smallest number of bytes needed.
/// Returned vector will be 1, 2, 4, or 8 bytes in length. Zero is represented as 1 byte.
///
/// Operates in **little-endian**.
pub fn minify_val(val: u64) -> impl Iterator<Item = u8> {
    let new_size = find_min_size(val);
    IntoIterator::into_iter(val.to_le_bytes()).take(new_size)
}

pub(crate) fn likely_incompressible(data: &[u8]) -> bool {
    const MIN_MATCH_LEN: usize = 5;
    const SAMPLE_COUNT: usize = 256;

    if data.len() < 8 * 1024 {
        return false;
    }

    let max_start = data.len() - MIN_MATCH_LEN;
    let samples = SAMPLE_COUNT.min(max_start + 1);
    let step = (max_start / samples).max(1);
    let mut keys = [0u64; SAMPLE_COUNT];
    for (used, sample) in (0..samples).enumerate() {
        let pos = (sample * step).min(max_start);
        let key = u64::from(data[pos])
            | (u64::from(data[pos + 1]) << 8)
            | (u64::from(data[pos + 2]) << 16)
            | (u64::from(data[pos + 3]) << 24)
            | (u64::from(data[pos + 4]) << 32);
        if keys[..used].contains(&key) {
            return false;
        }
        keys[used] = key;
    }

    true
}

pub(crate) fn likely_text(data: &[u8]) -> bool {
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

pub(crate) fn likely_lockfile_text(data: &[u8]) -> bool {
    const LOCKFILE_TEXT_MIN_LEN: usize = 16 * 1024;
    const LOCKFILE_TEXT_MIN_PACKAGE_MARKERS: usize = 4;

    if data.len() < LOCKFILE_TEXT_MIN_LEN {
        return false;
    }

    let mut package_markers = 0usize;
    let mut name_markers = 0usize;
    let mut version_markers = 0usize;
    let mut checksum_markers = 0usize;

    for line in data.split(|&byte| byte == b'\n') {
        if line.starts_with(b"[[package]]") {
            package_markers += 1;
        } else if line.starts_with(b"name = \"") {
            name_markers += 1;
        } else if line.starts_with(b"version = \"") {
            version_markers += 1;
        } else if line.starts_with(b"checksum = \"") {
            checksum_markers += 1;
        }

        if package_markers >= LOCKFILE_TEXT_MIN_PACKAGE_MARKERS
            && name_markers >= LOCKFILE_TEXT_MIN_PACKAGE_MARKERS
            && version_markers >= LOCKFILE_TEXT_MIN_PACKAGE_MARKERS
            && checksum_markers >= LOCKFILE_TEXT_MIN_PACKAGE_MARKERS
        {
            return true;
        }
    }

    false
}

pub(crate) fn likely_composer_lockfile_text(data: &[u8]) -> bool {
    const MIN_LEN: usize = 16 * 1024;
    const MIN_MARKERS: usize = 4;

    if data.len() < MIN_LEN {
        return false;
    }

    let mut packages_array = false;
    let mut name_markers = 0usize;
    let mut version_markers = 0usize;
    let mut require_markers = 0usize;
    let mut reference_markers = 0usize;

    for line in data.split(|&byte| byte == b'\n') {
        let trimmed = line
            .iter()
            .skip_while(|&&byte| matches!(byte, b' ' | b'\t'))
            .copied()
            .collect::<Vec<u8>>();
        let trimmed = trimmed.as_slice();

        if trimmed.starts_with(br#""packages": ["#) {
            packages_array = true;
        } else if trimmed.starts_with(br#""name": ""#) {
            name_markers += 1;
        } else if trimmed.starts_with(br#""version": ""#) {
            version_markers += 1;
        } else if trimmed.starts_with(br#""require": {"#) {
            require_markers += 1;
        } else if trimmed.starts_with(br#""reference": ""#) {
            reference_markers += 1;
        }

        if packages_array
            && name_markers >= MIN_MARKERS
            && version_markers >= MIN_MARKERS
            && require_markers >= MIN_MARKERS
            && reference_markers >= MIN_MARKERS
        {
            return true;
        }
    }

    false
}

pub(crate) fn likely_dependency_json_lockfile_text(data: &[u8]) -> bool {
    const MIN_LEN: usize = 8 * 1024;

    if data.len() < MIN_LEN || likely_composer_lockfile_text(data) {
        return false;
    }

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
    let mut lockfile_version = false;
    let mut meta_section = false;
    let mut packages = false;
    let mut dependencies = false;
    let mut default_section = false;
    let mut develop_section = false;

    for line in sample.split(|&byte| byte == b'\n').take(512) {
        let trimmed = line
            .iter()
            .skip_while(|&&byte| matches!(byte, b' ' | b'\t'))
            .copied()
            .collect::<Vec<u8>>();
        let trimmed = trimmed.as_slice();

        if trimmed.starts_with(br#""lockfileVersion":"#) {
            lockfile_version = true;
        } else if trimmed.starts_with(br#""_meta": {"#) || trimmed.starts_with(br#""_meta":{"#) {
            meta_section = true;
        } else if trimmed.starts_with(br#""packages": {"#)
            || trimmed.starts_with(br#""packages":{"#)
            || trimmed.starts_with(br#""packages": ["#)
        {
            packages = true;
        } else if trimmed.starts_with(br#""dependencies": {"#)
            || trimmed.starts_with(br#""dependencies":{"#)
        {
            dependencies = true;
        } else if trimmed.starts_with(br#""default": {"#) || trimmed.starts_with(br#""default":{"#)
        {
            default_section = true;
        } else if trimmed.starts_with(br#""develop": {"#) || trimmed.starts_with(br#""develop":{"#)
        {
            develop_section = true;
        }

        if (lockfile_version && (packages || dependencies))
            || (meta_section && default_section)
            || (default_section && develop_section)
        {
            return true;
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::find_min_size;
    use super::likely_composer_lockfile_text;
    use super::likely_dependency_json_lockfile_text;
    use super::likely_incompressible;
    use super::likely_lockfile_text;
    use super::likely_text;
    use super::minify_val;
    use alloc::format;
    use alloc::vec;
    use alloc::vec::Vec;

    #[test]
    fn min_size_detection() {
        assert_eq!(find_min_size(0), 1);
        assert_eq!(find_min_size(0xff), 1);
        assert_eq!(find_min_size(0xff_ff), 2);
        assert_eq!(find_min_size(0x00_ff_ff_ff), 4);
        assert_eq!(find_min_size(0xff_ff_ff_ff), 4);
        assert_eq!(find_min_size(0x00ff_ffff_ffff_ffff), 8);
        assert_eq!(find_min_size(0xffff_ffff_ffff_ffff), 8);
    }

    #[test]
    fn bytes_minified() {
        assert_eq!(minify_val(0).collect::<Vec<_>>(), vec![0]);
        assert_eq!(minify_val(0xff).collect::<Vec<_>>(), vec![0xff]);
        assert_eq!(minify_val(0xff_ff).collect::<Vec<_>>(), vec![0xff, 0xff]);
        assert_eq!(
            minify_val(0xff_ff_ff_ff).collect::<Vec<_>>(),
            vec![0xff, 0xff, 0xff, 0xff]
        );
        assert_eq!(
            minify_val(0xffff_ffff_ffff_ffff).collect::<Vec<_>>(),
            vec![0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff]
        );
    }

    #[test]
    fn incompressible_gate_distinguishes_random_from_repetitive_data() {
        let mut state = 0x1234_5678_9ABC_DEF0u64;
        let mut random = Vec::with_capacity(128 * 1024);
        while random.len() < 128 * 1024 {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            random.extend_from_slice(&state.to_le_bytes());
        }
        random.truncate(128 * 1024);
        assert!(likely_incompressible(&random));

        let mut repetitive = Vec::with_capacity(128 * 1024);
        while repetitive.len() < 128 * 1024 {
            repetitive.extend_from_slice(b"tenant=alpha path=/v1/archive status=200\n");
        }
        repetitive.truncate(128 * 1024);
        assert!(!likely_incompressible(&repetitive));
        assert!(likely_text(&repetitive));
        assert!(!likely_text(&random));
    }

    #[test]
    fn lockfile_gate_detects_large_cargo_lock_like_text() {
        let mut data = Vec::new();
        for idx in 0..256 {
            data.extend_from_slice(b"[[package]]\n");
            data.extend_from_slice(format!("name = \"crate-{idx}\"\n").as_bytes());
            data.extend_from_slice(b"version = \"1.0.0\"\n");
            data.extend_from_slice(
                b"checksum = \"0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef\"\n",
            );
        }

        assert!(likely_lockfile_text(&data));
    }

    #[test]
    fn composer_lockfile_gate_detects_large_composer_lock_like_text() {
        let mut data = Vec::new();
        data.extend_from_slice(b"{\n  \"packages\": [\n");
        for idx in 0..256 {
            data.extend_from_slice(b"    {\n");
            data.extend_from_slice(
                format!("      \"name\": \"vendor/package-{idx:04}\",\n").as_bytes(),
            );
            data.extend_from_slice(b"      \"require\": {\n");
            data.extend_from_slice(b"        \"php\": \">=8.2\"\n");
            data.extend_from_slice(b"      },\n");
            data.extend_from_slice(b"      \"source\": {\n");
            data.extend_from_slice(format!("        \"reference\": \"{idx:040}\",\n").as_bytes());
            data.extend_from_slice(b"        \"type\": \"git\"\n");
            data.extend_from_slice(b"      },\n");
            data.extend_from_slice(b"      \"version\": \"1.0.0\"\n");
            data.extend_from_slice(b"    },\n");
        }
        data.extend_from_slice(b"  ]\n}\n");

        assert!(likely_composer_lockfile_text(&data));
    }

    #[test]
    fn dependency_json_lockfile_gate_detects_package_lock_like_text() {
        let mut data = Vec::new();
        data.extend_from_slice(
            b"{\n  \"name\": \"app\",\n  \"lockfileVersion\": 3,\n  \"packages\": {\n",
        );
        for idx in 0..256 {
            data.extend_from_slice(format!("    \"node_modules/pkg-{idx:04}\": {{\n").as_bytes());
            data.extend_from_slice(b"      \"version\": \"1.0.0\",\n");
            data.extend_from_slice(b"      \"dependencies\": {\n");
            data.extend_from_slice(b"        \"dep-0000\": \"^1.0.0\"\n");
            data.extend_from_slice(b"      }\n");
            data.extend_from_slice(b"    },\n");
        }
        data.extend_from_slice(b"  }\n}\n");

        assert!(likely_dependency_json_lockfile_text(&data));
    }

    #[test]
    fn dependency_json_lockfile_gate_detects_pipfile_lock_like_text() {
        let mut data = Vec::new();
        data.extend_from_slice(b"{\n  \"_meta\": {},\n  \"default\": {\n");
        for idx in 0..256 {
            data.extend_from_slice(format!("    \"dep-{idx:04}\": {{\n").as_bytes());
            data.extend_from_slice(b"      \"hashes\": [\n");
            data.extend_from_slice(
                b"        \"sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\"\n",
            );
            data.extend_from_slice(b"      ],\n");
            data.extend_from_slice(b"      \"version\": \"==1.0.0\"\n");
            data.extend_from_slice(b"    },\n");
        }
        data.extend_from_slice(b"  },\n  \"develop\": {\n");
        for idx in 0..256 {
            data.extend_from_slice(format!("    \"devdep-{idx:04}\": {{\n").as_bytes());
            data.extend_from_slice(b"      \"hashes\": [\n");
            data.extend_from_slice(
                b"        \"sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb\"\n",
            );
            data.extend_from_slice(b"      ],\n");
            data.extend_from_slice(b"      \"version\": \"==2.0.0\"\n");
            data.extend_from_slice(b"    },\n");
        }
        data.extend_from_slice(b"  }\n}\n");

        assert!(likely_dependency_json_lockfile_text(&data));
    }
}
