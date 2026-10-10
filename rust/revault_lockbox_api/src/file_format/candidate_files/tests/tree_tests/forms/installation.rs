//! No public CLI writes the experimental tree. Use private fixtures and real
//! files, then independently reopen every family through authenticated readers.
use super::*;
use std::path::{Path, PathBuf};

struct Directory(PathBuf);
impl Directory {
    fn new() -> Self {
        let mut random = [0; 8];
        getrandom::fill(&mut random).unwrap();
        let path = std::env::temp_dir().join(format!(
            "revault-tree-install-{}-{}",
            std::process::id(),
            u64::from_le_bytes(random)
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
fn protection(bits: usize) -> FormatMode {
    mode(bits & 1 != 0, bits & 2 != 0, bits & 4 != 0, bits & 8 != 0)
}
fn fixture(
    path: &Path,
    mode: FormatMode,
    authority: &Authority<'_>,
    owner: &OwnerSigningKeyPair,
) -> Vec<u8> {
    let (mut source, defs, _) = super::lifecycle::fixture(mode, authority, owner);
    let signer = mode.signed().then_some(owner);
    let mut updated = defs[0].clone();
    updated.revision += 1;
    updated.fields[0].label = "Current label".into();
    import_definition(
        &mut source,
        archive(),
        mode,
        authority,
        signer,
        key(mode),
        &updated,
    )
    .unwrap();
    let secret = SecretString::try_from_slice(&vec![b's'; 65537]).unwrap();
    tree_image::variables::set_secret_variable(
        &mut source,
        archive(),
        mode,
        authority,
        signer,
        key(mode),
        &VariableName::new("secret").unwrap(),
        &secret,
    )
    .unwrap();
    // Copy the completed fixture image, bypassing only the test read guard.
    // Production readers remain subject to guarded secret access.
    let bytes = source.inner.read_all().unwrap();
    drop(StorageBackend::create_file(path, &bytes).unwrap());
    bytes
}
fn verify(storage: &StorageBackend, mode: FormatMode, authority: &Authority<'_>) {
    let (report, definitions, records) =
        super::lifecycle::collect(storage, mode, authority).unwrap();
    assert_eq!(report, tree_image::forms::SalvageReport::default());
    assert_eq!(definitions.len(), 3);
    assert_eq!(records.len(), 3);
    assert!(definitions
        .iter()
        .any(|d| d.revision == 2 && d.fields[0].label == "Current label"));
    for (n, record) in records.iter().enumerate() {
        assert_eq!(record.path.as_str(), format!("/forms/{n}"));
        assert_eq!(record.definition_revision, 1);
        assert_eq!(record.values[0].captured_label, "Historical password");
        match &record.values[0].value {
            FormValue::Secret(secret) => secret
                .with_str(|s| assert_eq!(s, format!("secret {n}")))
                .unwrap(),
            _ => panic!("secret downgraded"),
        }
    }
    let mut image = TreeImage::open(
        crate::file_format::allocation_map::compaction::View(storage),
        archive(),
        mode,
        authority,
        key(mode),
    )
    .unwrap();
    image.image.verify_all().unwrap();
    assert_eq!(
        image
            .get_variable(&VariableName::new("retained").unwrap())
            .unwrap()
            .as_deref(),
        Some("retained variable")
    );
    image
        .with_secret_variable(&VariableName::new("secret").unwrap(), |s| {
            s.with_str(|s| assert_eq!(s.as_bytes(), vec![b's'; 65537]))
                .unwrap();
        })
        .unwrap()
        .unwrap();
    for (path, expected) in [
        (b"/docs/data".as_slice(), b"payload".as_slice()),
        (b"/docs/neighbor".as_slice(), b"neighbor".as_slice()),
    ] {
        let mut bytes = Vec::new();
        image
            .image
            .read_range(path, 0, expected.len() as u64, |b| {
                bytes.extend_from_slice(b);
                Ok(())
            })
            .unwrap();
        assert_eq!(bytes, expected);
    }
    assert!(!image.image.filesystem_metadata().unwrap().is_empty());
}
#[test]
fn whole_tree_path_install_reopens_all_families_and_remains_writable() {
    let directory = Directory::new();
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let public = owner.public_key();
    for bits in 0..16 {
        let mode = protection(bits);
        let authority = authority(mode, &public);
        let signer = mode.signed().then_some(&owner);
        let path = directory.0.join(format!("{bits}.tree"));
        let baseline = fixture(&path, mode, &authority, &owner);
        let source = StorageBackend::memory(baseline);
        let old = TreeImage::open(source, archive(), mode, &authority, key(mode))
            .unwrap()
            .image
            .anchor;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o640)).unwrap();
        }
        let mut compacted_len = None;
        for generation in 1..=2 {
            let installed =
                tree_image::compact_path(&path, archive(), mode, &authority, signer, key(mode))
                    .unwrap();
            assert_eq!(
                installed.image.anchor.generation,
                old.generation + generation
            );
            if generation == 1 {
                assert_eq!(
                    installed.image.anchor.previous,
                    shared::commitment(&old).unwrap()
                );
            }
            let len = installed.image.storage.len().unwrap();
            if let Some(previous) = compacted_len {
                assert_eq!(len, previous, "repeat compaction must not grow: {bits}");
            }
            compacted_len = Some(len);
            drop(installed);
            let storage = StorageBackend::file(&path).unwrap();
            verify(&storage, mode, &authority);
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                assert_eq!(
                    std::fs::metadata(&path).unwrap().permissions().mode() & 0o777,
                    0o640
                );
            }
        }
        let mut storage = StorageBackend::file_for_write(&path).unwrap();
        tree_image::variables::set_variable(
            &mut storage,
            archive(),
            mode,
            &authority,
            signer,
            key(mode),
            &VariableName::new("after").unwrap(),
            "writable",
        )
        .unwrap();
        drop(storage);
        let reopened = TreeImage::open(
            StorageBackend::file(&path).unwrap(),
            archive(),
            mode,
            &authority,
            key(mode),
        )
        .unwrap();
        assert_eq!(
            reopened
                .get_variable(&VariableName::new("after").unwrap())
                .unwrap()
                .as_deref(),
            Some("writable")
        );
    }
    assert_eq!(std::fs::read_dir(&directory.0).unwrap().count(), 16);
}

