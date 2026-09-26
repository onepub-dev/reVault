//! Protocol tests use storage directly: no public CLI activates this candidate.
use super::*;
use crate::file_format::publication_anchor::{publish, select, Anchor, Authority};
use crate::storage::StorageBackend;
use crate::{
    Compression, EncryptionMode, LockboxFormatOptions, OwnerSigningKeyPair, SigningMode,
    SizePadding,
};
use std::cell::Cell;
use std::collections::BTreeMap;

struct Directory(std::path::PathBuf);
impl Directory {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "revault-index-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
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

fn mode(encrypted: bool, signed: bool) -> FormatMode {
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
        compression: Compression::None,
        size_padding: SizePadding::None,
    })
}
fn index(encrypted: bool, signed: bool) -> Index {
    Index::new(
        LockboxId::from_bytes([45; 16]),
        mode(encrypted, signed),
        encrypted.then_some(&[17; 32][..]),
    )
    .unwrap()
}
fn put(
    index: &Index,
    storage: &mut impl Storage,
    root: RootRef,
    ns: u8,
    key: &[u8],
    value: &[u8],
) -> Change {
    let sealed = storage.len().unwrap();
    index
        .put(storage, root, sealed, Entry::new(ns, key, value).unwrap())
        .unwrap()
}
fn remove(index: &Index, storage: &mut impl Storage, root: RootRef, ns: u8, key: &[u8]) -> Change {
    let sealed = storage.len().unwrap();
    index.remove(storage, root, sealed, ns, key).unwrap()
}
fn anchor(index: &Index, storage: &impl Storage, root: RootRef, old: Option<&Anchor>) -> Anchor {
    Anchor {
        archive: index.archive,
        generation: old.map_or(1, |old| old.generation + 1),
        mode: index.mode,
        sealed_len: storage.len().unwrap(),
        object_root: root.digest,
        previous: old.map_or([0; 32], |old| old.commitment().unwrap()),
        index: root,
        allocation: RootRef::default(),
        keys: RootRef::default(),
    }
}
type Values = BTreeMap<(u8, Vec<u8>), Vec<u8>>;
fn values(index: &Index, storage: &impl Storage, root: RootRef, sealed: u64) -> Values {
    let mut values = Values::new();
    index
        .visit(storage, root, sealed, |entry| {
            assert!(values
                .insert((entry.namespace, entry.key.to_vec()), entry.value.to_vec())
                .is_none());
            Ok(())
        })
        .unwrap();
    values
}

