//! Private candidate fixtures: the public CLI cannot construct this layout or
//! inject physical corruption. All observations use ordinary read/audit APIs.
use super::super::super::tree_image::TreeImage;
use super::*;
use crate::file_format::authenticated_index::{Index, Visit};
use std::{cell::RefCell, rc::Rc};

#[derive(Clone, Debug)]
struct Trace {
    storage: StorageBackend,
    reads: Rc<RefCell<Vec<(u64, u64)>>>,
}
impl Storage for Trace {
    fn len(&self) -> Result<u64> {
        self.storage.len()
    }
    fn read_at(&self, at: u64, len: usize) -> Result<Vec<u8>> {
        self.reads.borrow_mut().push((at, at + len as u64));
        self.storage.read_at(at, len)
    }
    fn read_at_into(&self, at: u64, bytes: &mut [u8]) -> Result<()> {
        self.reads.borrow_mut().push((at, at + bytes.len() as u64));
        self.storage.read_at_into(at, bytes)
    }
    fn append(&mut self, _: &[u8]) -> Result<u64> {
        panic!("read-only")
    }
    fn write_at(&mut self, _: u64, _: &[u8]) -> Result<()> {
        panic!("read-only")
    }
    fn truncate(&mut self, _: u64) -> Result<()> {
        panic!("read-only")
    }
    fn sync(&self) -> Result<()> {
        panic!("read-only")
    }
}
fn intersects(a: (u64, u64), b: (u64, u64)) -> bool {
    a.0 < b.1 && b.0 < a.1
}
fn content(n: usize) -> Vec<u8> {
    (0..n).map(|i| ((i * 31 + i / 17) % 251) as u8).collect()
}
fn files(
    mode: FormatMode,
    authority: &Authority<'_>,
    owner: &OwnerSigningKeyPair,
) -> (StorageBackend, Vec<u8>) {
    let bytes = content(140_123);
    let names = ["/file-a", "/file-z"];
    let signer = mode.signed().then_some(owner);
    let source = Files::create(
        StorageBackend::memory(Vec::new()),
        archive(),
        mode,
        authority,
        signer,
        key(mode),
        65536,
        names.iter().map(|name| Input {
            path: name.as_bytes().to_vec(),
            reader: std::io::Cursor::new(bytes.clone()),
        }),
    )
    .unwrap();
    let mut source = Files::open(source, archive(), mode, authority, key(mode)).unwrap();
    let metadata: Vec<_> = names
        .iter()
        .map(|name| Metadata {
            entry: crate::LockboxEntry {
                path: crate::LockboxPath::new(name).unwrap(),
                kind: crate::LockboxEntryKind::File,
                len: bytes.len() as u64,
                permissions: 0o600,
            },
            target: None,
        })
        .collect();
    (
        tree_image::from_candidate(
            &mut source,
            StorageBackend::memory(Vec::new()),
            authority,
            signer,
            key(mode),
            &metadata,
        )
        .unwrap(),
        bytes,
    )
}
fn read<S: Storage>(
    image: &mut TreeImage<S>,
    path: &[u8],
    offset: u64,
    len: u64,
) -> Result<Vec<u8>> {
    let mut out = Vec::new();
    image.image.read_range(path, offset, len, |v| {
        out.extend_from_slice(v);
        Ok(())
    })?;
    Ok(out)
}

