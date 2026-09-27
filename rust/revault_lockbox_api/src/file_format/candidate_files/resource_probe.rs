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
    if phase == "dense-create" || phase == "dense-lifecycle" || phase == "dense-tail-lifecycle" {
        let cycles = if phase != "dense-create" {
            std::env::var("REVAULT_CANDIDATE_CYCLES")
                .unwrap()
                .parse::<usize>()
                .unwrap()
        } else {
            0
        };
        assert!(cycles <= 1000);
        dense_create(
            &root,
            count,
            bytes,
            unit,
            mode,
            key,
            cycles,
            phase == "dense-tail-lifecycle",
        );
        return;
    }
    if phase == "dense-damage" {
        dense_damage(&root, count, bytes, mode, key);
        return;
    }
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
    } else if phase == "paged-cost" {
        let before = digest_file(&path);
        let result = super::paged_cost::project(&mut open(), &authority, key).unwrap();
        assert_eq!(digest_file(&path), before);
        result
    } else if phase == "cost-model" {
        super::cost_model::project(&mut open()).unwrap()
    } else if phase == "inspect" {
        let files = open();
        let snapshot = Snapshot::inspect(&files.storage, &files.anchor, &files.index).unwrap();
        snapshot.verify_reclaimed(&files.storage).unwrap();
        let a = snapshot.accounting;
        let mut fragments = 0u64;
        let mut packs = std::collections::BTreeSet::new();
        files
            .index
            .visit(
                &files.storage,
                files.anchor.index,
                files.anchor.sealed_len,
                |entry| {
                    if entry.namespace == CHUNK {
                        let owned = OwnedRecord::decode(&entry.value)?;
                        let slice = Slice::decode(&owned.metadata)?;
                        fragments += slice.physical(owned.extents[0])?.len;
                        packs.insert(owned.extents[0].start);
                    }
                    Ok(())
                },
            )
            .unwrap();
        json!({"kind":"accounting","total":a.total,"fixed":a.fixed,"payload":a.payload,
            "index":a.index,"keys":a.keys,"allocation":a.allocation,"reserve":a.reserve,
            "free":a.free,"pending":a.pending,"physical_packs":packs.len(),
            "stored_fragments":fragments,"pack_padding":a.payload-fragments,
            "archive_sha256":digest_file(&path),"verified_reclaimed":true})
    } else if phase == "compact" {
        // Fresh-process resource probe; signed installation requires the original
        // owner, which this read-probe protocol intentionally never persists.
        assert!(
            !signed,
            "signed compaction is covered by the synthetic correctness matrix"
        );
        let original_bytes = std::fs::metadata(&path).unwrap().len();
        let before = Resources::now();
        let started = Instant::now();
        let compacted = Files::compact_path(&path, archive(), mode, &authority, None, key).unwrap();
        drop(compacted);
        let elapsed = started.elapsed().as_secs_f64();
        let after = Resources::now();
        let replacement_bytes = std::fs::metadata(&path).unwrap().len();
        verify(&mut open(), &root, count, bytes);
        json!({"kind":"compaction","backend":"lockbox","wall_seconds":elapsed,"resources":after.delta(before),"peak_rss_kib":after.peak,"baseline_peak_rss_kib":before.peak,"source_archive_bytes":original_bytes,"replacement_archive_bytes":replacement_bytes,"peak_extra_logical_file_bytes":replacement_bytes,"space_method":"replacement is append-only until atomic rename; excludes filesystem allocation granularity and existing backups","verified":true})
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
    result["file_adapter_version"] = json!(2);
    result["candidate_scope"] =
        json!("file-only shared private packs; no public record/access integration");
    println!("CANDIDATE_SAMPLE {result}");
}