#[test]
fn file_reopen_all_modes_preserves_published_lifecycle_and_hides_preparation() {
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let public = owner.public_key();
    for encrypted in [false, true] {
        for signed in [false, true] {
            let index = index(encrypted, signed);
            let authority = if signed {
                Authority::Owner(&public)
            } else if encrypted {
                Authority::Symmetric(&[17; 32])
            } else {
                Authority::Checksum
            };
            let dir = Directory::new();
            let path = dir.0.join("candidate.lbox");
            let mut storage = StorageBackend::create_file(&path, &vec![0; REGION_LEN]).unwrap();
            let mut root = index.empty(&mut storage).unwrap();
            let mut expected = Values::new();
            for namespace in 0..8 {
                for number in 0..8 {
                    let key = format!("private/name/{number}").into_bytes();
                    let value = format!("secret metadata {namespace}:{number}").into_bytes();
                    root = put(&index, &mut storage, root, namespace, &key, &value).root;
                    expected.insert((namespace, key), value);
                }
            }
            let first = anchor(&index, &storage, root, None);
            let published = publish(
                &mut storage,
                &first,
                &authority,
                signed.then_some(&owner),
                None,
            )
            .unwrap();
            let before = storage.len().unwrap();
            let unchanged = put(
                &index,
                &mut storage,
                root,
                2,
                b"private/name/3",
                b"secret metadata 2:3",
            );
            assert_eq!(unchanged.root, root);
            assert!(unchanged.created.is_empty() && unchanged.retired.is_empty());
            assert_eq!(remove(&index, &mut storage, root, 2, b"missing").root, root);
            assert_eq!(storage.len().unwrap(), before);
            root = put(
                &index,
                &mut storage,
                root,
                2,
                b"private/name/3",
                b"replacement",
            )
            .root;
            expected.insert((2, b"private/name/3".to_vec()), b"replacement".to_vec());
            root = put(&index, &mut storage, root, 1, b"new-name", b"added").root;
            expected.insert((1, b"new-name".to_vec()), b"added".to_vec());
            let old_value = expected.remove(&(0, b"private/name/0".to_vec())).unwrap();
            root = remove(&index, &mut storage, root, 0, b"private/name/0").root;
            root = put(&index, &mut storage, root, 0, b"renamed", &old_value).root;
            expected.insert((0, b"renamed".to_vec()), old_value);
            let still_old = select(&storage, index.archive, index.mode, &authority).unwrap();
            assert_eq!(still_old.anchor, first);
            assert!(index
                .get(&storage, first.index, first.sealed_len, 1, b"new-name")
                .unwrap()
                .is_none());
            let second = anchor(&index, &storage, root, Some(&first));
            publish(
                &mut storage,
                &second,
                &authority,
                signed.then_some(&owner),
                Some(published.commitment),
            )
            .unwrap();
            drop(storage);
            let reopened = StorageBackend::file(&path).unwrap();
            assert_eq!(
                select(&reopened, index.archive, index.mode, &authority)
                    .unwrap()
                    .anchor,
                second
            );
            assert_eq!(values(&index, &reopened, root, second.sealed_len), expected);
            for ((namespace, key), value) in &expected {
                assert_eq!(
                    index
                        .get(&reopened, root, second.sealed_len, *namespace, key)
                        .unwrap()
                        .unwrap()
                        .value
                        .as_slice(),
                    value
                );
            }
            assert!(index
                .get(&reopened, root, second.sealed_len, 0, b"private/name/0")
                .unwrap()
                .is_none());
            if encrypted {
                let bytes = reopened.read_all().unwrap();
                for private in [
                    b"private/name/".as_slice(),
                    b"secret metadata",
                    b"replacement",
                ] {
                    assert!(!bytes.windows(private.len()).any(|window| window == private));
                }
            }
        }
    }
}
fn leaf_ref(
    index: &Index,
    storage: &impl Storage,
    root: RootRef,
    namespace: u8,
    key: &[u8],
) -> RootRef {
    let route = index.route(namespace, key);
    let mut reference = root;
    loop {
        match index
            .read(storage, reference, storage.len().unwrap())
            .unwrap()
        {
            Node::Leaf(entry) => {
                assert_eq!(entry.key.as_slice(), key);
                return reference;
            }
            Node::Branch { bit, children, .. } => {
                reference = children[bit_at(route, bit)].reference
            }
            Node::Empty => panic!("missing leaf"),
        }
    }
}
#[test]
fn signed_membership_recovers_intact_content_without_reading_damaged_neighbor() {
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let public = owner.public_key();
    for encrypted in [false, true] {
        let index = index(encrypted, true);
        let authority = Authority::Owner(&public);
        let mut storage = StorageBackend::memory(vec![0; REGION_LEN]);
        let mut root = index.empty(&mut storage).unwrap();
        // Test descriptor binds stored content. Codec integration is still pending.
        let data = b"surviving stored content";
        let offset = storage.append(data).unwrap();
        let descriptor = [
            offset.to_le_bytes().as_slice(),
            &(data.len() as u64).to_le_bytes(),
            &strong_checksum(data),
        ]
        .concat();
        root = put(&index, &mut storage, root, 1, b"keep", &descriptor).root;
        root = put(
            &index,
            &mut storage,
            root,
            1,
            b"lose",
            b"neighbor descriptor",
        )
        .root;
        let first = anchor(&index, &storage, root, None);
        publish(&mut storage, &first, &authority, Some(&owner), None).unwrap();
        let lost = leaf_ref(&index, &storage, root, 1, b"lose");
        for offset in [lost.primary, lost.mirror] {
            storage
                .write_at(offset, &vec![0; lost.len as usize])
                .unwrap();
        }
        storage
            .write_at(root.primary, &vec![0; root.len as usize])
            .unwrap();
        let selected = select(&storage, index.archive, index.mode, &authority).unwrap();
        let entry = index
            .get(
                &storage,
                selected.anchor.index,
                selected.anchor.sealed_len,
                1,
                b"keep",
            )
            .unwrap()
            .unwrap();
        let offset = u64::from_le_bytes(entry.value[..8].try_into().unwrap());
        let len = u64::from_le_bytes(entry.value[8..16].try_into().unwrap());
        let recovered = storage.read_at(offset, len as usize).unwrap();
        assert_eq!(strong_checksum(&recovered), entry.value[16..]);
        assert_eq!(recovered, data);
        assert!(index
            .get(&storage, root, first.sealed_len, 1, b"lose")
            .is_err());
        assert!(index
            .visit(&storage, root, first.sealed_len, |_| Ok(()))
            .is_err());
    }
}
#[test]
fn read_key_holder_cannot_replace_owner_authorized_nodes_or_reinstate_deleted_records() {
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let public = owner.public_key();
    let index = index(true, true);
    let authority = Authority::Owner(&public);
    let mut storage = StorageBackend::memory(vec![0; REGION_LEN]);
    let root = index.empty(&mut storage).unwrap();
    let old_root = put(&index, &mut storage, root, 1, b"secret", b"owner value").root;
    let first = anchor(&index, &storage, old_root, None);
    let old = publish(&mut storage, &first, &authority, Some(&owner), None).unwrap();
    let clean = storage.read_all().unwrap();
    let forged = index
        .encode(&Node::Leaf(
            Entry::new(1, b"secret", b"fake  value").unwrap(),
        ))
        .unwrap();
    assert_eq!(forged.len() as u64, old_root.len);
    for offset in [old_root.primary, old_root.mirror] {
        storage.write_at(offset, &forged).unwrap();
    }
    assert!(index
        .get(&storage, old_root, first.sealed_len, 1, b"secret")
        .is_err());
    let mut storage = StorageBackend::memory(clean);
    let root = remove(&index, &mut storage, old_root, 1, b"secret").root;
    let second = anchor(&index, &storage, root, Some(&first));
    publish(
        &mut storage,
        &second,
        &authority,
        Some(&owner),
        Some(old.commitment),
    )
    .unwrap();
    let prepared = put(&index, &mut storage, root, 1, b"secret", b"owner value").root;
    assert!(index
        .get(&storage, prepared, storage.len().unwrap(), 1, b"secret")
        .unwrap()
        .is_some());
    let selected = select(&storage, index.archive, index.mode, &authority).unwrap();
    assert_eq!(selected.anchor, second);
    assert!(index
        .get(
            &storage,
            selected.anchor.index,
            second.sealed_len,
            1,
            b"secret"
        )
        .unwrap()
        .is_none());
    storage.truncate(root.primary).unwrap();
    let selected = select(&storage, index.archive, index.mode, &authority).unwrap();
    assert_eq!(selected.anchor.generation, 2);
    assert!(index
        .get(
            &storage,
            selected.anchor.index,
            second.sealed_len,
            1,
            b"secret"
        )
        .is_err());
}
#[test]
fn insertion_and_publication_faults_select_only_complete_old_or_new_membership() {
    let index = index(true, true);
    let owner = OwnerSigningKeyPair::generate().unwrap();
    let public = owner.public_key();
    let authority = Authority::Owner(&public);
    let mut base = StorageBackend::memory(vec![0; REGION_LEN]);
    let mut root = index.empty(&mut base).unwrap();
    for key in 0u32..32 {
        root = put(&index, &mut base, root, 1, &key.to_le_bytes(), b"old").root;
    }
    let first = anchor(&index, &base, root, None);
    let published = publish(&mut base, &first, &authority, Some(&owner), None).unwrap();
    let bytes = base.read_all().unwrap();
    let perform = |storage: &mut StorageBackend| -> Result<()> {
        let change = index.put(
            storage,
            root,
            first.sealed_len,
            Entry::new(1, b"new identity", b"new")?,
        )?;
        let second = anchor(&index, storage, change.root, Some(&first));
        publish(
            storage,
            &second,
            &authority,
            Some(&owner),
            Some(published.commitment),
        )?;
        Ok(())
    };
    base.reset_memory_operation_count();
    perform(&mut base).unwrap();
    let operations = base.memory_operation_count();
    assert!(operations > 9);
    for failure in 0..operations {
        let mut storage = StorageBackend::memory(bytes.clone());
        storage.fail_memory_operation_after_successes(failure);
        assert!(perform(&mut storage).is_err(), "operation {failure}");
        let reopened = StorageBackend::memory(storage.read_all().unwrap());
        let selected = select(&reopened, index.archive, index.mode, &authority).unwrap();
        let actual = values(
            &index,
            &reopened,
            selected.anchor.index,
            selected.anchor.sealed_len,
        );
        assert_eq!(
            actual.len(),
            if selected.anchor.generation == 1 {
                32
            } else {
                33
            }
        );
        for key in 0u32..32 {
            assert_eq!(actual[&(1, key.to_le_bytes().to_vec())], b"old");
        }
        assert_eq!(
            actual.contains_key(&(1, b"new identity".to_vec())),
            selected.anchor.generation == 2
        );
    }
}
#[derive(Clone, Debug)]
struct Meter {
    inner: StorageBackend,
    reads: Cell<usize>,
    bytes: Cell<usize>,
    max_read: Cell<usize>,
}
impl Meter {
    fn reset(&self) {
        self.reads.set(0);
        self.bytes.set(0);
        self.max_read.set(0);
    }
}
impl Storage for Meter {
    fn len(&self) -> Result<u64> {
        self.inner.len()
    }
    fn read_at(&self, offset: u64, len: usize) -> Result<Vec<u8>> {
        self.reads.set(self.reads.get() + 1);
        self.bytes.set(self.bytes.get() + len);
        self.max_read.set(self.max_read.get().max(len));
        self.inner.read_at(offset, len)
    }
    fn read_at_into(&self, offset: u64, out: &mut [u8]) -> Result<()> {
        out.copy_from_slice(&self.read_at(offset, out.len())?);
        Ok(())
    }
    fn append(&mut self, bytes: &[u8]) -> Result<u64> {
        self.inner.append(bytes)
    }
    fn write_at(&mut self, offset: u64, bytes: &[u8]) -> Result<()> {
        self.inner.write_at(offset, bytes)
    }
    fn truncate(&mut self, len: u64) -> Result<()> {
        self.inner.truncate(len)
    }
    fn sync(&self) -> Result<()> {
        self.inner.sync()
    }
}
#[test]
fn incremental_updates_share_subtrees_and_bound_reads_without_a_whole_index_clone() {
    let index = index(false, false);
    let mut storage = Meter {
        inner: StorageBackend::memory(vec![0; REGION_LEN]),
        reads: Cell::new(0),
        bytes: Cell::new(0),
        max_read: Cell::new(0),
    };
    let mut root = index.empty(&mut storage).unwrap();
    let mut expected = Values::new();
    for key in 0u32..2048 {
        root = put(
            &index,
            &mut storage,
            root,
            1,
            &key.to_le_bytes(),
            b"descriptor",
        )
        .root;
        expected.insert((1, key.to_le_bytes().to_vec()), b"descriptor".to_vec());
    }
    let old = root;
    let before = storage.len().unwrap();
    storage.reset();
    let change = put(
        &index,
        &mut storage,
        root,
        1,
        &1024u32.to_le_bytes(),
        b"replacement",
    );
    assert!(change.created.len() < 32);
    assert_eq!(change.created.len(), change.retired.len());
    assert!(storage.reads.get() < 32);
    assert!(storage.bytes.get() < 16 * 1024);
    assert!(storage.len().unwrap() - before < 32 * 1024);
    assert!(storage.max_read.get() <= MAX_NODE);
    println!(
        "2048-entry replacement: nodes={} reads={} bytes_read={} bytes_appended={}",
        change.created.len(),
        storage.reads.get(),
        storage.bytes.get(),
        storage.len().unwrap() - before
    );
    root = change.root;
    expected.insert((1, 1024u32.to_le_bytes().to_vec()), b"replacement".to_vec());
    assert_eq!(
        values(&index, &storage, root, storage.len().unwrap()),
        expected
    );
    assert_eq!(
        index
            .get(&storage, old, before, 1, &1024u32.to_le_bytes())
            .unwrap()
            .unwrap()
            .value
            .as_slice(),
        b"descriptor"
    );
    for retired in change.retired {
        for offset in [retired.primary, retired.mirror] {
            storage
                .write_at(offset, &vec![0; retired.len as usize])
                .unwrap();
        }
    }
    assert_eq!(
        values(&index, &storage, root, storage.len().unwrap()),
        expected
    );
    for key in (0u32..2048).rev() {
        root = remove(&index, &mut storage, root, 1, &key.to_le_bytes()).root;
    }
    assert!(values(&index, &storage, root, storage.len().unwrap()).is_empty());
    assert!(matches!(
        index.read(&storage, root, storage.len().unwrap()).unwrap(),
        Node::Empty
    ));
}
#[test]
fn bounded_decoding_rejects_truncation_lengths_context_and_invalid_child_graphs() {
    let index = index(false, false);
    let mut storage = StorageBackend::memory(vec![0; REGION_LEN]);
    let root = index.empty(&mut storage).unwrap();
    let root = put(&index, &mut storage, root, 1, b"name", b"value").root;
    let bytes = root.read_verified(&storage).unwrap();
    for length in 0..bytes.len() {
        assert!(index
            .decode(&bytes[..length], storage.len().unwrap())
            .is_err());
    }
    for offset in [
        0,
        8,
        10,
        12,
        28,
        40,
        HEADER,
        HEADER + 1,
        HEADER + 10,
        HEADER + 12,
    ] {
        let mut bad = bytes.clone();
        bad[offset] ^= 0x80;
        assert!(
            index.decode(&bad, storage.len().unwrap()).is_err(),
            "field {offset}"
        );
    }
    let other = Index::new(LockboxId::from_bytes([46; 16]), index.mode, None).unwrap();
    assert!(other
        .get(&storage, root, storage.len().unwrap(), 1, b"name")
        .is_err());
    for bad in [
        RootRef {
            len: u64::MAX,
            ..root
        },
        RootRef {
            primary: u64::MAX,
            ..root
        },
        RootRef {
            mirror: root.primary,
            ..root
        },
        RootRef { primary: 0, ..root },
    ] {
        assert!(index
            .get(&storage, bad, storage.len().unwrap(), 1, b"name")
            .is_err());
    }
    let left = index
        .write(
            &mut storage,
            &Node::Leaf(Entry::new(1, b"left", b"v").unwrap()),
        )
        .unwrap();
    let right = index
        .write(
            &mut storage,
            &Node::Leaf(Entry::new(1, b"right", b"v").unwrap()),
        )
        .unwrap();
    let a = index.route(1, b"left");
    let b = index.route(1, b"right");
    let bit = first_difference(a, b).unwrap();
    let mut children = [left, right];
    if bit_at(a, bit) == 1 {
        children.swap(0, 1);
    }
    let good = branch(bit, prefix_of(a, bit), children).unwrap();
    let mut bytes = index.encode(&good).unwrap();
    bytes[HEADER + 1..HEADER + 9].copy_from_slice(&u64::MAX.to_le_bytes());
    assert!(index.decode(&bytes, storage.len().unwrap()).is_err());
    assert!(branch(bit, prefix_of(a, bit), [left, left]).is_err());
    children.swap(0, 1);
    let swapped = index
        .write(
            &mut storage,
            &branch(bit, prefix_of(a, bit), children).unwrap(),
        )
        .unwrap();
    assert!(index
        .get(
            &storage,
            swapped.reference,
            storage.len().unwrap(),
            1,
            b"left"
        )
        .is_err());
    assert!(index
        .visit(&storage, swapped.reference, storage.len().unwrap(), |_| Ok(
            ()
        ))
        .is_err());
    let before = storage.len().unwrap();
    assert!(Entry::new(1, &vec![0; MAX_KEY + 1], b"v").is_err());
    assert!(Entry::new(1, b"k", &vec![0; MAX_VALUE + 1]).is_err());
    assert_eq!(storage.len().unwrap(), before);
    let encrypted = self::index(true, true);
    let encrypted_root = encrypted.empty(&mut storage).unwrap();
    let wrong_key = Index::new(encrypted.archive, encrypted.mode, Some(&[18; 32])).unwrap();
    assert!(wrong_key
        .get(&storage, encrypted_root, storage.len().unwrap(), 1, b"name")
        .is_err());
}

