//! The public CLI cannot create these experimental dense/tree images. Build
//! through private writers, then reopen credentials and content independently.
use super::*;
use crate::file_format::publication_anchor::bootstrap::Credential;
use crate::key_slot::KeySlot;

fn credential_source(
    mode: FormatMode,
    authority: &Authority<'_>,
    owner: &OwnerSigningKeyPair,
    slots: &[KeySlot],
) -> StorageBackend {
    let signer = mode.signed().then_some(owner);
    let source = Files::create(
        StorageBackend::memory(Vec::new()),
        archive(),
        mode,
        authority,
        signer,
        key(mode),
        65536,
        [Input {
            path: b"/docs/data".to_vec(),
            reader: Cursor::new(b"payload".to_vec()),
        }],
    )
    .unwrap();
    let mut source = Files::open(source, archive(), mode, authority, key(mode)).unwrap();
    let mut dense = super::super::super::dense_image::from_candidate(
        &mut source,
        StorageBackend::memory(Vec::new()),
        authority,
        signer,
        key(mode),
        slots,
    )
    .unwrap();
    super::super::super::dense_update::replace_filesystem_metadata(
        &mut dense,
        archive(),
        mode,
        authority,
        signer,
        key(mode),
        &public_filesystem_metadata(owner),
    )
    .unwrap();
    // Force a canonical directory newer than generation one. There is no
    // public slot mutation API for this experimental image; the authenticated
    // fixture helper preserves lineage and independently checks the new anchor.
    let anchor = shared::tests::replace_fixture_slots(
        &mut dense,
        archive(),
        mode,
        authority,
        signer,
        key(mode),
        slots,
    )
    .unwrap();
    assert!(anchor.generation > 1);
    dense
}

#[test]
fn dense_tree_export_preserves_wrappers_and_resets_directory_generation() {
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let public = owner.public_key();
    let password = crate::SecretString::try_from_slice(b"synthetic dense export").unwrap();
    let contact = crate::ContactKeyPair::generate().unwrap();
    let wrong = crate::SecretString::try_from_slice(b"wrong dense export").unwrap();
    let entries = public_filesystem_metadata(&owner);
    for bits in [1, 3, 5, 7, 9, 11, 13, 15] {
        let mode = mode(bits & 1 != 0, bits & 2 != 0, bits & 4 != 0, bits & 8 != 0);
        let authority = authority(mode, &public);
        let slots = [
            KeySlot::password_bytes(
                31,
                b"synthetic dense export",
                vec![71; 16],
                key(mode).unwrap(),
            )
            .unwrap(),
            KeySlot::hybrid_contact(47, &contact.public_key(), key(mode).unwrap()).unwrap(),
        ];
        let source = credential_source(mode, &authority, &owner, &slots);
        let original = source.read_all().unwrap();
        let source_anchor = shared::open_private(&source, archive(), mode, &authority, key(mode))
            .unwrap()
            .0;
        let original_directory = shared::retained_public_directory(&source, &source_anchor)
            .unwrap()
            .unwrap();
        let decoded =
            crate::file_format::key_directory::read_key_directory_backup(&original_directory)
                .unwrap();
        assert!(decoded.generation > 1);
        for damaged_bank in [
            None,
            Some(source_anchor.keys.primary),
            Some(source_anchor.keys.mirror),
        ] {
            let mut readable = StorageBackend::memory(original.clone());
            if let Some(at) = damaged_bank {
                readable.write_at(at, &[0]).unwrap();
            }
            let before = readable.read_all().unwrap();
            let destination = tree_image::from_dense(
                &readable,
                StorageBackend::memory(Vec::new()),
                archive(),
                mode,
                &authority,
                mode.signed().then_some(&owner),
                key(mode),
            )
            .unwrap();
            check(&destination, mode, &authority, &entries);
            let anchor = shared::open_private(&destination, archive(), mode, &authority, key(mode))
                .unwrap()
                .0;
            assert_eq!(anchor.generation, 1);
            assert_eq!(anchor.previous, [0; 32]);
            let directory = shared::retained_public_directory(&destination, &anchor)
                .unwrap()
                .unwrap();
            assert_eq!(
                directory,
                publication::bootstrap::directory(archive(), 1, &slots).unwrap()
            );
            assert_ne!(directory, original_directory);
            for (credential, id) in [
                (Credential::Password(&password), 31),
                (Credential::Contact(&contact), 47),
            ] {
                let mut opened = TreeImage::open_credential(
                    allocation::compaction::View(&destination),
                    archive(),
                    mode,
                    mode.signed().then_some(&public),
                    credential,
                    Some(id),
                )
                .unwrap();
                assert_eq!(opened.image.filesystem_metadata().unwrap(), entries);
                let mut bytes = Vec::new();
                opened
                    .image
                    .read_range(b"/docs/data", 0, 7, |part| {
                        bytes.extend_from_slice(part);
                        Ok(())
                    })
                    .unwrap();
                assert_eq!(bytes, b"payload");
            }
            assert!(TreeImage::open_credential(
                allocation::compaction::View(&destination),
                archive(),
                mode,
                mode.signed().then_some(&public),
                Credential::Password(&wrong),
                Some(31)
            )
            .is_err());
            assert!(TreeImage::open_credential(
                allocation::compaction::View(&destination),
                archive(),
                mode,
                mode.signed().then_some(&public),
                Credential::Password(&password),
                Some(999)
            )
            .is_err());
            assert_eq!(readable.read_all().unwrap(), before);
        }
        let mut broken = StorageBackend::memory(original.clone());
        for at in [source_anchor.keys.primary, source_anchor.keys.mirror] {
            broken.write_at(at, &[0]).unwrap();
        }
        let before = broken.read_all().unwrap();
        let output = SharedMemory::new(Vec::new());
        assert!(tree_image::from_dense(
            &broken,
            output.clone(),
            archive(),
            mode,
            &authority,
            mode.signed().then_some(&owner),
            key(mode)
        )
        .is_err());
        assert_eq!(output.len().unwrap(), 0);
        assert_eq!(broken.read_all().unwrap(), before);
        assert_eq!(source.read_all().unwrap(), original);
    }
}

