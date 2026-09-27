//! Component fixtures use internal storage because this experimental bootstrap has
//! no public CLI writer. Private metadata is separately decrypted after bootstrap.
use super::*;
use crate::file_format::authenticated_index::{Entry, Index};
use crate::storage::StorageBackend;
use crate::{Compression, EncryptionMode, LockboxFormatOptions, SigningMode, SizePadding};

const KEY: [u8; 32] = [71; 32];
fn archive() -> LockboxId {
    LockboxId::from_bytes([42; 16])
}
fn mode(signed: bool) -> FormatMode {
    FormatMode::new(LockboxFormatOptions {
        encryption: EncryptionMode::ChaCha20Poly1305,
        signing: if signed {
            SigningMode::Owner
        } else {
            SigningMode::None
        },
        compression: Compression::None,
        size_padding: SizePadding::Default,
    })
}
fn password() -> SecretString {
    SecretString::try_from_slice(b"synthetic bootstrap password").unwrap()
}
fn slot(key: &[u8]) -> KeySlot {
    KeySlot::password_bytes(1, b"synthetic bootstrap password", vec![17; 16], key).unwrap()
}
fn roots(storage: &mut StorageBackend, bytes: &[u8]) -> RootRef {
    let mut append = || {
        let gap = (FAILURE_REGION - storage.len().unwrap() % FAILURE_REGION) % FAILURE_REGION;
        storage.append(&vec![0; gap as usize]).unwrap();
        storage.append(bytes).unwrap()
    };
    RootRef {
        primary: append(),
        mirror: append(),
        len: bytes.len() as u64,
        digest: strong_checksum(bytes),
    }
}
fn stage(
    storage: &mut StorageBackend,
    mode: FormatMode,
    key: &[u8],
    slots: &[KeySlot],
    old: Option<&Anchor>,
) -> Anchor {
    let generation = old.map_or(1, |old| old.generation + 1);
    let keys = roots(storage, &directory(archive(), generation, slots).unwrap());
    let mut temporary = StorageBackend::memory(vec![0; REGION_LEN]);
    let index = Index::new(archive(), mode, Some(key)).unwrap();
    let root = index
        .build_sorted(
            &mut temporary,
            [Entry::new(1, b"private name", b"private contents")],
        )
        .unwrap()
        .root;
    let bytes = root.read_verified(&temporary).unwrap();
    let index = roots(storage, &bytes);
    Anchor {
        archive: archive(),
        generation,
        mode,
        sealed_len: storage.len().unwrap(),
        object_root: index.digest,
        previous: old.map_or([0; 32], |a| a.commitment().unwrap()),
        index,
        allocation: RootRef::default(),
        keys,
    }
}
fn publish(
    storage: &mut StorageBackend,
    anchor: &Anchor,
    key: &[u8],
    owner: &OwnerSigningKeyPair,
    both: bool,
) {
    let public = owner.public_key();
    let authority = if anchor.mode.signed() {
        Authority::Owner(&public)
    } else {
        Authority::Symmetric(key)
    };
    anchor.verify_dependencies(storage).unwrap();
    let encoded = encode(anchor, &authority, anchor.mode.signed().then_some(owner)).unwrap();
    storage.write_at(0, &encoded).unwrap();
    if both {
        storage.write_at(SLOT_STRIDE as u64, &encoded).unwrap();
    }
    storage.sync().unwrap();
}
fn verify_private(storage: &StorageBackend, opened: &Opened, expected: &[u8]) {
    opened
        .key
        .with_bytes(|key| {
            assert_eq!(key, expected);
            let index = Index::new(archive(), opened.anchor.mode, Some(key)).unwrap();
            let entry = index
                .get(
                    storage,
                    opened.anchor.index,
                    opened.anchor.sealed_len,
                    1,
                    b"private name",
                )
                .unwrap()
                .unwrap();
            assert_eq!(entry.value.as_slice(), b"private contents");
        })
        .unwrap();
}
#[test]
fn password_and_contact_bootstrap_authenticate_before_private_metadata() {
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let public = owner.public_key();
    let contact = ContactKeyPair::generate().unwrap();
    let stranger = ContactKeyPair::generate().unwrap();
    let slots = [
        slot(&KEY),
        KeySlot::hybrid_contact(2, &contact.public_key(), &KEY).unwrap(),
    ];
    let password = password();
    let wrong = SecretString::try_from_slice(b"wrong synthetic password").unwrap();
    for signed in [false, true] {
        let mode = mode(signed);
        let mut storage = StorageBackend::memory(vec![0; REGION_LEN]);
        let anchor = stage(&mut storage, mode, &KEY, &slots, None);
        publish(&mut storage, &anchor, &KEY, &owner, true);
        let before = storage.read_all().unwrap();
        let owner = signed.then_some(&public);
        for (credential, id) in [
            (Credential::Password(&password), 1),
            (Credential::Contact(&contact), 2),
        ] {
            let opened = open(&storage, archive(), mode, owner, credential, Some(id)).unwrap();
            assert_eq!(opened.anchor, anchor);
            verify_private(&storage, &opened, &KEY);
            assert!(open(&storage, archive(), mode, owner, credential, Some(99)).is_err());
        }
        assert!(open(
            &storage,
            archive(),
            mode,
            owner,
            Credential::Password(&wrong),
            None
        )
        .is_err());
        assert!(open(
            &storage,
            archive(),
            mode,
            owner,
            Credential::Contact(&stranger),
            None
        )
        .is_err());
        assert_eq!(storage.read_all().unwrap(), before);
        // Removing all private metadata does not prevent authenticating the key
        // directory; reading private objects is a separate, failing operation.
        storage.truncate(anchor.index.primary).unwrap();
        assert!(open(
            &storage,
            archive(),
            mode,
            owner,
            Credential::Password(&password),
            None
        )
        .is_ok());
        assert!(anchor.index.read_verified(&storage).is_err());
    }
}
#[test]
fn old_credential_cannot_reopen_previous_generation_during_rekey() {
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let public = owner.public_key();
    let old_password = password();
    let new_password = SecretString::try_from_slice(b"new synthetic password").unwrap();
    let new_key = [88; 32];
    let old_slots = [slot(&KEY)];
    let new_slots =
        [KeySlot::password_bytes(1, b"new synthetic password", vec![19; 16], &new_key).unwrap()];
    for signed in [false, true] {
        let mode = mode(signed);
        let mut storage = StorageBackend::memory(vec![0; REGION_LEN]);
        let old = stage(&mut storage, mode, &KEY, &old_slots, None);
        publish(&mut storage, &old, &KEY, &owner, true);
        let new = stage(&mut storage, mode, &new_key, &new_slots, Some(&old));
        // Prepared but unpublished credentials are not authority.
        assert!(open(
            &storage,
            archive(),
            mode,
            signed.then_some(&public),
            Credential::Password(&new_password),
            None
        )
        .is_err());
        publish(&mut storage, &new, &new_key, &owner, false);
        let opened = open(
            &storage,
            archive(),
            mode,
            signed.then_some(&public),
            Credential::Password(&new_password),
            None,
        )
        .unwrap();
        assert_eq!(opened.anchor.generation, 2);
        verify_private(&storage, &opened, &new_key);
        assert!(open(
            &storage,
            archive(),
            mode,
            signed.then_some(&public),
            Credential::Password(&old_password),
            None
        )
        .is_err());
        for offset in [new.keys.primary, new.keys.mirror] {
            storage.write_at(offset, &[0; INLINE_BYTES]).unwrap();
        }
        assert!(open(
            &storage,
            archive(),
            mode,
            signed.then_some(&public),
            Credential::Password(&old_password),
            None
        )
        .is_err());
    }
}
#[test]
fn public_directory_mirrors_and_trust_context_are_enforced() {
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let public = owner.public_key();
    let wrong_owner = OwnerSigningKeyPair::generate().unwrap().public_key();
    let slots = [slot(&KEY)];
    let password = password();
    for signed in [false, true] {
        let mode = mode(signed);
        let mut storage = StorageBackend::memory(vec![0; REGION_LEN]);
        let anchor = stage(&mut storage, mode, &KEY, &slots, None);
        publish(&mut storage, &anchor, &KEY, &owner, true);
        assert!(open(
            &storage,
            archive(),
            mode,
            Some(&wrong_owner),
            Credential::Password(&password),
            None
        )
        .is_err());
        if signed {
            assert!(open(
                &storage,
                archive(),
                mode,
                None,
                Credential::Password(&password),
                None
            )
            .is_err());
        }
        storage
            .write_at(anchor.keys.primary, &[0; INLINE_BYTES])
            .unwrap();
        let opened = open(
            &storage,
            archive(),
            mode,
            signed.then_some(&public),
            Credential::Password(&password),
            None,
        )
        .unwrap();
        verify_private(&storage, &opened, &KEY);
        storage
            .write_at(anchor.keys.mirror, &[0; INLINE_BYTES])
            .unwrap();
        assert!(open(
            &storage,
            archive(),
            mode,
            signed.then_some(&public),
            Credential::Password(&password),
            None
        )
        .is_err());
    }
}
#[test]
fn directory_rejects_noncanonical_and_overflow_inputs() {
    let slots = [slot(&KEY)];
    assert!(directory(archive(), 0, &slots).is_err());
    assert!(directory(archive(), 1, &[]).is_err());
    let duplicate = [slots[0].clone(), slots[0].clone()];
    assert!(directory(archive(), 1, &duplicate).is_err());
    let contact = ContactKeyPair::generate().unwrap();
    let large: Vec<_> = (1..8)
        .map(|id| KeySlot::hybrid_contact(id, &contact.public_key(), &KEY).unwrap())
        .collect();
    assert!(directory(archive(), 1, &large).is_err());
    let mut storage = StorageBackend::memory(vec![0; REGION_LEN]);
    let mut anchor = stage(&mut storage, mode(false), &KEY, &slots, None);
    let mut bytes = directory(archive(), 1, &slots).unwrap();
    bytes[INLINE_BYTES - 1] = 1;
    anchor.keys = roots(&mut storage, &bytes);
    assert!(read_directory(&storage, &anchor).is_err());
    anchor.keys.len = INLINE_BYTES as u64 + 1;
    assert!(read_directory(&storage, &anchor).is_err());
}