#[test]
fn mixed_mutations_match_an_independent_map_and_maximum_records_round_trip() {
    let index = index(true, false);
    let mut storage = StorageBackend::memory(vec![0; REGION_LEN]);
    let mut root = index.empty(&mut storage).unwrap();
    let mut expected = Values::new();
    let mut random = 753u64;
    for step in 0u32..1024 {
        random = random.wrapping_mul(6364136223846793005).wrapping_add(1);
        let namespace = (random >> 40) as u8 % 8;
        let key = ((random >> 16) as u32 % 32).to_le_bytes().to_vec();
        if random & 3 == 0 {
            root = remove(&index, &mut storage, root, namespace, &key).root;
            expected.remove(&(namespace, key));
        } else {
            let value = step.to_le_bytes().to_vec();
            root = put(&index, &mut storage, root, namespace, &key, &value).root;
            expected.insert((namespace, key), value);
        }
        if step % 64 == 0 {
            assert_eq!(
                values(&index, &storage, root, storage.len().unwrap()),
                expected
            );
        }
    }
    assert_eq!(
        values(&index, &storage, root, storage.len().unwrap()),
        expected
    );
    let key = vec![0x5a; MAX_KEY];
    let value = vec![0xa5; MAX_VALUE];
    root = put(&index, &mut storage, root, 255, &key, &value).root;
    assert_eq!(
        index
            .get(&storage, root, storage.len().unwrap(), 255, &key)
            .unwrap()
            .unwrap()
            .value
            .as_slice(),
        value
    );
    root = put(&index, &mut storage, root, 0, b"", b"").root;
    assert!(index
        .get(&storage, root, storage.len().unwrap(), 0, b"")
        .unwrap()
        .unwrap()
        .value
        .is_empty());
}