#[test]
fn whole_tree_path_install_returned_errors_preserve_publication_boundary() {
    let directory = Directory::new();
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let public = owner.public_key();
    for bits in [0, 3, 12, 15] {
        let mode = protection(bits);
        let authority = authority(mode, &public);
        for phase in [
            "tree-created",
            "tree-verified",
            "tree-installed",
            "tree-synced",
        ] {
            let path = directory.0.join(format!("{bits}-{phase}.tree"));
            let baseline = fixture(&path, mode, &authority, &owner);
            let result = tree_image::compact_path_with(
                &path,
                archive(),
                mode,
                &authority,
                mode.signed().then_some(&owner),
                key(mode),
                |at| {
                    if at == phase {
                        Err(Error::Io("synthetic path-install failure".into()))
                    } else {
                        Ok(())
                    }
                },
            );
            assert!(result.is_err());
            let storage = StorageBackend::file(&path).unwrap();
            verify(&storage, mode, &authority);
            if phase == "tree-created" || phase == "tree-verified" {
                assert_eq!(storage.read_all().unwrap(), baseline);
            } else {
                assert_ne!(storage.read_all().unwrap(), baseline);
            }
        }
    }
    assert_eq!(std::fs::read_dir(&directory.0).unwrap().count(), 16);
}

#[test]
fn whole_tree_path_install_refuses_stale_source_and_symlinks() {
    let directory = Directory::new();
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let public = owner.public_key();
    let mode = protection(15);
    let authority = authority(mode, &public);
    let path = directory.0.join("source.tree");
    let original = fixture(&path, mode, &authority, &owner);
    assert!(tree_image::compact_path(&path, archive(), mode, &authority, None, key(mode)).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), original);
    assert_eq!(std::fs::read_dir(&directory.0).unwrap().count(), 1);
    #[cfg(unix)]
    {
        let link = directory.0.join("link.tree");
        std::os::unix::fs::symlink(&path, &link).unwrap();
        assert!(tree_image::compact_path(
            &link,
            archive(),
            mode,
            &authority,
            Some(&owner),
            key(mode)
        )
        .is_err());
        std::fs::remove_file(link).unwrap();
    }
    let moved = directory.0.join("moved.tree");
    let result = tree_image::compact_path_with(
        &path,
        archive(),
        mode,
        &authority,
        Some(&owner),
        key(mode),
        |phase| {
            if phase == "tree-verified" {
                std::fs::rename(&path, &moved).unwrap();
                std::fs::write(&path, b"external replacement").unwrap();
            }
            Ok(())
        },
    );
    assert!(result.is_err());
    assert_eq!(std::fs::read(path).unwrap(), b"external replacement");
    assert_eq!(std::fs::read(moved).unwrap(), original);
    assert_eq!(std::fs::read_dir(&directory.0).unwrap().count(), 2);
}