#[derive(Clone, Debug)]
struct Guarded {
    storage: StorageBackend,
    private_start: u64,
    fail_at: Option<u64>,
    reads: std::rc::Rc<std::cell::RefCell<Vec<(u64, usize)>>>,
}
impl Storage for Guarded {
    fn len(&self) -> Result<u64> {
        self.storage.len()
    }
    fn read_at(&self, offset: u64, len: usize) -> Result<Vec<u8>> {
        self.reads.borrow_mut().push((offset, len));
        assert!(
            offset + len as u64 <= self.private_start,
            "bootstrap touched private metadata"
        );
        if self.fail_at == Some(offset) {
            return Err(Error::InvalidInput("injected storage read failure".into()));
        }
        self.storage.read_at(offset, len)
    }
    fn read_at_into(&self, offset: u64, out: &mut [u8]) -> Result<()> {
        out.copy_from_slice(&self.read_at(offset, out.len())?);
        Ok(())
    }
    fn append(&mut self, _: &[u8]) -> Result<u64> {
        panic!("bootstrap wrote storage")
    }
    fn write_at(&mut self, _: u64, _: &[u8]) -> Result<()> {
        panic!("bootstrap wrote storage")
    }
    fn truncate(&mut self, _: u64) -> Result<()> {
        panic!("bootstrap truncated storage")
    }
    fn sync(&self) -> Result<()> {
        panic!("bootstrap synced storage")
    }
}
#[test]
fn bootstrap_reads_only_bounded_public_records_and_propagates_io_errors() {
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let public = owner.public_key();
    let slots = [slot(&KEY)];
    let password = password();
    for signed in [false, true] {
        let mode = mode(signed);
        let mut storage = StorageBackend::memory(vec![0; REGION_LEN]);
        let anchor = stage(&mut storage, mode, &KEY, &slots, None);
        publish(&mut storage, &anchor, &KEY, &owner, true);
        let reads = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
        let mut guarded = Guarded {
            storage,
            private_start: anchor.index.primary,
            fail_at: None,
            reads: reads.clone(),
        };
        open(
            &guarded,
            archive(),
            mode,
            signed.then_some(&public),
            Credential::Password(&password),
            None,
        )
        .unwrap();
        assert!(reads.borrow().iter().all(|(_, len)| *len <= SLOT_LEN));
        for offset in [0, SLOT_STRIDE as u64, anchor.keys.primary] {
            guarded.fail_at = Some(offset);
            let result = open(
                &guarded,
                archive(),
                mode,
                signed.then_some(&public),
                Credential::Password(&password),
                None,
            );
            assert!(
                matches!(result, Err(Error::InvalidInput(ref message)) if message == "injected storage read failure")
            );
        }
        guarded.fail_at = None;
        reads.borrow_mut().clear();
        let wrong = OwnerSigningKeyPair::generate().unwrap().public_key();
        assert!(open(
            &guarded,
            archive(),
            mode,
            Some(&wrong),
            Credential::Password(&password),
            None
        )
        .is_err());
        assert!(reads
            .borrow()
            .iter()
            .all(|(offset, _)| *offset < REGION_LEN as u64));
    }
}
#[test]
fn forged_unsigned_generation_never_grants_old_credentials_fallback() {
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let mode = mode(false);
    let password = password();
    let slots = [slot(&KEY)];
    let mut storage = StorageBackend::memory(vec![0; REGION_LEN]);
    let old = stage(&mut storage, mode, &KEY, &slots, None);
    publish(&mut storage, &old, &KEY, &owner, true);
    let new = stage(&mut storage, mode, &KEY, &slots, Some(&old));
    let mut forged = encode(&new, &Authority::Symmetric(&KEY), None).unwrap();
    forged[AUTH_START] ^= 1;
    let checksum = strong_checksum(&forged[..CHECKSUM_START]);
    forged[CHECKSUM_START..].copy_from_slice(&checksum);
    storage.write_at(0, &forged).unwrap();
    // Ordinary explicit-key selection can authenticate the old slot. Bootstrap
    // deliberately refuses: the newer structural slot may represent a rekey.
    assert_eq!(
        select(&storage, archive(), mode, &Authority::Symmetric(&KEY))
            .unwrap()
            .anchor
            .generation,
        1
    );
    assert!(open(
        &storage,
        archive(),
        mode,
        None,
        Credential::Password(&password),
        None
    )
    .is_err());
}
#[test]
fn directory_identity_generation_and_slot_ids_are_checked() {
    let slots = [slot(&KEY)];
    let mut zero = slots[0].clone();
    if let KeySlot::Password { id, .. } = &mut zero {
        *id = 0;
    }
    assert!(directory(archive(), 1, &[zero]).is_err());
    let mut storage = StorageBackend::memory(vec![0; REGION_LEN]);
    let mut anchor = stage(&mut storage, mode(false), &KEY, &slots, None);
    for bytes in [
        directory(LockboxId::from_bytes([99; 16]), 1, &slots).unwrap(),
        directory(archive(), 2, &slots).unwrap(),
    ] {
        anchor.keys = roots(&mut storage, &bytes);
        assert!(read_directory(&storage, &anchor).is_err());
    }
}