/// Correctness/size probe only. Building C as a source is not a comparable
/// creation workload, so no time/CPU/RSS fields are reported for this phase.
#[allow(clippy::too_many_arguments)]
fn dense_create(
    root: &Path,
    count: usize,
    bytes: u64,
    unit: usize,
    mode: FormatMode,
    key: Option<&[u8]>,
    cycles: usize,
    trim: bool,
) {
    use super::dense_image::{from_candidate, Image};
    let source_path = root.join("dense-staging.lbox");
    let target = root.join("dense.lbox");
    let owner_path = root.join("dense.public");
    assert!(!source_path.exists() && !target.exists() && !owner_path.exists());
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let public = owner.public_key();
    let authority = if mode.signed() {
        Authority::Owner(&public)
    } else if mode.plaintext() {
        Authority::Checksum
    } else {
        Authority::Symmetric(KEY)
    };
    let signer = mode.signed().then_some(&owner);
    let inputs = (0..count).map(|index| {
        let name = name(index);
        Input {
            path: name.as_bytes().to_vec(),
            reader: File::open(root.join("source").join(name)).unwrap(),
        }
    });
    let staging = Files::create(
        StorageBackend::create_file(&source_path, &[]).unwrap(),
        archive(),
        mode,
        &authority,
        signer,
        key,
        unit,
        inputs,
    )
    .unwrap();
    let mut source = Files::open(staging, archive(), mode, &authority, key).unwrap();
    let slots = if mode.plaintext() {
        Vec::new()
    } else {
        vec![crate::key_slot::KeySlot::password_bytes(
            1,
            b"synthetic dense fixture password",
            vec![85; 16],
            KEY,
        )
        .unwrap()]
    };
    let output = from_candidate(
        &mut source,
        StorageBackend::create_file(&target, &[]).unwrap(),
        &authority,
        signer,
        key,
        &slots,
    )
    .unwrap();
    drop(output);
    std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&owner_path)
        .unwrap()
        .write_all(&public.to_bytes())
        .unwrap();
    let mut reopened = Image::open(
        StorageBackend::file(&target).unwrap(),
        archive(),
        mode,
        &authority,
        key,
    )
    .unwrap();
    let mut expected = vec![0; MAX_LOGICAL];
    for index in 0..count {
        let name = name(index);
        let mut input = File::open(root.join("source").join(&name)).unwrap();
        let mut actual = 0;
        reopened
            .read_range(name.as_bytes(), 0, bytes, |part| {
                input.read_exact(&mut expected[..part.len()]).unwrap();
                assert_eq!(part, &expected[..part.len()]);
                actual += part.len() as u64;
                Ok(())
            })
            .unwrap();
        assert_eq!(actual, bytes);
        assert_eq!(input.read(&mut expected[..1]).unwrap(), 0);
    }
    drop(reopened);
    let initial_bytes = std::fs::metadata(&target).unwrap().len();
    let mut first_update_bytes = initial_bytes;
    let mut temporary_update_bytes = initial_bytes;
    for cycle in 0..cycles {
        let bits = if cycle % 2 == 0 { 0o600 } else { 0o644 };
        let mut storage = StorageBackend::file_for_write(&target).unwrap();
        assert!(super::dense_update::edit(
            &mut storage,
            archive(),
            mode,
            &authority,
            signer,
            key,
            name(0).as_bytes(),
            name(0).as_bytes(),
            Some(bits)
        )
        .unwrap());
        temporary_update_bytes = temporary_update_bytes.max(storage.len().unwrap());
        if trim {
            assert!(super::dense_update::return_inline(
                &mut storage,
                archive(),
                mode,
                &authority,
                signer,
                key
            )
            .unwrap());
            assert_eq!(storage.len().unwrap(), initial_bytes);
        }
        if cycle == 0 {
            first_update_bytes = storage.len().unwrap();
        }
        assert_eq!(storage.len().unwrap(), first_update_bytes);
        drop(storage);
        // Separate handle and source-byte comparison for every committed edit.
        let mut image = Image::open(
            StorageBackend::file(&target).unwrap(),
            archive(),
            mode,
            &authority,
            key,
        )
        .unwrap();
        for index in 0..count {
            let path = name(index);
            let mut input = File::open(root.join("source").join(&path)).unwrap();
            let mut actual = 0;
            image
                .read_range(path.as_bytes(), 0, bytes, |part| {
                    input.read_exact(&mut expected[..part.len()]).unwrap();
                    assert!(part == &expected[..part.len()]);
                    actual += part.len() as u64;
                    Ok(())
                })
                .unwrap();
            assert_eq!(actual, bytes);
            assert_eq!(input.read(&mut expected[..1]).unwrap(), 0);
        }
        let (anchor, body) =
            publication::shared::open_private(&image.storage, archive(), mode, &authority, key)
                .unwrap();
        let codec = Codec::shared_packed(archive(), mode, key).unwrap();
        let catalogue =
            super::dense_catalogue::Catalogue::decode(&body, &codec, anchor.sealed_len).unwrap();
        assert_eq!(
            anchor.generation,
            (cycle as u64 + 1) * if trim { 2 } else { 1 } + 1
        );
        assert_eq!(catalogue.files[0].permissions, bits);
        drop(image);
        let before = digest_file(&target);
        let mut storage = StorageBackend::file_for_write(&target).unwrap();
        assert!(!super::dense_update::edit(
            &mut storage,
            archive(),
            mode,
            &authority,
            signer,
            key,
            name(0).as_bytes(),
            name(0).as_bytes(),
            Some(bits)
        )
        .unwrap());
        drop(storage);
        assert_eq!(digest_file(&target), before);
    }
    drop(source);
    std::fs::remove_file(&source_path).unwrap();
    let result = json!({"kind":"dense_file_image_size","archive_bytes":std::fs::metadata(&target).unwrap().len(),"archive_sha256":digest_file(&target),"binary_sha256":digest_file(&std::env::current_exe().unwrap()),"files":count,"bytes_per_file":bytes,"verified":true,"scope":"fresh file-only image; no mutation/public API/migration or performance qualification"});
    let result = if cycles == 0 {
        result
    } else {
        json!({"kind":if trim {"dense_metadata_tail_lifecycle_size"} else {"dense_metadata_lifecycle_size"},"metadata_tail_retirement":trim,"temporary_update_bytes":temporary_update_bytes,"initial_bytes":initial_bytes,"first_update_bytes":first_update_bytes,"archive_bytes":std::fs::metadata(&target).unwrap().len(),"cycles":cycles,"unchanged_repeats":cycles,"stable_after_first_update":true,"fresh_handle_source_bytes_verified_each_edit":true,"archive_sha256":digest_file(&target),"binary_sha256":digest_file(&std::env::current_exe().unwrap()),"files":count,"bytes_per_file":bytes,"scope":"bounded metadata updates only; no payload edits, full aging or performance qualification"})
    };
    println!("CANDIDATE_SAMPLE {result}");
}

