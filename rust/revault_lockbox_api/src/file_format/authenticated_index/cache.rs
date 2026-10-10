//! Bounded cache of already authenticated decoded pages for one read snapshot.
//! Keys include both locations, length and digest. No filesystem timestamps or
//! external integrity cache are trusted. Entry/branch buffers wipe on eviction.
use super::*;
use std::collections::VecDeque;
const CACHE_PAGES: usize = 16;

#[derive(Default)]
pub(crate) struct PageCache {
    pages: VecDeque<(RootRef, Node)>,
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
                    let node = self.read(storage, reference, sealed)?;
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
                Node::Leaf(entries) => {
                    return Ok(entries
                        .binary_search_by(|e| e.identity().cmp(&identity))
                        .ok()
                        .map(|i| entries[i].clone()))
                }
                Node::Branch { height, children } => {
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