#[test]
fn whole_tree_path_install_does_not_erase_replaced_temporary_path() {
    let directory = Directory::new();
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let public = owner.public_key();
    let mode = protection(15);
    let authority = authority(mode, &public);
    let path = directory.0.join("source.tree");
    let original = fixture(&path, mode, &authority, &owner);
    let displaced = directory.0.join("displaced.tree");
    let mut substituted = None;
    let result = tree_image::compact_path_with(
        &path,
        archive(),
        mode,
        &authority,
        Some(&owner),
        key(mode),
        |phase| {
            if phase == "tree-verified" {
                let temporary = std::fs::read_dir(&directory.0)
                    .unwrap()
                    .map(|e| e.unwrap().path())
                    .find(|p| {
                        p.file_name()
                            .unwrap()
                            .to_string_lossy()
                            .starts_with(".source.tree.compact-")
                    })
                    .unwrap();
                std::fs::rename(&temporary, &displaced).unwrap();
                std::fs::write(&temporary, b"do not erase substitute").unwrap();
                substituted = Some(temporary);
            }
            Ok(())
        },
    );
    assert!(
        matches!(result, Err(Error::InvalidOperation(ref message)) if message.contains("temporary cleanup failed"))
    );
    assert_eq!(std::fs::read(&path).unwrap(), original);
    assert_eq!(
        std::fs::read(substituted.unwrap()).unwrap(),
        b"do not erase substitute"
    );
    verify(&StorageBackend::file(displaced).unwrap(), mode, &authority);
}

#[test]
fn whole_tree_path_install_process_child() {
    let Some(path) = std::env::var_os("REVAULT_TREE_INSTALL_PATH") else {
        return;
    };
    let path = PathBuf::from(path);
    let bits: usize = std::env::var("REVAULT_TREE_INSTALL_MODE")
        .unwrap()
        .parse()
        .unwrap();
    let mode = protection(bits);
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let public = owner.public_key();
    let authority = authority(mode, &public);
    let baseline = fixture(&path, mode, &authority, &owner);
    std::fs::write(path.with_extension("base"), baseline).unwrap();
    std::fs::write(path.with_extension("public"), public.to_bytes()).unwrap();
    tree_image::compact_path(
        &path,
        archive(),
        mode,
        &authority,
        mode.signed().then_some(&owner),
        key(mode),
    )
    .unwrap();
    panic!("did not reach requested process-death checkpoint");
}

#[test]
fn whole_tree_path_install_process_death_reopens_old_or_complete_new() {
    use std::process::{Command, Stdio};
    use std::time::{Duration, Instant};
    let directory = Directory::new();
    let mut cases = 0;
    for bits in 0..16 {
        for phase in [
            "tree-created",
            "tree-copying",
            "tree-dependencies",
            "tree-verified",
            "tree-installed",
            "tree-synced",
        ] {
            let path = directory.0.join(format!("{bits}-{phase}.tree"));
            let mut child = Command::new(std::env::current_exe().unwrap())
                .args(["--exact", "file_format::candidate_files::tests::tree_tests::forms::installation::whole_tree_path_install_process_child", "--nocapture"])
                .env("REVAULT_TREE_INSTALL_PATH", &path).env("REVAULT_TREE_INSTALL_MODE", bits.to_string())
                .env("REVAULT_CANDIDATE_COMPACTION_EXIT", phase)
                .stdout(Stdio::null()).stderr(Stdio::inherit()).spawn().unwrap();
            let deadline = Instant::now() + Duration::from_secs(60);
            let status = loop {
                if let Some(status) = child.try_wait().unwrap() {
                    break status;
                }
                if Instant::now() >= deadline {
                    let _ = child.kill();
                    let _ = child.wait();
                    panic!("child timeout: {bits}/{phase}");
                }
                std::thread::sleep(Duration::from_millis(10));
            };
            assert_eq!(status.code(), Some(71), "{bits}/{phase}");
            let mode = protection(bits);
            let public = crate::OwnerSigningPublicKey::from_bytes(
                &std::fs::read(path.with_extension("public")).unwrap(),
            )
            .unwrap();
            let authority = authority(mode, &public);
            let baseline = std::fs::read(path.with_extension("base")).unwrap();
            let old = TreeImage::open(
                StorageBackend::memory(baseline.clone()),
                archive(),
                mode,
                &authority,
                key(mode),
            )
            .unwrap()
            .image
            .anchor;
            let storage = StorageBackend::file(&path).unwrap();
            verify(&storage, mode, &authority);
            let actual = TreeImage::open(
                crate::file_format::allocation_map::compaction::View(&storage),
                archive(),
                mode,
                &authority,
                key(mode),
            )
            .unwrap()
            .image
            .anchor;
            if phase == "tree-installed" || phase == "tree-synced" {
                assert_eq!(actual.generation, old.generation + 1);
                assert_eq!(actual.previous, shared::commitment(&old).unwrap());
            } else {
                assert_eq!(storage.read_all().unwrap(), baseline);
                assert_eq!(actual, old);
            }
            cases += 1;
        }
    }
    println!("WHOLE_TREE_PATH_PROCESS_DEATH_CASES {cases}");
}