#[test]
fn selective_reads_skip_unrelated_payload_padding_and_free_space_all_modes() {
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let public = owner.public_key();
    for bits in 0..16 {
        let mode = mode(bits & 1 != 0, bits & 2 != 0, bits & 4 != 0, bits & 8 != 0);
        let authority = authority(mode, &public);
        let (storage, bytes) = files(mode, &authority, &owner);
        let audit = AuditedTreeImage::open(storage.clone(), archive(), mode, &authority, key(mode))
            .unwrap();
        let first = &audit.image.catalogue.files[0];
        let selected = &first.fragments[0];
        let selected_start =
            audit.image.catalogue.packs[selected.pack].extent.start + selected.relative as u64;
        let selected_end = selected_start + selected.descriptor.stored_len() as u64;
        let mut forbidden = Vec::new();
        for file in &audit.image.catalogue.files {
            for fragment in &file.fragments {
                let at = audit.image.catalogue.packs[fragment.pack].extent.start
                    + fragment.relative as u64;
                if at != selected_start {
                    forbidden.push((at, at + fragment.descriptor.stored_len() as u64));
                }
            }
        }
        for pack in &audit.image.catalogue.packs {
            if (pack.used as u64) < pack.extent.len {
                forbidden.push((
                    pack.extent.start + pack.used as u64,
                    pack.extent.start + pack.extent.len,
                ));
            }
        }
        // Control/journal admission is bounded and intentionally still read.
        // Only genuinely free claims are excluded, not descendant pages marked
        // as pending by the return-to-inline inventory helper.
        for vacant in audit.tree.graph.catalogue_vacant() {
            if vacant.kind == shared::ownership::VacantKind::Free {
                forbidden.push((vacant.span.start, vacant.span.start + vacant.span.len));
            }
        }
        let trace = Trace {
            storage: storage.clone(),
            reads: Rc::new(RefCell::new(Vec::new())),
        };
        let mut reader =
            TreeImage::open(trace.clone(), archive(), mode, &authority, key(mode)).unwrap();
        assert!(
            !trace
                .reads
                .borrow()
                .iter()
                .any(|&r| intersects(r, (selected_start, selected_end))),
            "open read payload mode={bits}"
        );
        assert_eq!(
            read(&mut reader, b"/file-a", 31, 4096).unwrap(),
            bytes[31..4127]
        );
        for &r in trace.reads.borrow().iter() {
            assert!(
                !forbidden.iter().any(|&span| intersects(r, span)),
                "unrelated read {r:?} mode={bits}"
            );
        }
        trace.reads.borrow_mut().clear();
        assert!(reader.image.info(b"/file-a").unwrap().is_some());
        assert!(
            trace.reads.borrow().is_empty(),
            "verified page cache should satisfy repeated metadata lookup"
        );
        assert_eq!(
            read(&mut reader, b"/file-a", 65000, 70000).unwrap(),
            bytes[65000..135000]
        );
        assert_eq!(
            read(&mut reader, b"/file-z", 0, bytes.len() as u64).unwrap(),
            bytes
        );
        assert!(reader.image.info(b"/absent").unwrap().is_none());
        assert!(read(&mut reader, b"/file-a", u64::MAX, 2).is_err());
        assert!(read(&mut reader, b"/file-a", bytes.len() as u64, 1).is_err());
        assert!(read(&mut reader, b"/file-a", bytes.len() as u64, 0)
            .unwrap()
            .is_empty());
        reader.image.verify_all().unwrap();

        // Unread payload damage does not prevent opening or authenticated reads
        // elsewhere, including signed plaintext. Requested corruption is fatal
        // before the first callback for that chunk; a full audit also rejects it.
        let bad = &first.fragments[1];
        let at = audit.image.catalogue.packs[bad.pack].extent.start + bad.relative as u64;
        let mut damaged = StorageBackend::memory(storage.read_all().unwrap());
        let original = damaged.read_at(at, 1).unwrap()[0];
        damaged.write_at(at, &[original ^ 1]).unwrap();
        let mut reader = TreeImage::open(damaged, archive(), mode, &authority, key(mode)).unwrap();
        assert_eq!(
            read(&mut reader, b"/file-a", 31, 4096).unwrap(),
            bytes[31..4127]
        );
        let mut calls = 0;
        assert!(reader
            .image
            .read_range(b"/file-a", bad.descriptor.offset, 1, |_| {
                calls += 1;
                Ok(())
            })
            .is_err());
        assert_eq!(calls, 0);
        assert!(reader.image.verify_all().is_err());
    }
}

