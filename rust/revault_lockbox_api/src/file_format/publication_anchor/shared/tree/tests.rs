//! Direct storage fixtures are required: no public CLI writes this profile.
use super::*;
use crate::storage::StorageBackend;
use crate::{Compression, EncryptionMode, LockboxFormatOptions, SigningMode, SizePadding};

#[derive(Clone, Debug)]
struct Aligned(StorageBackend);
impl Storage for Aligned {
    fn len(&self) -> Result<u64> {
        self.0.len()
    }
    fn read_at(&self, offset: u64, len: usize) -> Result<Vec<u8>> {
        self.0.read_at(offset, len)
    }
    fn read_at_into(&self, offset: u64, out: &mut [u8]) -> Result<()> {
        self.0.read_at_into(offset, out)
    }
    fn append(&mut self, bytes: &[u8]) -> Result<u64> {
        self.0.append(bytes)
    }
    fn append_pair(&mut self, bytes: &[u8]) -> Result<(u64, u64)> {
        assert!(bytes.len() <= FAILURE_REGION as usize);
        assert_eq!(self.len()? % FAILURE_REGION, 0);
        let primary = self.append(bytes)?;
        self.append(&vec![0; FAILURE_REGION as usize - bytes.len()])?;
        let mirror = self.append(bytes)?;
        self.append(&vec![0; FAILURE_REGION as usize - bytes.len()])?;
        Ok((primary, mirror))
    }
    fn write_at(&mut self, offset: u64, bytes: &[u8]) -> Result<()> {
        self.0.write_at(offset, bytes)
    }
    fn truncate(&mut self, len: u64) -> Result<()> {
        self.0.truncate(len)
    }
    fn sync(&self) -> Result<()> {
        self.0.sync()
    }
}
fn mode(encrypted: bool, signed: bool, padded: bool, compressed: bool) -> FormatMode {
    FormatMode::new(LockboxFormatOptions {
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
        compression: if compressed {
            Compression::default()
        } else {
            Compression::None
        },
        size_padding: if padded {
            SizePadding::Default
        } else {
            SizePadding::None
        },
    })
}
fn archive() -> LockboxId {
    LockboxId::from_bytes([73; 16])
}
fn authority<'a>(mode: FormatMode, public: &'a OwnerSigningPublicKey) -> Authority<'a> {
    if mode.signed() {
        Authority::Owner(public)
    } else if mode.plaintext() {
        Authority::Checksum
    } else {
        Authority::Symmetric(&[71; 32])
    }
}
fn key(mode: FormatMode) -> Option<&'static [u8]> {
    (!mode.plaintext()).then_some(&[71; 32])
}
fn seed(
    mode: FormatMode,
    owner: &OwnerSigningKeyPair,
    entries: Vec<Result<Entry>>,
) -> (Aligned, RootRef, Vec<RootRef>) {
    let public = owner.public_key();
    let authority = authority(mode, &public);
    let mut storage = Aligned(StorageBackend::memory(vec![0; REGION_LEN]));
    let index = Index::new(archive(), mode, key(mode)).unwrap();
    let change = index.build_sorted(&mut storage, entries).unwrap();
    initialize(
        &mut storage,
        archive(),
        mode,
        &authority,
        mode.signed().then_some(owner),
        key(mode),
        &manifest(change.root),
        &[],
    )
    .unwrap();
    (storage, change.root, change.created)
}
fn entries(n: u32) -> Vec<Result<Entry>> {
    (0..n)
        .map(|i| Entry::new(1, &i.to_be_bytes(), &[(i % 251) as u8; 128]))
        .collect()
}
#[test]
fn shared_tree_opens_beyond_inline_capacity_and_survives_each_metadata_copy_loss() {
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let public = owner.public_key();
    for bits in 0..16 {
        let encrypted = bits & 1 != 0;
        let signed = bits & 2 != 0;
        let padded = bits & 4 != 0;
        let compressed = bits & 8 != 0;
        let mode = mode(encrypted, signed, padded, compressed);
        let authority = authority(mode, &public);
        let (storage, _, pages) = seed(mode, &owner, entries(2048));
        assert!(pages.len() > 1);
        let original = storage.0.read_all().unwrap();
        let check = |storage: &Aligned| {
            let tree = Tree::open(storage, archive(), mode, &authority, key(mode)).unwrap();
            tree.graph.verify_reclaimed(storage).unwrap();
            let mut count = 0u32;
            tree.visit(storage, |entry| {
                assert_eq!(entry.key.as_slice(), count.to_be_bytes());
                assert_eq!(entry.value.as_slice(), vec![(count % 251) as u8; 128]);
                count += 1;
                Ok(())
            })
            .unwrap();
            assert_eq!(count, 2048);
            for i in [0u32, 1024, 2047] {
                assert_eq!(
                    tree.get(storage, 1, &i.to_be_bytes())
                        .unwrap()
                        .unwrap()
                        .value
                        .as_slice(),
                    vec![(i % 251) as u8; 128]
                );
            }
            assert!(tree
                .get(storage, 1, &2048u32.to_be_bytes())
                .unwrap()
                .is_none());
            assert!(tree.get(storage, 0, b"reserved").is_err());
        };
        check(&storage);
        for bank in [0, FAILURE_REGION] {
            let mut damaged = Aligned(StorageBackend::memory(original.clone()));
            damaged
                .write_at(bank, &vec![0; FAILURE_REGION as usize])
                .unwrap();
            check(&damaged);
        }
        for page in &pages {
            for offset in [page.primary, page.mirror] {
                let mut damaged = Aligned(StorageBackend::memory(original.clone()));
                damaged
                    .write_at(offset, &vec![0; FAILURE_REGION as usize])
                    .unwrap();
                check(&damaged);
            }
        }
        let mut damaged = Aligned(StorageBackend::memory(original.clone()));
        for offset in [pages[0].primary, pages[0].mirror] {
            damaged
                .write_at(offset, &vec![0; FAILURE_REGION as usize])
                .unwrap();
        }
        assert!(Tree::open(&damaged, archive(), mode, &authority, key(mode)).is_err());
        if !padded {
            let page = pages.iter().find(|page| page.len < FAILURE_REGION).unwrap();
            let mut dirty = Aligned(StorageBackend::memory(original.clone()));
            dirty.write_at(page.primary + page.len, &[1]).unwrap();
            assert!(Tree::open(&dirty, archive(), mode, &authority, key(mode)).is_err());
        }
        assert_eq!(storage.0.read_all().unwrap(), original);
        if encrypted {
            assert!(Tree::open(&storage, archive(), mode, &authority, Some(&[99; 32])).is_err());
        }
        if signed {
            let wrong = OwnerSigningKeyPair::generate().unwrap().public_key();
            assert!(Tree::open(
                &storage,
                archive(),
                mode,
                &Authority::Owner(&wrong),
                key(mode)
            )
            .is_err());
        }
    }
}