fn completed_copy(
    path: &Path,
    candidate: &Path,
    mode: FormatMode,
    authority: &Authority<'_>,
    owner: &OwnerSigningKeyPair,
) {
    let source = StorageBackend::file(path).unwrap();
    let destination = StorageBackend::create_file(candidate, &[]).unwrap();
    drop(
        tree_image::compact(
            &source,
            destination,
            archive(),
            mode,
            authority,
            mode.signed().then_some(owner),
            key(mode),
        )
        .unwrap(),
    );
}

#[test]
fn whole_tree_resume_complete_copy_reopens_all_modes() {
    let directory = Directory::new();
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let public = owner.public_key();
    for bits in 0..16 {
        let mode = protection(bits);
        let authority = authority(mode, &public);
        let path = directory.0.join(format!("{bits}.tree"));
        let candidate = directory.0.join(format!("{bits}.candidate"));
        fixture(&path, mode, &authority, &owner);
        completed_copy(&path, &candidate, mode, &authority, &owner);
        let expected = std::fs::read(&candidate).unwrap();
        drop(
            tree_image::resume_path(&path, &candidate, archive(), mode, &authority, key(mode))
                .unwrap(),
        );
        assert!(!candidate.exists());
        assert_eq!(std::fs::read(&path).unwrap(), expected);
        verify(&StorageBackend::file(&path).unwrap(), mode, &authority);
        assert!(
            tree_image::resume_path(&path, &candidate, archive(), mode, &authority, key(mode))
                .is_err()
        );
        assert_eq!(std::fs::read(&path).unwrap(), expected);
    }
}

#[test]
fn whole_tree_resume_refuses_partial_stale_edited_and_corrupt_candidates() {
    let directory = Directory::new();
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let public = owner.public_key();
    for bits in [0, 3, 12, 15] {
        let mode = protection(bits);
        let authority = authority(mode, &public);
        for case in ["partial", "stale", "edited", "corrupt", "trailing"] {
            let path = directory.0.join(format!("{bits}-{case}.tree"));
            let candidate = directory.0.join(format!("{bits}-{case}.candidate"));
            fixture(&path, mode, &authority, &owner);
            if case == "edited" {
                // A real authenticated direct successor is insufficient: an
                // ordinary variable edit must not be installed as compaction.
                std::fs::copy(&path, &candidate).unwrap();
                let mut storage = StorageBackend::file_for_write(&candidate).unwrap();
                tree_image::variables::set_variable(
                    &mut storage,
                    archive(),
                    mode,
                    &authority,
                    mode.signed().then_some(&owner),
                    key(mode),
                    &VariableName::new("retained").unwrap(),
                    "different",
                )
                .unwrap();
            } else {
                completed_copy(&path, &candidate, mode, &authority, &owner);
            }
            if case == "stale" {
                let mut storage = StorageBackend::file_for_write(&path).unwrap();
                tree_image::variables::set_variable(
                    &mut storage,
                    archive(),
                    mode,
                    &authority,
                    mode.signed().then_some(&owner),
                    key(mode),
                    &VariableName::new("retained").unwrap(),
                    "new source",
                )
                .unwrap();
            } else if case == "partial" {
                std::fs::OpenOptions::new()
                    .write(true)
                    .open(&candidate)
                    .unwrap()
                    .set_len(4096)
                    .unwrap();
            } else if case == "corrupt" {
                let mut bytes = std::fs::read(&candidate).unwrap();
                bytes[crate::file_format::publication_anchor::REGION_LEN] ^= 1;
                std::fs::write(&candidate, bytes).unwrap();
            } else if case == "trailing" {
                use std::io::Write;
                std::fs::OpenOptions::new()
                    .append(true)
                    .open(&candidate)
                    .unwrap()
                    .write_all(b"unowned tail")
                    .unwrap();
            }
            let source_before = std::fs::read(&path).unwrap();
            let candidate_before = std::fs::read(&candidate).unwrap();
            assert!(
                tree_image::resume_path(&path, &candidate, archive(), mode, &authority, key(mode))
                    .is_err(),
                "{bits}/{case}"
            );
            assert_eq!(std::fs::read(&path).unwrap(), source_before);
            assert_eq!(std::fs::read(&candidate).unwrap(), candidate_before);
        }
    }
}

