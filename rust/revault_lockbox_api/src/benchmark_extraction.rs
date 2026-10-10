//! Shared test/benchmark-only extraction sink. Not a public extraction API.
//! Synthetic names come from the fixture contract, never untrusted archive paths.
use serde_json::{json, Value};
use std::{
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::Path,
    time::Instant,
};

#[allow(clippy::too_many_arguments)]
pub(crate) fn measure<H>(
    root: &Path,
    output: &Path,
    count: usize,
    bytes: u64,
    open: impl FnOnce() -> H,
    mut copy: impl FnMut(&mut H, usize, &mut File) -> u64,
    resources: impl FnOnce() -> Value,
) -> Value {
    assert!((1..=100_000).contains(&count));
    assert!(!output.exists(), "extraction requires a fresh destination");
    let started = Instant::now();
    let mut archive = open();
    let open_seconds = started.elapsed().as_secs_f64();
    fs::create_dir(output).unwrap();
    let mut total = 0u64;
    for index in 0..count {
        let name = format!("file-{index:06}.bin");
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(output.join(name))
            .unwrap();
        let copied = copy(&mut archive, index, &mut file);
        assert_eq!(copied, bytes);
        file.flush().unwrap();
        drop(file);
        total = total.checked_add(copied).unwrap();
    }
    drop(archive);
    let seconds = started.elapsed().as_secs_f64();
    let resources = resources();
    // Reopen persisted filesystem outputs after the timing/resource snapshot.
    assert_eq!(fs::read_dir(output).unwrap().count(), count);
    let mut expected = [0u8; 65_536];
    let mut actual = [0u8; 65_536];
    for index in 0..count {
        let name = format!("file-{index:06}.bin");
        let mut source = File::open(root.join("source").join(&name)).unwrap();
        let mut extracted = File::open(output.join(&name)).unwrap();
        assert_eq!(source.metadata().unwrap().len(), bytes);
        assert_eq!(extracted.metadata().unwrap().len(), bytes);
        let mut remaining = bytes;
        while remaining != 0 {
            let n = remaining.min(expected.len() as u64) as usize;
            source.read_exact(&mut expected[..n]).unwrap();
            extracted.read_exact(&mut actual[..n]).unwrap();
            assert_eq!(&expected[..n], &actual[..n], "{name}: output bytes differ");
            remaining -= n as u64;
        }
        assert_eq!(source.read(&mut expected[..1]).unwrap(), 0);
        assert_eq!(extracted.read(&mut actual[..1]).unwrap(), 0);
    }
    json!({"kind":"filesystem_extraction", "open_seconds":open_seconds,
        "total_seconds":seconds,"logical_bytes_written":total,
        "mib_per_second":total as f64 / 1_048_576.0 / seconds,
        "resources":resources,"files":count,"verified":true,
        "durability":"write/flush/close; no fsync; directory creation included",
        "scope":"synthetic regular-file component; excludes CLI startup and permission restoration"})
}

#[cfg(test)]
// The harness=false benchmark includes this module but does not run its tests.
#[allow(dead_code)]
mod tests {
    use super::*;
    use std::{
        path::PathBuf,
        sync::atomic::{AtomicU64, Ordering},
    };
    static NEXT: AtomicU64 = AtomicU64::new(0);
    struct Scratch(PathBuf);
    impl Scratch {
        fn new() -> Self {
            let path = std::env::temp_dir().join(format!(
                "revault-extraction-check-{}-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
            ));
            fs::create_dir(&path).unwrap();
            fs::create_dir(path.join("source")).unwrap();
            Self(path)
        }
    }
    impl Drop for Scratch {
        fn drop(&mut self) {
            fs::remove_dir_all(&self.0).unwrap();
        }
    }
    #[test]
    fn extraction_checks_reopened_bytes_across_buffer_boundary() {
        let root = Scratch::new();
        let bytes: Vec<u8> = (0..65_539).map(|n| (n % 251) as u8).collect();
        fs::write(root.0.join("source/file-000000.bin"), &bytes).unwrap();
        let output = root.0.join("output");
        let result = measure(
            &root.0,
            &output,
            1,
            bytes.len() as u64,
            || (),
            |_, _, file| {
                file.write_all(&bytes).unwrap();
                bytes.len() as u64
            },
            || json!({}),
        );
        assert_eq!(result["verified"], true);
        assert_eq!(fs::read(output.join("file-000000.bin")).unwrap(), bytes);
    }
    #[test]
    fn extraction_rejects_wrong_output_even_with_correct_reported_count() {
        let root = Scratch::new();
        fs::write(root.0.join("source/file-000000.bin"), b"correct").unwrap();
        let output = root.0.join("output");
        assert!(std::panic::catch_unwind(|| measure(
            &root.0,
            &output,
            1,
            7,
            || (),
            |_, _, file| {
                file.write_all(b"corrupt").unwrap();
                7
            },
            || json!({})
        ))
        .is_err());
    }
    #[test]
    fn extraction_refuses_existing_destination_before_opening_archive() {
        let root = Scratch::new();
        let output = root.0.join("output");
        fs::create_dir(&output).unwrap();
        fs::write(output.join("keep"), b"preserve").unwrap();
        let opened = std::cell::Cell::new(false);
        assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            measure(
                &root.0,
                &output,
                1,
                1,
                || {
                    opened.set(true);
                },
                |_, _, _| 1,
                || json!({}),
            )
        }))
        .is_err());
        assert!(!opened.get());
        assert_eq!(fs::read(output.join("keep")).unwrap(), b"preserve");
    }
}