#[test]
fn selective_reads_leave_spare_space_auditing_to_full_verification_and_writers() {
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let public = owner.public_key();
    for encrypted in [false, true] {
        let mode = mode(encrypted, true, false, true);
        let authority = authority(mode, &public);
        let (storage, bytes) = files(mode, &authority, &owner);
        let audit = AuditedTreeImage::open(storage.clone(), archive(), mode, &authority, key(mode))
            .unwrap();
        let pack = audit
            .image
            .catalogue
            .packs
            .iter()
            .find(|p| (p.used as u64) < p.extent.len)
            .unwrap();
        let vacant = audit
            .tree
            .graph
            .catalogue_vacant()
            .into_iter()
            .find(|v| v.kind == shared::ownership::VacantKind::Free && v.span.len > 0)
            .unwrap();
        for at in [pack.extent.start + pack.used as u64, vacant.span.start] {
            let mut damaged = StorageBackend::memory(storage.read_all().unwrap());
            let original = damaged.read_at(at, 1).unwrap()[0];
            damaged.write_at(at, &[original ^ 1]).unwrap();
            let before = damaged.read_all().unwrap();
            let mut reader =
                TreeImage::open(damaged.clone(), archive(), mode, &authority, key(mode)).unwrap();
            assert_eq!(read(&mut reader, b"/file-a", 0, 7).unwrap(), bytes[..7]);
            assert!(reader.image.verify_all().is_err());
            assert!(tree_image::remove_files(
                &mut damaged,
                archive(),
                mode,
                &authority,
                Some(&owner),
                key(mode),
                &[b"/file-a".to_vec()]
            )
            .is_err());
            assert_eq!(
                damaged.read_all().unwrap(),
                before,
                "writer changed unaudited storage"
            );
        }
    }
}

#[test]
fn selective_reads_skip_damaged_unrelated_index_subtrees_and_authenticate_selected_pages() {
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let public = owner.public_key();
    let mode = mode(false, true, false, true);
    let authority = authority(mode, &public);
    let entries = grown(&public_filesystem_metadata(&owner));
    let mut storage = exported(
        mode,
        &authority,
        &owner,
        &public_filesystem_metadata(&owner),
    );
    tree_image::replace_metadata(
        &mut storage,
        archive(),
        mode,
        &authority,
        Some(&owner),
        key(mode),
        &entries,
    )
    .unwrap();
    let (anchor, body) =
        shared::open_private(&storage, archive(), mode, &authority, key(mode)).unwrap();
    let root = shared::tree::parse_manifest(&body).unwrap();
    let index = Index::new(archive(), mode, key(mode)).unwrap();
    let mut pages = Vec::new();
    index
        .visit_owned(&storage, root, anchor.sealed_len, |event| {
            match event {
                Visit::Page(page) => pages.push((page, Vec::new())),
                Visit::Entry(row) => pages.last_mut().unwrap().1.push(row.namespace),
            }
            Ok(())
        })
        .unwrap();
    let unrelated = pages
        .iter()
        .find(|(_, ns)| !ns.is_empty() && ns.iter().all(|&n| n == 4))
        .unwrap()
        .0;
    let mut damaged = StorageBackend::memory(storage.read_all().unwrap());
    for at in [unrelated.primary, unrelated.mirror] {
        damaged
            .write_at(at, &vec![0; unrelated.len as usize])
            .unwrap();
    }
    let trace = Trace {
        storage: damaged,
        reads: Rc::new(RefCell::new(Vec::new())),
    };
    let mut reader =
        TreeImage::open(trace.clone(), archive(), mode, &authority, key(mode)).unwrap();
    assert_eq!(read(&mut reader, b"/docs/data", 0, 7).unwrap(), b"payload");
    for at in [unrelated.primary, unrelated.mirror] {
        assert!(!trace
            .reads
            .borrow()
            .iter()
            .any(|&r| intersects(r, (at, at + unrelated.len))));
    }
    assert!(reader.image.filesystem_metadata().is_err());
    assert!(reader.image.verify_all().is_err());
    // The authenticated root page is always necessary. One valid mirror works;
    // destroying both copies refuses before exposing any file data.
    let mut damaged = StorageBackend::memory(storage.read_all().unwrap());
    damaged
        .write_at(root.primary, &vec![0; root.len as usize])
        .unwrap();
    let mut reader =
        TreeImage::open(damaged.clone(), archive(), mode, &authority, key(mode)).unwrap();
    assert_eq!(read(&mut reader, b"/docs/data", 0, 7).unwrap(), b"payload");
    damaged
        .write_at(root.mirror, &vec![0; root.len as usize])
        .unwrap();
    assert!(TreeImage::open(damaged, archive(), mode, &authority, key(mode)).is_err());
}