#[test]
fn whole_tree_resume_refuses_same_file_links_and_foreign_directory() {
    let directory = Directory::new();
    let other = Directory::new();
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let public = owner.public_key();
    let mode = protection(0);
    let authority = authority(mode, &public);
    let path = directory.0.join("source");
    let original = fixture(&path, mode, &authority, &owner);
    assert!(tree_image::resume_path(&path, &path, archive(), mode, &authority, key(mode)).is_err());
    let candidate = other.0.join("candidate");
    completed_copy(&path, &candidate, mode, &authority, &owner);
    assert!(
        tree_image::resume_path(&path, &candidate, archive(), mode, &authority, key(mode)).is_err()
    );
    #[cfg(unix)]
    {
        let link = directory.0.join("link");
        std::os::unix::fs::symlink(&path, &link).unwrap();
        assert!(
            tree_image::resume_path(&path, &link, archive(), mode, &authority, key(mode)).is_err()
        );
        std::fs::remove_file(&link).unwrap();
        std::fs::hard_link(&path, &link).unwrap();
        assert!(
            tree_image::resume_path(&path, &link, archive(), mode, &authority, key(mode)).is_err()
        );
    }
    assert_eq!(std::fs::read(&path).unwrap(), original);
}

#[test]
fn whole_tree_resume_process_child() {
    let Some(path) = std::env::var_os("REVAULT_TREE_RESUME_PATH") else {
        return;
    };
    let path = PathBuf::from(path);
    let bits: usize = std::env::var("REVAULT_TREE_RESUME_MODE")
        .unwrap()
        .parse()
        .unwrap();
    let mode = protection(bits);
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let public = owner.public_key();
    let authority = authority(mode, &public);
    let baseline = fixture(&path, mode, &authority, &owner);
    std::fs::write(path.with_extension("base"), baseline).unwrap();
    std::fs::write(path.with_extension("public"), public.to_bytes()).unwrap();
    let candidate = path.with_extension("candidate");
    completed_copy(&path, &candidate, mode, &authority, &owner);
    std::fs::copy(&candidate, path.with_extension("expected")).unwrap();
    tree_image::resume_path(&path, &candidate, archive(), mode, &authority, key(mode)).unwrap();
    panic!("missed resume checkpoint");
}