#[test]
fn independently_encoded_index_vectors_match_wire_bytes_and_identity_routes() {
    fn bytes(hex: &serde_json::Value) -> Vec<u8> {
        hex.as_str()
            .unwrap()
            .as_bytes()
            .chunks_exact(2)
            .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
            .collect()
    }
    let vector: serde_json::Value = serde_json::from_str(include_str!(
        "../../tests/fixtures/authenticated_index_v1.json"
    ))
    .unwrap();
    let index = index(false, false);
    let mut storage = StorageBackend::memory(vec![0; REGION_LEN]);
    let mut last = RootRef::default();
    for record in vector["records"].as_array().unwrap() {
        let encoded = bytes(&record["bytes"]);
        let node = index.decode(&encoded, u64::MAX).unwrap();
        assert_eq!(*index.encode(&node).unwrap(), encoded);
        assert_eq!(
            strong_checksum(&encoded).as_slice(),
            bytes(&record["digest"])
        );
        if let Node::Leaf(entry) = &node {
            assert_eq!(
                index.route(entry.namespace, &entry.key).as_slice(),
                bytes(&record["route"])
            );
        }
        last = index.write(&mut storage, &node).unwrap().reference;
        assert_eq!(last.primary, record["primary"].as_u64().unwrap());
    }
    let found = values(&index, &storage, last, storage.len().unwrap());
    assert_eq!(found.len(), 2);
    assert_eq!(found[&(1, b"alpha".to_vec())], b"descriptor A");
    assert_eq!(found[&(2, b"beta".to_vec())], b"descriptor B");
}