struct VerifyRecovered<'a> {
    root: &'a Path,
    count: usize,
    bytes: u64,
    active: Option<(File, u64)>,
    seen: std::collections::BTreeSet<usize>,
    scratch: Vec<u8>,
}
impl recovery::Sink for VerifyRecovered<'_> {
    fn begin(&mut self, path: &[u8], len: u64) -> Result<()> {
        assert!(self.active.is_none());
        assert_eq!(len, self.bytes);
        let text = std::str::from_utf8(path).unwrap();
        let index: usize = text
            .strip_prefix("file-")
            .unwrap()
            .strip_suffix(".bin")
            .unwrap()
            .parse()
            .unwrap();
        assert!(index < self.count && self.seen.insert(index));
        assert_eq!(text, name(index));
        self.active = Some((
            File::open(self.root.join("source").join(name(index))).unwrap(),
            0,
        ));
        Ok(())
    }
    fn data(&mut self, bytes: &[u8]) -> Result<()> {
        let (file, position) = self.active.as_mut().unwrap();
        file.read_exact(&mut self.scratch[..bytes.len()]).unwrap();
        assert_eq!(bytes, &self.scratch[..bytes.len()]);
        *position += bytes.len() as u64;
        Ok(())
    }
    fn finish(&mut self, complete: bool) -> Result<()> {
        let (mut file, position) = self.active.take().unwrap();
        if complete {
            assert_eq!(position, self.bytes);
            assert_eq!(file.read(&mut self.scratch[..1]).unwrap(), 0);
        }
        Ok(())
    }
}
/// Damage only in-memory copies. All successfully recovered file bytes are
/// compared with the retained corpus; original archives remain unchanged.
fn dense_damage(root: &Path, count: usize, bytes: u64, mode: FormatMode, key: Option<&[u8]>) {
    use crate::file_format::publication_anchor::{FAILURE_REGION, REGION_LEN};
    let control = std::path::PathBuf::from(std::env::var_os("REVAULT_CANDIDATE_CONTROL").unwrap());
    let mut results = Vec::new();
    for dense in [false, true] {
        let (path, public_path) = if dense {
            (root.join("dense.lbox"), root.join("dense.public"))
        } else {
            (
                control.join("candidate.lbox"),
                control.join("candidate.public"),
            )
        };
        let hash_before = digest_file(&path);
        let public =
            OwnerSigningPublicKey::from_bytes(&std::fs::read(public_path).unwrap()).unwrap();
        let authority = if mode.signed() {
            Authority::Owner(&public)
        } else if mode.plaintext() {
            Authority::Checksum
        } else {
            Authority::Symmetric(KEY)
        };
        let seed = std::fs::read(&path).unwrap();
        let at = if dense {
            REGION_LEN as u64
        } else {
            let files = Files::open(
                StorageBackend::memory(seed.clone()),
                archive(),
                mode,
                &authority,
                key,
            )
            .unwrap();
            let info = files.info(name(0).as_bytes()).unwrap().unwrap();
            let record = files
                .index
                .get(
                    &files.storage,
                    files.anchor.index,
                    files.anchor.sealed_len,
                    CHUNK,
                    &chunk_key(info.id, 0),
                )
                .unwrap()
                .unwrap();
            let owned = OwnedRecord::decode(&record.value).unwrap();
            assert_eq!(owned.extents[0].start % FAILURE_REGION, 0);
            owned.extents[0].start
        };
        for region in [false, true] {
            let mut damaged = StorageBackend::memory(seed.clone());
            if region {
                let n = (seed.len() as u64 - at).min(FAILURE_REGION) as usize;
                damaged.write_at(at, &vec![0; n]).unwrap();
            } else {
                let byte = damaged.read_at(at, 1).unwrap()[0];
                damaged.write_at(at, &[byte ^ 1]).unwrap();
            }
            let mut sink = VerifyRecovered {
                root,
                count,
                bytes,
                active: None,
                seen: Default::default(),
                scratch: vec![0; MAX_LOGICAL],
            };
            let (complete, incomplete) = if dense {
                let report = super::dense_image::salvage(
                    &damaged,
                    archive(),
                    mode,
                    &authority,
                    key,
                    &mut sink,
                )
                .unwrap();
                (report.complete, report.incomplete)
            } else {
                let report =
                    Files::salvage(&damaged, archive(), mode, &authority, key, &mut sink).unwrap();
                assert_eq!(report.membership.unavailable, 0);
                assert_eq!(report.orphan_chunks, 0);
                (report.complete, report.incomplete)
            };
            assert_eq!(sink.seen.len(), count);
            assert!(sink.active.is_none());
            assert_eq!(complete + incomplete, count as u64);
            results.push(json!({"layout":if dense { "shared-control" } else { "C-control" },"damage":if region { "64KiB payload region" } else { "one payload byte" },"complete_files":complete,"incomplete_files":incomplete,"incomplete_file_bytes":incomplete * bytes,"recovered_bytes_verified":true}));
        }
        assert_eq!(digest_file(&path), hash_before);
    }
    let result = json!({"kind":"dense_recovery_comparison","files":count,"bytes_per_file":bytes,"results":results,"source_archives_unchanged":true,"scope":"read-only corruption comparison, no performance claim","binary_sha256":digest_file(&std::env::current_exe().unwrap())});
    println!("CANDIDATE_SAMPLE {result}");
}
