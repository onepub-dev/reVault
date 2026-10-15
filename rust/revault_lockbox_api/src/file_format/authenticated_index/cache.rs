//! Bounded cache of already authenticated decoded pages for one read snapshot.
//! Keys include both locations, length and digest. No filesystem timestamps or
//! external integrity cache are trusted. Entry/branch buffers wipe on eviction.
use super::*;
use std::collections::VecDeque;
const CACHE_PAGES: usize = 16;

#[derive(Default)]
pub(crate) struct PageCache {
    pages: VecDeque<(RootRef, CachedNode)>,
    scope: Option<(LockboxId, u16, RootRef, u64)>,
    key: Option<Zeroizing<[u8; 32]>>,
}
impl Index {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn get_cached(
        &self,
        storage: &impl Storage,
        root: RootRef,
        sealed: u64,
        namespace: u8,
        key: &[u8],
        cache: &mut PageCache,
    ) -> Result<Option<Entry>> {
        validate_input(key, &[])?;
        let scope = (self.archive, self.mode.0, root, sealed);
        if cache.scope != Some(scope) || cache.key.as_deref() != self.key.as_deref() {
            cache.pages.clear();
            cache.scope = Some(scope);
            cache.key = self.key.clone();
        }
        let identity = (namespace, key);
        let mut reference = root;
        let mut expected: Option<(u8, Link)> = None;
        let mut upper: Option<Key> = None;
        loop {
            // Bounds are checked even on a hit: a cached page is not authority
            // for a different sealed length or parent-child relationship.
            validate_ref(reference, sealed)?;
            let position =
                if let Some(position) = cache.pages.iter().position(|(r, _)| *r == reference) {
                    position
                } else {
                    let node = self.read_cached_node(storage, reference, sealed)?;
                    if cache.pages.len() == CACHE_PAGES {
                        cache.pages.pop_front();
                    }
                    cache.pages.push_back((reference, node));
                    cache.pages.len() - 1
                };
            let node = &cache.pages[position].1;
            if let Some((height, link)) = &expected {
                if node.height() + 1 != *height
                    || node.count() != link.count
                    || node.first() != Some(link.first.identity())
                    || node.last().is_some_and(|last| {
                        upper.as_ref().is_some_and(|bound| last >= bound.identity())
                    })
                {
                    return Err(Error::CorruptRecord);
                }
            }
            match node {
                CachedNode::Leaf(leaf) => return leaf.get(identity),
                CachedNode::Branch { height, children } => {
                    let position = children.partition_point(|c| c.first.identity() <= identity);
                    if position == 0 {
                        return Ok(None);
                    }
                    let position = position - 1;
                    if let Some(next) = children.get(position + 1) {
                        upper = Some(next.first.clone());
                    }
                    reference = children[position].reference;
                    expected = Some((*height, children[position].clone()));
                }
            }
        }
    }
}

#[test]
fn page_cache_evicts_at_its_fixed_bound_and_preserves_exact_lookups() {
    // Private index fixture; the public CLI cannot expose cache occupancy.
    let mode = crate::creation_options::FormatMode::new(crate::LockboxFormatOptions {
        encryption: crate::EncryptionMode::None,
        signing: crate::SigningMode::None,
        compression: crate::Compression::None,
        size_padding: crate::SizePadding::Default,
    });
    let index = Index::new(LockboxId::from_bytes([81; 16]), mode, None).unwrap();
    let mut storage = crate::storage::StorageBackend::memory(vec![0; REGION_LEN]);
    let root = index
        .build_sorted(
            &mut storage,
            (0u32..1000).map(|n| Entry::new(1, &n.to_be_bytes(), &vec![n as u8; 1500])),
        )
        .unwrap()
        .root;
    let sealed = storage.len().unwrap();
    let mut cache = PageCache::default();
    for n in (0u32..1000).chain((0u32..1000).rev()) {
        let entry = index
            .get_cached(&storage, root, sealed, 1, &n.to_be_bytes(), &mut cache)
            .unwrap()
            .unwrap();
        assert_eq!(entry.value.as_slice(), &vec![n as u8; 1500]);
        assert!(cache.pages.len() <= CACHE_PAGES);
    }
    assert_eq!(cache.pages.len(), CACHE_PAGES);
}