/// Manual component resource observation, not a ZIP/archive performance gate.
/// Run one fresh process per mode after building, without concurrent workloads.
#[test]
#[ignore = "100,000-entry file-backed index resource observation; run explicitly in release mode"]
#[cfg(target_os = "linux")]
fn authenticated_index_resource_observation() {
    fn usage() -> (f64, f64, i64) {
        let mut value = std::mem::MaybeUninit::<libc::rusage>::uninit();
        // SAFETY: getrusage initializes the output on success, checked below.
        let status = unsafe { libc::getrusage(libc::RUSAGE_SELF, value.as_mut_ptr()) };
        assert_eq!(status, 0);
        // SAFETY: the successful getrusage call above initialized every field.
        let value = unsafe { value.assume_init() };
        let seconds = |time: libc::timeval| time.tv_sec as f64 + time.tv_usec as f64 / 1e6;
        (
            seconds(value.ru_utime),
            seconds(value.ru_stime),
            value.ru_maxrss,
        )
    }
    let encrypted =
        std::env::var("REVAULT_INDEX_RESOURCE_ENCRYPTED").is_ok_and(|value| value == "1");
    let index = index(encrypted, false);
    let dir = Directory::new();
    let path = dir.0.join("resource.candidate");
    let inner = StorageBackend::create_file(&path, &vec![0; REGION_LEN]).unwrap();
    let mut storage = Meter {
        inner,
        reads: Cell::new(0),
        bytes: Cell::new(0),
        max_read: Cell::new(0),
    };
    let mut root = index.empty(&mut storage).unwrap();
    let mut live_bytes = 2 * root.len;
    let before = usage();
    let started = std::time::Instant::now();
    for number in 0u32..100_000 {
        let value = [number.to_le_bytes().as_slice(), &[0x39; 92]].concat();
        let change = put(&index, &mut storage, root, 1, &number.to_le_bytes(), &value);
        live_bytes += change.created.iter().map(|r| 2 * r.len).sum::<u64>();
        live_bytes -= change.retired.iter().map(|r| 2 * r.len).sum::<u64>();
        root = change.root;
    }
    storage.sync().unwrap();
    let elapsed = started.elapsed().as_secs_f64();
    let after = usage();
    println!(
        "{}",
        serde_json::json!({"operation":"incremental_build", "encrypted":encrypted, "entries":100000, "wall_s":elapsed, "user_s":after.0-before.0, "system_s":after.1-before.1, "peak_rss_kib":after.2, "file_bytes":storage.len().unwrap(), "live_node_bytes":live_bytes, "retired_node_bytes":storage.len().unwrap()-REGION_LEN as u64-live_bytes, "reads":storage.reads.get(), "bytes_read":storage.bytes.get()})
    );
    // Updates are warm component observations, no owner signatures/publication.
    for sample in 0u32..30 {
        storage.reset();
        let len = storage.len().unwrap();
        let before = usage();
        let started = std::time::Instant::now();
        let key = (sample * 3319).to_le_bytes();
        let value = [key.as_slice(), &[0x51; 92]].concat();
        let change = put(&index, &mut storage, root, 1, &key, &value);
        root = change.root;
        let elapsed = started.elapsed().as_secs_f64();
        let after = usage();
        println!(
            "{}",
            serde_json::json!({"operation":"replace", "encrypted":encrypted, "sample":sample, "wall_s":elapsed, "user_s":after.0-before.0, "system_s":after.1-before.1, "peak_rss_kib":after.2, "nodes_created":change.created.len(), "bytes_appended":storage.len().unwrap()-len, "reads":storage.reads.get(), "bytes_read":storage.bytes.get()})
        );
    }
    storage.sync().unwrap();
    let sealed = storage.len().unwrap();
    drop(storage);
    let reopened = StorageBackend::file(&path).unwrap();
    let mut entries = 0;
    index
        .visit(&reopened, root, sealed, |entry| {
            let number = u32::from_le_bytes(entry.key.as_slice().try_into().unwrap());
            let fill = if number % 3319 == 0 && number / 3319 < 30 {
                0x51
            } else {
                0x39
            };
            assert_eq!(entry.namespace, 1);
            assert_eq!(&entry.value[..4], entry.key.as_slice());
            assert_eq!(entry.value[4..], [fill; 92]);
            entries += 1;
            Ok(())
        })
        .unwrap();
    assert_eq!(entries, 100_000);
}

