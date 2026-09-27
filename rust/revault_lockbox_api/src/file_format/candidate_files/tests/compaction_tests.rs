//! Candidate C has no public CLI writer; these deliberately use the internal
//! adapter, real files and fresh handles to qualify its proposed installation.
use super::*;
use std::path::PathBuf;
struct Directory(PathBuf);
impl Directory {
    fn new() -> Self {
        let mut nonce = [0; 8];
        getrandom::fill(&mut nonce).unwrap();
        let path = std::env::temp_dir().join(format!(
            "revault-candidate-compact-{}-{}",
            std::process::id(),
            u64::from_le_bytes(nonce)
        ));
        std::fs::create_dir(&path).unwrap();
        Self(path)
    }
}
impl Drop for Directory {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
fn protection(n: usize) -> FormatMode {
    mode(n & 1 != 0, n & 2 != 0, n & 1 == 0, true)
}
fn aged(
    mode: FormatMode,
    authority: &Authority<'_>,
    signer: Option<&OwnerSigningKeyPair>,
) -> StorageBackend {
    let storage = packed_pair(mode, authority, signer);
    let storage = Files::update(
        storage,
        archive(),
        mode,
        authority,
        signer,
        key(mode),
        MAX_LOGICAL,
        update_inputs(&[(b"erase", &vec![0x55; 700_000])]),
        [],
    )
    .unwrap();
    Files::update(
        storage,
        archive(),
        mode,
        authority,
        signer,
        key(mode),
        MAX_LOGICAL,
        update_inputs(&[(b"erase", b"current")]),
        [],
    )
    .unwrap()
}
fn verify(files: &mut Files<StorageBackend>) {
    assert_file(files, b"erase", Some(b"current"));
    assert_file(files, b"keep", Some(&vec![0x39; 8192]));
}
#[test]
fn file_compaction_installs_verified_writable_archive() {
    let directory = Directory::new();
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let public = owner.public_key();
    for n in 0..4 {
        let mode = protection(n);
        let authority = authority(mode, &public);
        let signer = mode.signed().then_some(&owner);
        let source = aged(mode, &authority, signer);
        let old = publication::select(&source, archive(), mode, &authority).unwrap();
        let path = directory.0.join(format!("{n}.candidate"));
        drop(StorageBackend::create_file(&path, &source.read_all().unwrap()).unwrap());
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o640)).unwrap();
        }
        let mut installed =
            Files::compact_path(&path, archive(), mode, &authority, signer, key(mode)).unwrap();
        assert!(installed.storage.len().unwrap() < source.len().unwrap());
        assert_eq!(installed.anchor.generation, old.anchor.generation + 1);
        assert_eq!(installed.anchor.previous, old.commitment);
        verify(&mut installed);
        let storage = Files::update(
            installed.into_storage(),
            archive(),
            mode,
            &authority,
            signer,
            key(mode),
            MAX_LOGICAL,
            update_inputs(&[(b"added", b"after installed compaction")]),
            [b"erase".to_vec()],
        )
        .unwrap();
        drop(storage);
        let mut reopened = Files::open(
            StorageBackend::file(&path).unwrap(),
            archive(),
            mode,
            &authority,
            key(mode),
        )
        .unwrap();
        assert_file(&mut reopened, b"erase", None);
        assert_file(&mut reopened, b"added", Some(b"after installed compaction"));
        assert_file(&mut reopened, b"keep", Some(&vec![0x39; 8192]));
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(
                std::fs::metadata(&path).unwrap().permissions().mode() & 0o777,
                0o640
            );
        }
    }
    assert_eq!(std::fs::read_dir(&directory.0).unwrap().count(), 4);
}
#[test]
fn compaction_process_child() {
    let Some(path) = std::env::var_os("REVAULT_CANDIDATE_COMPACTION_PATH") else {
        return;
    };
    let path = PathBuf::from(path);
    let n: usize = std::env::var("REVAULT_CANDIDATE_COMPACTION_MODE")
        .unwrap()
        .parse()
        .unwrap();
    let mode = protection(n);
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let public = owner.public_key();
    let authority = authority(mode, &public);
    let signer = mode.signed().then_some(&owner);
    let source = aged(mode, &authority, signer);
    std::fs::write(path.with_extension("public"), public.to_bytes()).unwrap();
    std::fs::write(path.with_extension("base"), source.read_all().unwrap()).unwrap();
    drop(StorageBackend::create_file(&path, &source.read_all().unwrap()).unwrap());
    Files::compact_path(&path, archive(), mode, &authority, signer, key(mode)).unwrap();
    panic!("did not reach compaction checkpoint");
}
#[test]
fn compaction_process_death_preserves_old_or_installs_complete_new_archive() {
    use std::process::{Command, Stdio};
    use std::time::{Duration, Instant};
    let directory = Directory::new();
    let mut cases = 0;
    for n in 0..4 {
        for phase in ["payloads", "dependencies", "verified", "installed"] {
            let path = directory.0.join(format!("{n}-{phase}.candidate"));
            let mut child = Command::new(std::env::current_exe().unwrap())
            .args(["--exact", "file_format::candidate_files::tests::compaction_tests::compaction_process_child", "--nocapture"])
            .env("REVAULT_CANDIDATE_COMPACTION_PATH", &path)
            .env("REVAULT_CANDIDATE_COMPACTION_MODE", n.to_string())
            .env("REVAULT_CANDIDATE_COMPACTION_EXIT", phase)
            .stdout(Stdio::null()).stderr(Stdio::inherit()).spawn().unwrap();
            let deadline = Instant::now() + Duration::from_secs(30);
            let status = loop {
                if let Some(status) = child.try_wait().unwrap() {
                    break status;
                }
                if Instant::now() >= deadline {
                    let _ = child.kill();
                    let _ = child.wait();
                    panic!("compaction child timed out at {phase}");
                }
                std::thread::sleep(Duration::from_millis(10));
            };
            assert_eq!(status.code(), Some(71), "{n}/{phase}");
            let mode = protection(n);
            let public = crate::OwnerSigningPublicKey::from_bytes(
                &std::fs::read(path.with_extension("public")).unwrap(),
            )
            .unwrap();
            let authority = authority(mode, &public);
            let baseline = std::fs::read(path.with_extension("base")).unwrap();
            let base = StorageBackend::memory(baseline.clone());
            let old = publication::select(&base, archive(), mode, &authority).unwrap();
            let mut reopened = Files::open(
                StorageBackend::file(&path).unwrap(),
                archive(),
                mode,
                &authority,
                key(mode),
            )
            .unwrap();
            verify(&mut reopened);
            if phase == "installed" {
                assert_eq!(reopened.anchor.generation, old.anchor.generation + 1);
                assert_eq!(reopened.anchor.previous, old.commitment);
                assert!(reopened.storage.len().unwrap() < baseline.len() as u64);
            } else {
                assert_eq!(reopened.storage.read_all().unwrap(), baseline);
                assert_eq!(reopened.anchor, old.anchor);
            }
            cases += 1;
        }
    }
    println!("CANDIDATE_COMPACTION_PROCESS_DEATH_CASES {cases}");
}

