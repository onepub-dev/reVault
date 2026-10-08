//! Compare complete typed opens against the former two-traversal construction.
use super::*;
use std::{cell::Cell, rc::Rc};

#[derive(Clone, Debug)]
struct ReadCount {
    storage: StorageBackend,
    calls: Rc<Cell<usize>>,
}
impl Storage for ReadCount {
    fn len(&self) -> Result<u64> {
        self.storage.len()
    }
    fn read_at(&self, at: u64, len: usize) -> Result<Vec<u8>> {
        self.calls.set(self.calls.get() + 1);
        self.storage.read_at(at, len)
    }
    fn read_at_into(&self, at: u64, out: &mut [u8]) -> Result<()> {
        self.calls.set(self.calls.get() + 1);
        self.storage.read_at_into(at, out)
    }
    fn append(&mut self, _: &[u8]) -> Result<u64> {
        panic!("read-only open")
    }
    fn write_at(&mut self, _: u64, _: &[u8]) -> Result<()> {
        panic!("read-only open")
    }
    fn truncate(&mut self, _: u64) -> Result<()> {
        panic!("read-only open")
    }
    fn sync(&self) -> Result<()> {
        panic!("read-only open")
    }
}

#[test]
fn typed_tree_single_traversal_preserves_complete_open_all_modes() {
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let public = owner.public_key();
    let entries = public_filesystem_metadata(&owner);
    for bits in 0..16 {
        let mode = mode(bits & 1 != 0, bits & 2 != 0, bits & 4 != 0, bits & 8 != 0);
        let authority = authority(mode, &public);
        let counted = ReadCount {
            storage: exported(mode, &authority, &owner, &entries),
            calls: Rc::new(Cell::new(0)),
        };
        let codec = Codec::shared_packed(archive(), mode, key(mode)).unwrap();
        let (catalogue, tree) =
            super::super::super::dense_catalogue::Catalogue::with_tree(&codec, |visitor| {
                let tree =
                    shared::tree::Tree::open(&counted, archive(), mode, &authority, key(mode))?;
                tree.visit(&counted, visitor)?;
                Ok(tree)
            })
            .unwrap();
        catalogue.verify_padding(&counted, &codec).unwrap();
        let mut previous = super::super::super::dense_image::Image {
            storage: counted.clone(),
            anchor: tree.anchor,
            value_key: super::super::super::dense_image::value_key_for(&catalogue, mode, key(mode))
                .unwrap(),
            catalogue,
            codec,
        };
        if mode.plaintext() && mode.signed() {
            previous.verify_all().unwrap();
        }
        let old_reads = counted.calls.replace(0);
        let mut current =
            TreeImage::open(counted.clone(), archive(), mode, &authority, key(mode)).unwrap();
        let new_reads = counted.calls.get();
        assert!(
            new_reads < old_reads,
            "mode={bits}: {new_reads} >= {old_reads}"
        );
        assert_eq!(
            previous.filesystem_metadata().unwrap(),
            current.image.filesystem_metadata().unwrap()
        );
        previous.verify_all().unwrap();
        current.image.verify_all().unwrap();
        assert!(shared::tree::Tree::open_visit(
            &counted,
            archive(),
            mode,
            &authority,
            key(mode),
            |_| Err(Error::CorruptRecord)
        )
        .is_err());
        println!("TYPED_OPEN_READS mode={bits} previous={old_reads} single={new_reads}");
    }
}