#[derive(Clone, Debug)]
struct PartialDirectoryWrite {
    storage: SharedMemory,
    offset: u64,
    hit: std::rc::Rc<std::cell::Cell<bool>>,
}
impl Storage for PartialDirectoryWrite {
    fn len(&self) -> Result<u64> {
        self.storage.len()
    }
    fn read_at(&self, at: u64, len: usize) -> Result<Vec<u8>> {
        self.storage.read_at(at, len)
    }
    fn read_at_into(&self, at: u64, out: &mut [u8]) -> Result<()> {
        self.storage.read_at_into(at, out)
    }
    fn append(&mut self, bytes: &[u8]) -> Result<u64> {
        self.storage.append(bytes)
    }
    fn write_at(&mut self, at: u64, bytes: &[u8]) -> Result<()> {
        if at == self.offset && !self.hit.replace(true) {
            self.storage.write_at(at, &bytes[..bytes.len() / 2])?;
            return Err(Error::Io("synthetic partial exported directory".into()));
        }
        self.storage.write_at(at, bytes)
    }
    fn truncate(&mut self, len: u64) -> Result<()> {
        self.storage.truncate(len)
    }
    fn sync(&self) -> Result<()> {
        self.storage.sync()
    }
}

#[test]
fn dense_tree_export_discards_partial_credential_directories() {
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let public = owner.public_key();
    let mut cases = 0;
    for bits in [1, 3, 5, 7, 9, 11, 13, 15] {
        let mode = mode(bits & 1 != 0, bits & 2 != 0, bits & 4 != 0, bits & 8 != 0);
        let authority = authority(mode, &public);
        let slot = KeySlot::password_bytes(
            1,
            b"synthetic partial export",
            vec![72; 16],
            key(mode).unwrap(),
        )
        .unwrap();
        let source = credential_source(mode, &authority, &owner, &[slot]);
        let before = source.read_all().unwrap();
        let completed = tree_image::from_dense(
            &source,
            StorageBackend::memory(Vec::new()),
            archive(),
            mode,
            &authority,
            mode.signed().then_some(&owner),
            key(mode),
        )
        .unwrap();
        let anchor = shared::open_private(&completed, archive(), mode, &authority, key(mode))
            .unwrap()
            .0;
        for offset in [anchor.keys.primary, anchor.keys.mirror] {
            let storage = SharedMemory::new(Vec::new());
            let hit = std::rc::Rc::new(std::cell::Cell::new(false));
            let output = PartialDirectoryWrite {
                storage: storage.clone(),
                offset,
                hit: hit.clone(),
            };
            assert!(tree_image::from_dense(
                &source,
                output,
                archive(),
                mode,
                &authority,
                mode.signed().then_some(&owner),
                key(mode)
            )
            .is_err());
            assert!(hit.get());
            assert_eq!(storage.len().unwrap(), 0);
            assert_eq!(source.read_all().unwrap(), before);
            cases += 1;
        }
    }
    assert_eq!(cases, 16);
    println!("DENSE_ACCESS_PARTIAL_WRITES cases={cases}");
}