#[test]
fn file_compaction_cleans_failed_output_and_refuses_symlink() {
    let directory = Directory::new();
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let public = owner.public_key();
    let mode = protection(2);
    let authority = authority(mode, &public);
    let source = aged(mode, &authority, Some(&owner));
    let path = directory.0.join("source.candidate");
    let original = source.read_all().unwrap();
    drop(StorageBackend::create_file(&path, &original).unwrap());
    // Missing signing capability is discovered at replacement publication. Even
    // that late failure must retain the source and remove the private output.
    assert!(Files::compact_path(&path, archive(), mode, &authority, None, key(mode)).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), original);
    assert_eq!(std::fs::read_dir(&directory.0).unwrap().count(), 1);
    #[cfg(unix)]
    {
        let link = directory.0.join("link.candidate");
        std::os::unix::fs::symlink(&path, &link).unwrap();
        assert!(
            Files::compact_path(&link, archive(), mode, &authority, Some(&owner), key(mode))
                .is_err()
        );
        assert!(std::fs::symlink_metadata(&link)
            .unwrap()
            .file_type()
            .is_symlink());
        assert_eq!(std::fs::read(&path).unwrap(), original);
    }
}

#[test]
fn file_relocation_refuses_unimplemented_access_tree_before_writes() {
    let mode = protection(0);
    let authority = Authority::Checksum;
    let source = packed_pair(mode, &authority, None);
    let mut anchor = publication::select(&source, archive(), mode, &authority)
        .unwrap()
        .anchor;
    anchor.keys = anchor.index;
    let replacement = SharedMemory::new(Vec::new());
    let result = allocation::compaction::relocate(
        &source,
        &anchor,
        replacement.clone(),
        &authority,
        None,
        None,
    );
    assert!(matches!(result, Err(Error::InvalidInput(_))));
    assert_eq!(replacement.operations(), 0);
}
