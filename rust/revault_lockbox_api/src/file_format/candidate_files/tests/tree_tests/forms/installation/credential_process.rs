//! Process-death coverage uses only synthetic passwords and parent-held contact
//! private keys. Child processes receive public keys, never private key files.
use super::*;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

const PASSWORD: &[u8] = b"synthetic tree credential crash password";

#[test]
fn credential_install_process_child() {
    let Some(path) = std::env::var_os("REVAULT_CREDENTIAL_CRASH_PATH") else {
        return;
    };
    let path = PathBuf::from(path);
    let bits: usize = std::env::var("REVAULT_CREDENTIAL_CRASH_MODE")
        .unwrap()
        .parse()
        .unwrap();
    let phase = std::env::var("REVAULT_CREDENTIAL_CRASH_PHASE").unwrap();
    let mode = protection(bits);
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let public = owner.public_key();
    let authority = authority(mode, &public);
    let contact = crate::ContactPublicKey::from_bytes(
        &std::fs::read(path.with_extension("contact-public")).unwrap(),
    )
    .unwrap();
    fixture(&path, mode, &authority, &owner);
    let slots = [
        crate::key_slot::KeySlot::password_bytes(1, PASSWORD, vec![73; 16], key(mode).unwrap())
            .unwrap(),
        crate::key_slot::KeySlot::hybrid_contact(2, &contact, key(mode).unwrap()).unwrap(),
    ];
    attach_access(&path, mode, &authority, &owner, &slots);
    std::fs::copy(&path, path.with_extension("base")).unwrap();
    std::fs::write(path.with_extension("public"), public.to_bytes()).unwrap();
    let candidate = path.with_extension("candidate");
    if phase.starts_with("tree-resume-") {
        completed_copy(&path, &candidate, mode, &authority, &owner);
        std::fs::copy(&candidate, path.with_extension("expected")).unwrap();
    }
    // Set the exit checkpoint only after preparing the fixture: preparation
    // itself uses compaction, which must not consume the intended interruption.
    std::env::set_var("REVAULT_CANDIDATE_COMPACTION_EXIT", &phase);
    if phase.starts_with("tree-resume-") {
        tree_image::resume_path(&path, &candidate, archive(), mode, &authority, key(mode)).unwrap();
    } else {
        tree_image::compact_path(
            &path,
            archive(),
            mode,
            &authority,
            mode.signed().then_some(&owner),
            key(mode),
        )
        .unwrap();
    }
    panic!("did not reach credential crash checkpoint");
}

fn credential_check(
    path: &Path,
    mode: FormatMode,
    public: &crate::OwnerSigningPublicKey,
    password: &SecretString,
    contact: &crate::ContactKeyPair,
) {
    let before = std::fs::read(path).unwrap();
    for (credential, id) in [
        (publication::bootstrap::Credential::Password(password), 1),
        (publication::bootstrap::Credential::Contact(contact), 2),
    ] {
        let opened = tree_image::TreeImage::open_credential(
            StorageBackend::file(path).unwrap(),
            archive(),
            mode,
            mode.signed().then_some(public),
            credential,
            Some(id),
        )
        .unwrap();
        check_credential_reader(opened);
    }
    assert_eq!(std::fs::read(path).unwrap(), before);
}

#[test]
fn credential_install_process_death_preserves_password_contact_and_resume() {
    let directory = Directory::new();
    let contact = crate::ContactKeyPair::generate().unwrap();
    let password = SecretString::try_from_slice(PASSWORD).unwrap();
    let mut cases = 0;
    for bits in [1, 3, 5, 7, 9, 11, 13, 15] {
        for phase in [
            "tree-created",
            "tree-copying",
            "tree-dependencies",
            "tree-verified",
            "tree-installed",
            "tree-synced",
            "tree-resume-verified",
            "tree-resume-installed",
            "tree-resume-synced",
        ] {
            let path = directory.0.join(format!("{bits}-{phase}.tree"));
            std::fs::write(
                path.with_extension("contact-public"),
                contact.public_key().to_bytes(),
            )
            .unwrap();
            let mut child=Command::new(std::env::current_exe().unwrap())
                .args(["--exact","file_format::candidate_files::tests::tree_tests::forms::installation::credential_process::credential_install_process_child","--nocapture"])
                .env("REVAULT_CREDENTIAL_CRASH_PATH",&path)
                .env("REVAULT_CREDENTIAL_CRASH_MODE",bits.to_string())
                .env("REVAULT_CREDENTIAL_CRASH_PHASE",phase)
                .env_remove("REVAULT_CANDIDATE_COMPACTION_EXIT")
                .stdout(Stdio::null()).stderr(Stdio::inherit()).spawn().unwrap();
            let deadline = Instant::now() + Duration::from_secs(60);
            let status = loop {
                if let Some(status) = child.try_wait().unwrap() {
                    break status;
                }
                if Instant::now() >= deadline {
                    let _ = child.kill();
                    let _ = child.wait();
                    panic!("credential child timeout {bits}/{phase}");
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
            credential_check(&path, mode, &public, &password, &contact);
            let base = StorageBackend::memory(std::fs::read(path.with_extension("base")).unwrap());
            let old = shared::open_private(&base, archive(), mode, &authority, key(mode))
                .unwrap()
                .0;
            let current = StorageBackend::file(&path).unwrap();
            let actual = shared::open_private(&current, archive(), mode, &authority, key(mode))
                .unwrap()
                .0;
            assert_eq!(
                shared::retained_public_directory(&current, &actual).unwrap(),
                shared::retained_public_directory(&base, &old).unwrap()
            );
            let installed = phase.ends_with("installed") || phase.ends_with("synced");
            if installed {
                assert_eq!(actual.generation, old.generation + 1);
                assert_eq!(actual.previous, shared::commitment(&old).unwrap());
            } else {
                assert_eq!(current.read_all().unwrap(), base.read_all().unwrap());
            }
            drop(current);
            if phase == "tree-resume-verified" {
                let candidate = path.with_extension("candidate");
                credential_check(&candidate, mode, &public, &password, &contact);
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
                credential_check(&path, mode, &public, &password, &contact);
                assert!(!candidate.exists());
            }
            if phase.starts_with("tree-resume-") {
                assert_eq!(
                    std::fs::read(&path).unwrap(),
                    std::fs::read(path.with_extension("expected")).unwrap()
                );
            }
            cases += 1;
        }
    }
    assert_eq!(cases, 72);
    println!("CREDENTIAL_INSTALL_PROCESS_DEATH_CASES {cases}");
}