#[test]
fn shared_tree_refuses_malformed_ownership_and_unclaimed_or_aliased_bytes() {
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let public = owner.public_key();
    let mode = mode(false, true, true, false);
    let authority = authority(mode, &public);
    for (name, value) in [
        (vec![], vec![]),
        (vec![9; 9], vec![0; 8]),
        (
            [vec![0], 0u64.to_be_bytes().to_vec()].concat(),
            4096u64.to_le_bytes().to_vec(),
        ),
        (
            [vec![2], (REGION_LEN as u64).to_be_bytes().to_vec()].concat(),
            [65536u64.to_le_bytes().to_vec(), vec![1; 32]].concat(),
        ),
    ] {
        let (storage, _, _) = seed(mode, &owner, vec![Entry::new(0, &name, &value)]);
        assert!(Tree::open(&storage, archive(), mode, &authority, None).is_err());
    }
    let (mut storage, root, _) = seed(mode, &owner, entries(2));
    // Append an unowned region and create a new complete initial publication on
    // copied bytes. Authentication alone must not make missing ownership valid.
    storage.append(&vec![0; FAILURE_REGION as usize]).unwrap();
    storage.write_at(0, &vec![0; REGION_LEN]).unwrap();
    initialize(
        &mut storage,
        archive(),
        mode,
        &authority,
        Some(&owner),
        None,
        &manifest(root),
        &[],
    )
    .unwrap();
    assert!(Tree::open(&storage, archive(), mode, &authority, None).is_err());
    for body in [vec![], vec![0; 64], vec![0; 65]] {
        assert!(parse_manifest(&body).is_err());
    }
}

#[test]
fn shared_tree_validates_free_pending_and_payload_ownership_before_exposing_records() {
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let public = owner.public_key();
    let mode = mode(false, true, true, false);
    let authority = authority(mode, &public);
    for free_kind in [0u8, 1] {
        let mut storage = Aligned(StorageBackend::memory(vec![0; REGION_LEN]));
        let payload = vec![37; FAILURE_REGION as usize];
        let payload_start = storage.append(&payload).unwrap();
        let free_start = storage.append(&vec![0; FAILURE_REGION as usize]).unwrap();
        let name = |kind: u8, start: u64| [vec![kind], start.to_be_bytes().to_vec()].concat();
        let rows = vec![
            Entry::new(
                0,
                &name(free_kind, free_start),
                &FAILURE_REGION.to_le_bytes(),
            ),
            Entry::new(
                0,
                &name(2, payload_start),
                &[
                    FAILURE_REGION.to_le_bytes().to_vec(),
                    strong_checksum(&payload).to_vec(),
                ]
                .concat(),
            ),
            Entry::new(1, b"public-record", b"value"),
        ];
        let index = Index::new(archive(), mode, None).unwrap();
        let change = index.build_sorted(&mut storage, rows).unwrap();
        initialize(
            &mut storage,
            archive(),
            mode,
            &authority,
            Some(&owner),
            None,
            &manifest(change.root),
            &[],
        )
        .unwrap();
        let tree = Tree::open(&storage, archive(), mode, &authority, None).unwrap();
        let mut count = 0;
        tree.visit(&storage, |entry| {
            assert_eq!(entry.key.as_slice(), b"public-record");
            count += 1;
            Ok(())
        })
        .unwrap();
        assert_eq!(count, 1);
        // The ownership reader deliberately does not claim typed payload content
        // verification; that belongs to the future record-specific adapter.
        storage.write_at(free_start, &[1]).unwrap();
        assert!(Tree::open(&storage, archive(), mode, &authority, None).is_err());
    }
}