// One compact allocation owns the private bytes in a decoded leaf. Offsets
// replace per-record key/value allocations without retaining borrowed storage.
// The shared decoder validates the entire page before this representation exists.
pub(super) enum CachedNode {
    Leaf(Leaf),
    Branch { height: u8, children: Vec<Link> },
}
pub(super) struct Leaf {
    bytes: ZeroizingBytes,
    rows: Vec<Row>,
}
struct Row {
    namespace: u8,
    start: usize,
    key_end: usize,
    end: usize,
}
impl Leaf {
    pub(super) fn new(entries: Vec<EntryRef<'_>>) -> Self {
        let size = entries.iter().map(|e| e.key.len() + e.value.len()).sum();
        let mut bytes = ZeroizingBytes::new(Vec::with_capacity(size));
        let mut rows = Vec::with_capacity(entries.len());
        for entry in entries {
            let start = bytes.len();
            bytes.extend_from_slice(entry.key);
            let key_end = bytes.len();
            bytes.extend_from_slice(entry.value);
            rows.push(Row {
                namespace: entry.namespace,
                start,
                key_end,
                end: bytes.len(),
            });
        }
        Self { bytes, rows }
    }
    fn identity(&self, row: &Row) -> (u8, &[u8]) {
        (row.namespace, &self.bytes[row.start..row.key_end])
    }
    fn get(&self, identity: (u8, &[u8])) -> Result<Option<Entry>> {
        self.rows
            .binary_search_by(|row| self.identity(row).cmp(&identity))
            .ok()
            .map(|i| {
                let row = &self.rows[i];
                Entry::new(
                    row.namespace,
                    &self.bytes[row.start..row.key_end],
                    &self.bytes[row.key_end..row.end],
                )
            })
            .transpose()
    }
}
impl CachedNode {
    fn height(&self) -> u8 {
        match self {
            Self::Leaf(_) => 0,
            Self::Branch { height, .. } => *height,
        }
    }
    fn count(&self) -> u64 {
        match self {
            Self::Leaf(leaf) => leaf.rows.len() as u64,
            Self::Branch { children, .. } => children.iter().map(|c| c.count).sum(),
        }
    }
    fn first(&self) -> Option<(u8, &[u8])> {
        match self {
            Self::Leaf(leaf) => leaf.rows.first().map(|row| leaf.identity(row)),
            Self::Branch { children, .. } => children.first().map(|c| c.first.identity()),
        }
    }
    fn last(&self) -> Option<(u8, &[u8])> {
        match self {
            Self::Leaf(leaf) => leaf.rows.last().map(|row| leaf.identity(row)),
            Self::Branch { children, .. } => children.last().map(|c| c.first.identity()),
        }
    }
}
impl Index {
    fn read_cached_node(
        &self,
        storage: &impl Storage,
        reference: RootRef,
        sealed: u64,
    ) -> Result<CachedNode> {
        validate_ref(reference, sealed)?;
        let bytes = ZeroizingBytes::new(reference.read_verified(storage)?);
        self.decode_cached_node(&bytes, sealed)
    }
}

#[test]
fn compact_leaf_preserves_binary_empty_and_boundary_records_all_modes() {
    // Private representation check: experimental index pages cannot be created
    // by the public CLI. Both representations use the same validating decoder.
    use crate::{Compression, EncryptionMode, LockboxFormatOptions, SigningMode, SizePadding};
    let archive = LockboxId::from_bytes([83; 16]);
    for bits in 0..16 {
        let encrypted = bits & 1 != 0;
        let mode = crate::creation_options::FormatMode::new(LockboxFormatOptions {
            encryption: if encrypted {
                EncryptionMode::ChaCha20Poly1305
            } else {
                EncryptionMode::None
            },
            signing: if bits & 2 != 0 {
                SigningMode::Owner
            } else {
                SigningMode::None
            },
            compression: if bits & 4 != 0 {
                Compression::default()
            } else {
                Compression::None
            },
            size_padding: if bits & 8 != 0 {
                SizePadding::None
            } else {
                SizePadding::Default
            },
        });
        let index = Index::new(archive, mode, encrypted.then_some([84; 32].as_slice())).unwrap();
        for entries in [
            vec![],
            vec![
                Entry::new(0, &[], &[]).unwrap(),
                Entry::new(1, &[], &[0, 255, 0]).unwrap(),
                Entry::new(255, &[255; MAX_KEY], &[0; MAX_VALUE]).unwrap(),
            ],
            (0u32..MAX_ITEMS as u32)
                .map(|n| Entry::new(3, &n.to_be_bytes(), &[n as u8; 8]).unwrap())
                .collect(),
        ] {
            let bytes = index.encode(&Node::Leaf(entries.clone())).unwrap();
            let compact = index.decode_cached_node(&bytes, u64::MAX).unwrap();
            let CachedNode::Leaf(leaf) = compact else {
                panic!("leaf expected")
            };
            assert_eq!(leaf.rows.len(), entries.len());
            assert_eq!(
                leaf.bytes.len(),
                entries
                    .iter()
                    .map(|e| e.key.len() + e.value.len())
                    .sum::<usize>()
            );
            assert!(leaf.bytes.len() <= MAX_BODY);
            for expected in entries {
                let actual = leaf.get(expected.identity()).unwrap().unwrap();
                assert_eq!(actual.identity(), expected.identity());
                assert_eq!(actual.value, expected.value);
            }
            assert!(leaf.get((254, b"absent")).unwrap().is_none());
            assert!(index
                .decode_cached_node(&bytes[..bytes.len() - 1], u64::MAX)
                .is_err());
        }
    }
}