#[test]
fn padding_policy_preserves_constant_node_lengths_and_rejects_noncanonical_padding() {
    for encrypted in [false, true] {
        for signed in [false, true] {
            let mut options = mode(encrypted, signed).options();
            options.size_padding = SizePadding::Default;
            let padded = Index::new(
                LockboxId::from_bytes([45; 16]),
                FormatMode::new(options),
                encrypted.then_some(&[17; 32][..]),
            )
            .unwrap();
            let mut storage = StorageBackend::memory(vec![0; REGION_LEN]);
            let root = padded.empty(&mut storage).unwrap();
            assert_eq!(root.len, MAX_NODE as u64);
            let change = put(&padded, &mut storage, root, 1, b"key", &vec![27; MAX_VALUE]);
            assert!(change
                .created
                .iter()
                .all(|reference| reference.len == MAX_NODE as u64));
            let root = put(&padded, &mut storage, change.root, 1, b"small", b"tiny").root;
            assert_eq!(
                values(&padded, &storage, root, storage.len().unwrap()).len(),
                2
            );
            let compact = index(encrypted, signed);
            assert!(compact
                .get(&storage, root, storage.len().unwrap(), 1, b"small")
                .is_err());
            let mut bad = padded.encode(&Node::Empty).unwrap();
            *bad.last_mut().unwrap() ^= 1;
            assert!(padded.decode(&bad, storage.len().unwrap()).is_err());
        }
    }
}
