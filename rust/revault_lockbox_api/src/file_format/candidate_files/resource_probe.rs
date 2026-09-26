//! Protocol adapter for the common A/B/ZIP fresh-process runner. Test-only C.
use super::*;
use crate::storage::StorageBackend;
use crate::{
    Compression, EncryptionMode, LockboxFormatOptions, OwnerSigningPublicKey, SigningMode,
    SizePadding,
};
use serde_json::{json, Value};
use std::fs::File;
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::Path;
use std::time::Instant;
const KEY: &[u8; 32] = &[71; 32];
fn archive() -> LockboxId {
    LockboxId::from_bytes([18; 16])
}
fn name(index: usize) -> String {
    format!("file-{index:06}.bin")
}
fn digest_file(path: &Path) -> String {
    let mut file = File::open(path).unwrap();
    let mut hash = Sha256::new();
    let mut buffer = [0; 65536];
    loop {
        let count = file.read(&mut buffer).unwrap();
        if count == 0 {
            break;
        }
        hash.update(&buffer[..count]);
    }
    hash.finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}
#[derive(Clone, Copy)]
struct Resources {
    user: f64,
    system: f64,
    peak: i64,
    faults: i64,
}
impl Resources {
    fn now() -> Self {
        let mut value = std::mem::MaybeUninit::<libc::rusage>::uninit();
        // SAFETY: getrusage initializes the valid output pointer on success.
        let status = unsafe { libc::getrusage(libc::RUSAGE_SELF, value.as_mut_ptr()) };
        assert_eq!(status, 0);
        // SAFETY: the successful call above initialized the structure.
        let value = unsafe { value.assume_init() };
        let seconds = |time: libc::timeval| time.tv_sec as f64 + time.tv_usec as f64 / 1e6;
        Self {
            user: seconds(value.ru_utime),
            system: seconds(value.ru_stime),
            peak: value.ru_maxrss,
            faults: value.ru_majflt,
        }
    }
    fn delta(self, before: Self) -> Value {
        json!({"user_seconds":self.user-before.user,"system_seconds":self.system-before.system,"cpu_seconds":self.user+self.system-before.user-before.system,"major_faults":self.faults-before.faults})
    }
}
fn verify(files: &mut Files<StorageBackend>, root: &Path, count: usize, bytes: u64) {
    let mut expected = vec![0; MAX_LOGICAL];
    for index in 0..count {
        let name = name(index);
        let mut source = File::open(root.join("source").join(&name)).unwrap();
        assert_eq!(files.info(name.as_bytes()).unwrap().unwrap().len, bytes);
        let mut position = 0;
        files
            .read_range(name.as_bytes(), 0, bytes, |chunk| {
                source.read_exact(&mut expected[..chunk.len()]).unwrap();
                assert!(
                    chunk == &expected[..chunk.len()],
                    "persisted bytes differ: file={index},offset={position}"
                );
                position += chunk.len() as u64;
                Ok(())
            })
            .unwrap();
        assert_eq!(position, bytes);
        assert_eq!(source.read(&mut expected[..1]).unwrap(), 0);
    }
}
#[test]
#[ignore = "common-runner candidate C CPU/RSS and extent-size evaluation"]
fn candidate_file_resource_probe() {
    let root = std::path::PathBuf::from(std::env::var_os("REVAULT_CANDIDATE_ROOT").unwrap());
    let phase = std::env::var("REVAULT_CANDIDATE_PHASE").unwrap();
    let unit: usize = std::env::var("REVAULT_CANDIDATE_UNIT")
        .unwrap()
        .parse()
        .unwrap();
    assert!([65536, MAX_LOGICAL].contains(&unit));
    let case: Value =
        serde_json::from_slice(&std::fs::read(root.join("case.json")).unwrap()).unwrap();
    let count = case["files"].as_u64().unwrap() as usize;
    let bytes = case["bytes"].as_u64().unwrap();
    assert!((1..=100000).contains(&count));
    let encrypted = matches!(
        case["mode"].as_str().unwrap(),
        "encrypted" | "encrypted-signed"
    );
    let signed = matches!(
        case["mode"].as_str().unwrap(),
        "signed" | "encrypted-signed"
    );
    let mode = FormatMode::new(LockboxFormatOptions {
        encryption: if encrypted {
            EncryptionMode::ChaCha20Poly1305
        } else {
            EncryptionMode::None
        },
        signing: if signed {
            SigningMode::Owner
        } else {
            SigningMode::None
        },
        compression: if case["compressed"].as_bool().unwrap() {
            Compression::default()
        } else {
            Compression::None
        },
        size_padding: if case["unpadded"].as_bool().unwrap() {
            SizePadding::None
        } else {
            SizePadding::Default
        },
    });
    let key = encrypted.then_some(KEY.as_slice());
    let path = root.join("candidate.lbox");
    let binary_hash = digest_file(&std::env::current_exe().unwrap());
    if phase == "create" {
        assert!(!path.exists());
        assert!(!root.join("candidate.public").exists());
    }
    let owner = (phase == "create").then(|| OwnerSigningKeyPair::generate().unwrap());
    let public = if let Some(owner) = &owner {
        let public = owner.public_key();
        std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(root.join("candidate.public"))
            .unwrap()
            .write_all(&public.to_bytes())
            .unwrap();
        public
    } else {
        OwnerSigningPublicKey::from_bytes(&std::fs::read(root.join("candidate.public")).unwrap())
            .unwrap()
    };
    let authority = if signed {
        Authority::Owner(&public)
    } else if encrypted {
        Authority::Symmetric(KEY)
    } else {
        Authority::Checksum
    };
    let open = || {
        Files::open(
            StorageBackend::file(&path).unwrap(),
            archive(),
            mode,
            &authority,
            key,
        )
        .unwrap()
    };
    let mut result = if phase == "create" {
        assert!(!path.exists());
        let before = Resources::now();
        let started = Instant::now();
        let inputs = (0..count).map(|index| {
            let name = name(index);
            Input {
                path: name.as_bytes().to_vec(),
                reader: File::open(root.join("source").join(name)).unwrap(),
            }
        });
        let storage = Files::create(
            StorageBackend::create_file(&path, &[]).unwrap(),
            archive(),
            mode,
            &authority,
            if signed { owner.as_ref() } else { None },
            key,
            unit,
            inputs,
        )
        .unwrap();
        drop(storage);
        let elapsed = started.elapsed().as_secs_f64();
        let after = Resources::now();
        verify(&mut open(), &root, count, bytes);
        json!({"kind":"fixture_create","backend":"lockbox","wall_seconds":elapsed,"resources":after.delta(before),"peak_rss_kib":after.peak,"baseline_peak_rss_kib":before.peak,"archive_bytes":std::fs::metadata(&path).unwrap().len(),"archive_sha256":digest_file(&path),"verified":true})
    } else {
        assert_eq!(phase, "sample");
        let access = std::env::var("REVAULT_CANDIDATE_ACCESS").unwrap();
        assert!(["stream", "range"].contains(&access.as_str()));
        let passes: usize = std::env::var("REVAULT_CANDIDATE_PASSES")
            .unwrap()
            .parse()
            .unwrap();
        assert!((1..=1000).contains(&passes));
        let before = Resources::now();
        let started = Instant::now();
        let mut files = open();
        let open_seconds = started.elapsed().as_secs_f64();
        let opened = Resources::now();
        let read_started = Instant::now();
        let mut first = None;
        let mut total = 0;
        let mut copy = [0; 65536];
        for _ in 0..passes {
            for index in 0..count {
                let name = name(index);
                let (offset, len) = if access == "range" {
                    (bytes / 2, 4096.min(bytes - bytes / 2))
                } else {
                    (0, bytes)
                };
                files
                    .read_range(name.as_bytes(), offset, len, |chunk| {
                        for part in chunk.chunks(copy.len()) {
                            copy[..part.len()].copy_from_slice(part);
                            first.get_or_insert_with(|| started.elapsed().as_secs_f64());
                            std::hint::black_box(&copy[..part.len()]);
                            total += part.len() as u64;
                        }
                        Ok(())
                    })
                    .unwrap();
            }
        }
        let read_seconds = read_started.elapsed().as_secs_f64();
        let elapsed = started.elapsed().as_secs_f64();
        let after = Resources::now();
        drop(files);
        // Independent requested-range and full-file checks are outside timers.
        if access == "range" {
            let mut checked = open();
            let offset = bytes / 2;
            let len = 4096.min(bytes - offset);
            let mut expected = [0; 4096];
            for index in 0..count {
                let name = name(index);
                let mut source = File::open(root.join("source").join(&name)).unwrap();
                source.seek(SeekFrom::Start(offset)).unwrap();
                source.read_exact(&mut expected[..len as usize]).unwrap();
                let mut position = 0;
                checked
                    .read_range(name.as_bytes(), offset, len, |chunk| {
                        assert!(chunk == &expected[position..position + chunk.len()]);
                        position += chunk.len();
                        Ok(())
                    })
                    .unwrap();
                assert_eq!(position, len as usize);
            }
        }
        verify(&mut open(), &root, count, bytes);
        json!({"kind":"sample","backend":"lockbox","access":access,"passes":passes,"open_seconds":open_seconds,"read_seconds":read_seconds,"total_seconds":elapsed,"first_byte_seconds":first.unwrap_or(0.0),"logical_bytes_read":total,"probe_overhead_seconds":elapsed-open_seconds-read_seconds,"open_resources":opened.delta(before),"read_resources":after.delta(opened),"resources":after.delta(before),"peak_rss_kib":after.peak,"baseline_peak_rss_kib":before.peak,"verified":true})
    };
    result["layout"] = json!(format!("candidate-{unit}"));
    result["candidate_test_executable_sha256"] = json!(binary_hash);
    result["extent_unit"] = json!(unit);
    result["candidate_scope"] =
        json!("file-only; no small-file packing or public record/access integration");
    println!("CANDIDATE_SAMPLE {result}");
}