#[test]
fn selective_reads_refuse_authenticated_misbound_fragment_descriptors() {
    // Deliberately malformed but authenticated records require the private raw
    // tree writer: the public CLI cannot publish these invalid typed bindings.
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let public = owner.public_key();
    for bits in 0..16 {
        let mode = mode(bits & 1 != 0, bits & 2 != 0, bits & 4 != 0, bits & 8 != 0);
        let authority = authority(mode, &public);
        let source = exported(
            mode,
            &authority,
            &owner,
            &public_filesystem_metadata(&owner),
        );
        let tree =
            shared::tree::Tree::open(&source, archive(), mode, &authority, key(mode)).unwrap();
        let mut original = Vec::new();
        tree.visit(&source, |row| {
            original.push(row);
            Ok(())
        })
        .unwrap();
        for fault in 0..10 {
            let mut records = original.clone();
            let fragment = records.iter().position(|r| r.namespace == 2).unwrap();
            let file = records.iter().position(|r| r.namespace == 1).unwrap();
            let pack = records.iter().position(|r| r.namespace == 3).unwrap();
            match fault {
                0 => records[fragment].value[8..24].copy_from_slice(&[99; 16]),
                1 => records[fragment].value[24..32].copy_from_slice(&1u64.to_le_bytes()),
                2 => records[fragment].value[32..40].copy_from_slice(&1u64.to_le_bytes()),
                3 => records[fragment].value[40..44].copy_from_slice(&6u32.to_le_bytes()),
                4 => records[fragment].value[64..72].copy_from_slice(&0u64.to_le_bytes()),
                5 => records[fragment].value[72..80].copy_from_slice(&u64::MAX.to_le_bytes()),
                6 => records[fragment].value[80] ^= 1,
                7 => records[pack].value[..8].copy_from_slice(&u64::MAX.to_le_bytes()),
                8 => records[file].value[28..36].copy_from_slice(&65537u64.to_le_bytes()),
                9 => {
                    records.remove(fragment);
                }
                _ => unreachable!(),
            }
            let mut storage = StorageBackend::memory(source.read_all().unwrap());
            shared::tree::rewrite_records(
                &mut storage,
                archive(),
                mode,
                &authority,
                mode.signed().then_some(&owner),
                key(mode),
                records,
            )
            .unwrap();
            let persisted = storage.read_all().unwrap();
            let mut reader = TreeImage::open(
                StorageBackend::memory(persisted.clone()),
                archive(),
                mode,
                &authority,
                key(mode),
            )
            .unwrap();
            let mut callbacks = 0;
            assert!(
                reader
                    .image
                    .read_range(b"/docs/data", 0, 7, |_| {
                        callbacks += 1;
                        Ok(())
                    })
                    .is_err(),
                "mode={bits} fault={fault}"
            );
            assert_eq!(callbacks, 0, "mode={bits} fault={fault}");
            assert_eq!(storage.read_all().unwrap(), persisted);
        }
    }
}