#[test]
fn whole_tree_resume_process_death_reopens_and_retries_before_rename() {
    use std::process::{Command, Stdio};
    use std::time::{Duration, Instant};
    let directory = Directory::new();
    let mut cases = 0;
    for bits in 0..16 {
        for phase in [
            "tree-resume-verified",
            "tree-resume-installed",
            "tree-resume-synced",
        ] {
            let path = directory.0.join(format!("{bits}-{phase}.tree"));
            let mut child = Command::new(std::env::current_exe().unwrap())
                .args(["--exact", "file_format::candidate_files::tests::tree_tests::forms::installation::whole_tree_resume_process_child", "--nocapture"])
                .env("REVAULT_TREE_RESUME_PATH", &path).env("REVAULT_TREE_RESUME_MODE", bits.to_string())
                .env("REVAULT_CANDIDATE_COMPACTION_EXIT", phase)
                .stdout(Stdio::null()).stderr(Stdio::inherit()).spawn().unwrap();
            let deadline = Instant::now() + Duration::from_secs(60);
            let status = loop {
                if let Some(status) = child.try_wait().unwrap() {
                    break status;
                }
                if Instant::now() >= deadline {
                    let _ = child.kill();
                    let _ = child.wait();
                    panic!("resume child timeout");
                }
                std::thread::sleep(Duration::from_millis(10));
            };
            assert_eq!(status.code(), Some(71), "{bits}/{phase}");
            let mode = protection(bits);
            let public = crate::OwnerSigningPublicKey::from_bytes(
                &std::fs::read(path.with_extension("public")).unwrap(),
            )
            .unwrap();
            let authority = authority(mode, &public);
            let candidate = path.with_extension("candidate");
            if phase == "tree-resume-verified" {
                assert_eq!(
                    std::fs::read(&path).unwrap(),
                    std::fs::read(path.with_extension("base")).unwrap()
                );
                verify(&StorageBackend::file(&path).unwrap(), mode, &authority);
                drop(
                    tree_image::resume_path(
                        &path,
                        &candidate,
                        archive(),
                        mode,
                        &authority,
                        key(mode),
                    )
                    .unwrap(),
                );
            }
            assert!(!candidate.exists());
            assert_eq!(
                std::fs::read(&path).unwrap(),
                std::fs::read(path.with_extension("expected")).unwrap()
            );
            verify(&StorageBackend::file(&path).unwrap(), mode, &authority);
            cases += 1;
        }
    }
    println!("WHOLE_TREE_RESUME_PROCESS_CASES {cases}");
}

// No public writer creates a shared-tree access root. Compact the synthetic
// fixture first to remove aged free claims, then initialize only its control
// state with the real bootstrap encoder and the unchanged selected manifest.
fn attach_access(
    path: &Path,
    mode: FormatMode,
    authority: &Authority<'_>,
    owner: &OwnerSigningKeyPair,
    slots: &[crate::key_slot::KeySlot],
) {
    drop(
        tree_image::compact_path(
            path,
            archive(),
            mode,
            authority,
            mode.signed().then_some(owner),
            key(mode),
        )
        .unwrap(),
    );
    let mut storage = StorageBackend::file_for_write(path).unwrap();
    let (_, manifest) =
        shared::open_private(&storage, archive(), mode, authority, key(mode)).unwrap();
    storage
        .write_at(
            0,
            &vec![0; crate::file_format::publication_anchor::REGION_LEN],
        )
        .unwrap();
    shared::initialize(
        &mut storage,
        archive(),
        mode,
        authority,
        mode.signed().then_some(owner),
        key(mode),
        &manifest,
        slots,
    )
    .unwrap();
}

#[test]
fn whole_tree_resume_preserves_credentials_and_rejects_changed_wrappers() {
    let directory = Directory::new();
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let public = owner.public_key();
    let password = SecretString::try_from_slice(b"synthetic resume password").unwrap();
    for bits in [1, 3, 5, 7, 9, 11, 13, 15] {
        let mode = protection(bits);
        let authority = authority(mode, &public);
        let path = directory.0.join(format!("{bits}.tree"));
        let candidate = directory.0.join(format!("{bits}.candidate"));
        fixture(&path, mode, &authority, &owner);
        let slot = crate::key_slot::KeySlot::password_bytes(
            1,
            b"synthetic resume password",
            vec![93; 16],
            key(mode).unwrap(),
        )
        .unwrap();
        attach_access(&path, mode, &authority, &owner, &[slot]);
        completed_copy(&path, &candidate, mode, &authority, &owner);
        drop(
            tree_image::resume_path(&path, &candidate, archive(), mode, &authority, key(mode))
                .unwrap(),
        );
        let storage = StorageBackend::file(&path).unwrap();
        let credential =
            crate::file_format::publication_anchor::bootstrap::Credential::Password(&password);
        let opened = shared::credential_open(
            &storage,
            archive(),
            mode,
            mode.signed().then_some(&public),
            credential,
            Some(1),
        )
        .unwrap();
        opened
            .key
            .with_bytes(|k| assert_eq!(k, key(mode).unwrap()))
            .unwrap();
        verify(&storage, mode, &authority);
        drop(storage);
        // A valid successor with the same payloads but different credentials is
        // not the same compaction. Re-sign only synthetic candidate control state.
        completed_copy(&path, &candidate, mode, &authority, &owner);
        let changed = crate::key_slot::KeySlot::password_bytes(
            1,
            b"different password",
            vec![94; 16],
            key(mode).unwrap(),
        )
        .unwrap();
        let mut replacement = StorageBackend::file_for_write(&candidate).unwrap();
        let old_anchor = TreeImage::open(
            crate::file_format::allocation_map::compaction::View(&replacement),
            archive(),
            mode,
            &authority,
            key(mode),
        )
        .unwrap()
        .image
        .anchor;
        let changed_anchor = shared::tests::replace_fixture_slots(
            &mut replacement,
            archive(),
            mode,
            &authority,
            mode.signed().then_some(&owner),
            key(mode),
            &[changed],
        )
        .unwrap();
        assert_eq!(changed_anchor.generation, old_anchor.generation);
        assert_eq!(changed_anchor.previous, old_anchor.previous);
        let different = SecretString::try_from_slice(b"different password").unwrap();
        let opened = shared::credential_open(
            &replacement,
            archive(),
            mode,
            mode.signed().then_some(&public),
            crate::file_format::publication_anchor::bootstrap::Credential::Password(&different),
            Some(1),
        )
        .unwrap();
        opened
            .key
            .with_bytes(|k| assert_eq!(k, key(mode).unwrap()))
            .unwrap();
        drop(replacement);
        let before = std::fs::read(&path).unwrap();
        let pending = std::fs::read(&candidate).unwrap();
        assert!(
            tree_image::resume_path(&path, &candidate, archive(), mode, &authority, key(mode))
                .is_err()
        );
        assert_eq!(std::fs::read(&path).unwrap(), before);
        assert_eq!(std::fs::read(&candidate).unwrap(), pending);
    }
}

