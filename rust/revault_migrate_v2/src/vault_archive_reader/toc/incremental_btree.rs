use std::collections::BTreeSet;

/// Persisted topology and dirty-key set shared by the domain B-trees.
#[derive(Debug, Clone)]
pub(crate) struct PersistedBTree<Node, Leaf, Key> {
    pub(crate) root_offset: u64,
    pub(crate) root: Option<Node>,
    pub(crate) leaves: Vec<Leaf>,
    pub(crate) dirty_keys: BTreeSet<Key>,
}

impl<Node, Leaf, Key> Default for PersistedBTree<Node, Leaf, Key> {
    fn default() -> Self {
        Self {
            root_offset: 0,
            root: None,
            leaves: Vec::new(),
            dirty_keys: BTreeSet::new(),
        }
    }
}

impl<Node, Leaf, Key> PersistedBTree<Node, Leaf, Key> {
    pub(crate) fn is_clean(&self) -> bool {
        self.dirty_keys.is_empty()
    }

    pub(crate) fn replace_topology(&mut self, root: Node, leaves: Vec<Leaf>) {
        self.root = Some(root);
        self.leaves = leaves;
    }

    pub(crate) fn clear(&mut self) {
        self.root_offset = 0;
        self.root = None;
        self.leaves.clear();
        self.dirty_keys.clear();
    }
}

/// An entry stored in an incrementally rewritten B-tree leaf.
pub(crate) trait BTreeEntry {
    type Key: Ord;

    fn key(&self) -> &Self::Key;
}

/// A persisted B-tree leaf whose entries can be reused by a later commit.
pub(crate) trait BTreeLeaf<Entry> {
    fn entries(&self) -> &[Entry];
}

/// One leaf-level action in an incremental B-tree rewrite.
pub(crate) enum LeafRewrite<Leaf, Entry> {
    Reuse(Leaf),
    Write(Vec<Entry>),
}

/// Plans which B-tree leaves can be reused and which entry ranges must be written.
///
/// The planner owns the comparison between the new ordered entry set, the
/// persisted leaf directory, and the keys dirtied by the current transaction.
/// Tree-specific code remains responsible for page sizing, encoding, and I/O.
pub(crate) struct IncrementalBTree<'a, Entry, Leaf>
where
    Entry: BTreeEntry,
{
    entries: &'a [Entry],
    old_leaves: &'a [Leaf],
    dirty_keys: &'a BTreeSet<Entry::Key>,
}

impl<'a, Entry, Leaf> IncrementalBTree<'a, Entry, Leaf>
where
    Entry: BTreeEntry + Clone,
    Leaf: BTreeLeaf<Entry> + Clone,
{
    pub(crate) fn new(
        entries: &'a [Entry],
        old_leaves: &'a [Leaf],
        dirty_keys: &'a BTreeSet<Entry::Key>,
    ) -> Self {
        Self {
            entries,
            old_leaves,
            dirty_keys,
        }
    }

    pub(crate) fn plan(
        self,
        can_reuse: impl Fn(&[Entry], &[Entry]) -> bool,
    ) -> Vec<LeafRewrite<Leaf, Entry>> {
        let mut rewrites = Vec::new();
        let mut cursor = 0usize;

        for (index, leaf) in self.old_leaves.iter().enumerate() {
            let Some(first) = leaf.entries().first().map(BTreeEntry::key) else {
                continue;
            };
            let next = self
                .old_leaves
                .get(index + 1)
                .and_then(|leaf| leaf.entries().first())
                .map(BTreeEntry::key);

            while cursor < self.entries.len() && self.entries[cursor].key() < first {
                let chunk_start = cursor;
                cursor += 1;
                while cursor < self.entries.len()
                    && self.entries[cursor].key() < first
                    && !self.dirty_keys.contains(self.entries[cursor].key())
                {
                    cursor += 1;
                }
                rewrites.push(LeafRewrite::Write(
                    self.entries[chunk_start..cursor].to_vec(),
                ));
            }

            let start = cursor;
            while cursor < self.entries.len()
                && next.is_none_or(|next| self.entries[cursor].key() < next)
            {
                cursor += 1;
            }
            let replacement_entries = &self.entries[start..cursor];
            let overlaps_dirty = replacement_entries
                .iter()
                .any(|entry| self.dirty_keys.contains(entry.key()))
                || self
                    .dirty_keys
                    .iter()
                    .any(|key| key >= first && next.is_none_or(|next| key < next));
            if !overlaps_dirty && can_reuse(leaf.entries(), replacement_entries) {
                rewrites.push(LeafRewrite::Reuse(leaf.clone()));
            } else {
                rewrites.push(LeafRewrite::Write(replacement_entries.to_vec()));
            }
        }

        if cursor < self.entries.len() {
            rewrites.push(LeafRewrite::Write(self.entries[cursor..].to_vec()));
        }
        rewrites
    }
}

#[cfg(test)]
mod tests {
    use super::{BTreeEntry, BTreeLeaf, IncrementalBTree, LeafRewrite};
    use std::collections::BTreeSet;

    #[derive(Clone, Debug, PartialEq, Eq)]
    struct Entry(String);

    impl BTreeEntry for Entry {
        type Key = String;

        fn key(&self) -> &Self::Key {
            &self.0
        }
    }

    #[derive(Clone, Debug)]
    struct Leaf(Vec<Entry>);

    impl BTreeLeaf<Entry> for Leaf {
        fn entries(&self) -> &[Entry] {
            &self.0
        }
    }

    #[test]
    fn reuses_clean_leaves_and_rewrites_dirty_ranges() {
        let entries = ["a", "b", "c", "d"].map(|value| Entry(value.to_string()));
        let leaves = vec![Leaf(entries[..2].to_vec()), Leaf(entries[2..].to_vec())];
        let dirty = BTreeSet::from(["c".to_string()]);

        let plan = IncrementalBTree::new(&entries, &leaves, &dirty).plan(|old, new| old == new);

        assert!(matches!(&plan[0], LeafRewrite::Reuse(_)));
        assert!(matches!(&plan[1], LeafRewrite::Write(entries) if entries.len() == 2));
    }

    #[test]
    fn writes_entries_inserted_before_the_first_persisted_leaf() {
        let entries = ["a", "b", "c"].map(|value| Entry(value.to_string()));
        let leaves = vec![Leaf(entries[1..].to_vec())];
        let dirty = BTreeSet::from(["a".to_string()]);

        let plan = IncrementalBTree::new(&entries, &leaves, &dirty).plan(|old, new| old == new);

        assert!(
            matches!(&plan[0], LeafRewrite::Write(entries) if entries == &[Entry("a".to_string())])
        );
        assert!(matches!(&plan[1], LeafRewrite::Reuse(_)));
    }
}