fn check_credential_reader<S: Storage>(mut opened: TreeImage<S>) {
    assert_eq!(
        opened
            .get_variable(&VariableName::new("retained").unwrap())
            .unwrap()
            .as_deref(),
        Some("retained variable")
    );
    opened
        .with_secret_variable(&VariableName::new("secret").unwrap(), |secret| {
            secret
                .with_str(|value| assert_eq!(value.as_bytes(), vec![b's'; 65537]))
                .unwrap();
        })
        .unwrap()
        .unwrap();
    let record = opened
        .get_form_record(&LockboxPath::new("/forms/0").unwrap())
        .unwrap()
        .unwrap();
    assert_eq!(record.definition_revision, 1);
    assert_eq!(record.values[0].captured_label, "Historical password");
    match &record.values[0].value {
        FormValue::Secret(secret) => secret
            .with_str(|value| assert_eq!(value, "secret 0"))
            .unwrap(),
        _ => panic!("secret form field downgraded"),
    }
    let latest = opened
        .get_form_definition(&record.type_id, 2)
        .unwrap()
        .unwrap();
    assert_eq!(latest.fields[0].label, "Current label");
    let mut bytes = Vec::new();
    opened
        .image
        .read_range(b"/docs/data", 0, 7, |part| {
            bytes.extend_from_slice(part);
            Ok(())
        })
        .unwrap();
    assert_eq!(bytes, b"payload");
    assert!(!opened.image.filesystem_metadata().unwrap().is_empty());
}

#[test]
fn typed_credential_open_reads_all_families_and_preserves_refusals() {
    let directory = Directory::new();
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let public = owner.public_key();
    let wrong_owner = OwnerSigningKeyPair::generate().unwrap().public_key();
    let password = SecretString::try_from_slice(b"synthetic typed-reader password").unwrap();
    let wrong = SecretString::try_from_slice(b"wrong typed-reader password").unwrap();
    let contact = crate::ContactKeyPair::generate().unwrap();
    for bits in [1, 3, 5, 7, 9, 11, 13, 15] {
        let mode = protection(bits);
        let authority = authority(mode, &public);
        let path = directory.0.join(format!("{bits}.tree"));
        fixture(&path, mode, &authority, &owner);
        let slots = [
            crate::key_slot::KeySlot::password_bytes(
                1,
                b"synthetic typed-reader password",
                vec![67; 16],
                key(mode).unwrap(),
            )
            .unwrap(),
            crate::key_slot::KeySlot::hybrid_contact(2, &contact.public_key(), key(mode).unwrap())
                .unwrap(),
        ];
        attach_access(&path, mode, &authority, &owner, &slots);
        let before = std::fs::read(&path).unwrap();
        for (credential, slot) in [
            (publication::bootstrap::Credential::Password(&password), 1),
            (publication::bootstrap::Credential::Contact(&contact), 2),
        ] {
            let opened = TreeImage::open_credential(
                StorageBackend::file(&path).unwrap(),
                archive(),
                mode,
                mode.signed().then_some(&public),
                credential,
                Some(slot),
            )
            .unwrap();
            check_credential_reader(opened);
            assert!(TreeImage::open_credential(
                StorageBackend::file(&path).unwrap(),
                archive(),
                mode,
                mode.signed().then_some(&public),
                credential,
                Some(99)
            )
            .is_err());
            assert!(TreeImage::open_credential(
                StorageBackend::file(&path).unwrap(),
                archive(),
                mode,
                Some(&wrong_owner),
                credential,
                Some(slot)
            )
            .is_err());
        }
        assert!(TreeImage::open_credential(
            StorageBackend::file(&path).unwrap(),
            archive(),
            mode,
            mode.signed().then_some(&public),
            publication::bootstrap::Credential::Password(&wrong),
            Some(1)
        )
        .is_err());
        assert_eq!(std::fs::read(&path).unwrap(), before);
    }
}

#[derive(Clone, Debug)]
struct BootstrapSwitch {
    original: StorageBackend,
    replacement: StorageBackend,
    selections: std::rc::Rc<std::cell::Cell<usize>>,
}
impl BootstrapSwitch {
    fn storage(&self) -> &StorageBackend {
        if self.selections.get() >= 3 {
            &self.replacement
        } else {
            &self.original
        }
    }
    fn read_storage(&self, at: u64) -> &StorageBackend {
        if at == 0 {
            self.selections.set(self.selections.get() + 1);
        }
        self.storage()
    }
}
impl Storage for BootstrapSwitch {
    fn len(&self) -> Result<u64> {
        self.storage().len()
    }
    fn read_at(&self, at: u64, len: usize) -> Result<Vec<u8>> {
        self.read_storage(at).read_at(at, len)
    }
    fn read_at_into(&self, at: u64, out: &mut [u8]) -> Result<()> {
        self.read_storage(at).read_at_into(at, out)
    }
    fn append(&mut self, _: &[u8]) -> Result<u64> {
        panic!("credential opening must not write")
    }
    fn write_at(&mut self, _: u64, _: &[u8]) -> Result<()> {
        panic!("credential opening must not write")
    }
    fn truncate(&mut self, _: u64) -> Result<()> {
        panic!("credential opening must not truncate")
    }
    fn sync(&self) -> Result<()> {
        panic!("credential opening must not sync")
    }
}

#[test]
fn typed_credential_open_rejects_publication_switch_after_bootstrap() {
    let directory = Directory::new();
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let public = owner.public_key();
    let password = SecretString::try_from_slice(b"synthetic bootstrap switch").unwrap();
    for bits in [1, 3, 13, 15] {
        let mode = protection(bits);
        let authority = authority(mode, &public);
        let path = directory.0.join(format!("{bits}.tree"));
        fixture(&path, mode, &authority, &owner);
        let slot = crate::key_slot::KeySlot::password_bytes(
            1,
            b"synthetic bootstrap switch",
            vec![68; 16],
            key(mode).unwrap(),
        )
        .unwrap();
        attach_access(&path, mode, &authority, &owner, &[slot]);
        let original = StorageBackend::memory(std::fs::read(&path).unwrap());
        let replacement = tree_image::compact(
            &original,
            StorageBackend::memory(Vec::new()),
            archive(),
            mode,
            &authority,
            mode.signed().then_some(&owner),
            key(mode),
        )
        .unwrap();
        // Both snapshots independently open with the same credential and records.
        // Only switching after bootstrap is rejected: no public CLI can simulate
        // a backend changing publication inside one read-only open operation.
        for storage in [&original, &replacement] {
            check_credential_reader(
                TreeImage::open_credential(
                    allocation::compaction::View(storage),
                    archive(),
                    mode,
                    mode.signed().then_some(&public),
                    publication::bootstrap::Credential::Password(&password),
                    Some(1),
                )
                .unwrap(),
            );
        }
        let selections = std::rc::Rc::new(std::cell::Cell::new(0));
        let switching = BootstrapSwitch {
            original,
            replacement,
            selections: selections.clone(),
        };
        assert!(matches!(
            TreeImage::open_credential(
                switching,
                archive(),
                mode,
                mode.signed().then_some(&public),
                publication::bootstrap::Credential::Password(&password),
                Some(1)
            ),
            Err(Error::CorruptRecord)
        ));
        assert!(selections.get() >= 3);
    }
}
