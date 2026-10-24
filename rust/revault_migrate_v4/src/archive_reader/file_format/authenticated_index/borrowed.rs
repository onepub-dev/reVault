//! Page-scoped record views. No view outlives its zeroizing decoded page.
use super::*;

#[derive(Clone, Copy)]
pub(crate) struct EntryRef<'a> {
    pub namespace: u8,
    pub key: &'a [u8],
    pub value: &'a [u8],
}
impl EntryRef<'_> {
    pub(crate) fn to_owned(self) -> Result<Entry> {
        Entry::new(self.namespace, self.key, self.value)
    }
    fn identity(&self) -> (u8, &[u8]) {
        (self.namespace, self.key)
    }
}
impl Entry {
    pub(crate) fn as_ref(&self) -> EntryRef<'_> {
        EntryRef {
            namespace: self.namespace,
            key: &self.key,
            value: &self.value,
        }
    }
}
pub(crate) enum VisitRef<'a> {
    Page(RootRef),
    Entry(EntryRef<'a>),
}
enum NodeView<'a> {
    Leaf(Vec<EntryRef<'a>>),
    Branch(Node),
}
impl NodeView<'_> {
    fn height(&self) -> u8 {
        match self {
            Self::Leaf(_) => 0,
            Self::Branch(node) => node.height(),
        }
    }
    fn count(&self) -> u64 {
        match self {
            Self::Leaf(entries) => entries.len() as u64,
            Self::Branch(node) => node.count(),
        }
    }
    fn first(&self) -> Option<(u8, &[u8])> {
        match self {
            Self::Leaf(entries) => entries.first().map(EntryRef::identity),
            Self::Branch(node) => node.first(),
        }
    }
    fn last(&self) -> Option<(u8, &[u8])> {
        match self {
            Self::Leaf(entries) => entries.last().map(EntryRef::identity),
            Self::Branch(node) => node.last(),
        }
    }
    fn validate(&self, sealed: u64) -> Result<()> {
        match self {
            Self::Branch(node) => validate_node(node, sealed),
            Self::Leaf(entries) => {
                if entries.len() > MAX_ITEMS
                    || entries.len() as u64 > MAX_ENTRIES
                    || BODY_HEADER
                        + entries
                            .iter()
                            .map(|entry| 7 + entry.key.len() + entry.value.len())
                            .sum::<usize>()
                        > MAX_BODY
                {
                    return Err(Error::CorruptRecord);
                }
                for entry in entries {
                    validate_input(entry.key, entry.value)?;
                }
                if entries
                    .windows(2)
                    .any(|pair| pair[0].identity() >= pair[1].identity())
                {
                    return Err(Error::CorruptRecord);
                }
                Ok(())
            }
        }
    }
    fn into_owned(self) -> Result<Node> {
        match self {
            Self::Leaf(entries) => entries
                .into_iter()
                .map(EntryRef::to_owned)
                .collect::<Result<Vec<_>>>()
                .map(Node::Leaf),
            Self::Branch(node) => Ok(node),
        }
    }
}
impl Index {
    pub(super) fn decode(&self, bytes: &[u8], sealed: u64) -> Result<Node> {
        self.with_decoded(bytes, sealed, |node| node.into_owned())
    }
    pub(super) fn decode_cached_node(
        &self,
        bytes: &[u8],
        sealed: u64,
    ) -> Result<super::cache::CachedNode> {
        self.with_decoded(bytes, sealed, |node| {
            Ok(match node {
                NodeView::Leaf(entries) => {
                    super::cache::CachedNode::Leaf(super::cache::Leaf::new(entries))
                }
                NodeView::Branch(Node::Branch { height, children }) => {
                    super::cache::CachedNode::Branch { height, children }
                }
                NodeView::Branch(Node::Leaf(_)) => {
                    unreachable!("branch decoder yields branch nodes")
                }
            })
        })
    }
    fn with_decoded<T>(
        &self,
        bytes: &[u8],
        sealed: u64,
        consume: impl FnOnce(NodeView<'_>) -> Result<T>,
    ) -> Result<T> {
        if bytes.len() < HEADER
            || bytes.len() > MAX_NODE
            || (!self.mode.unpadded() && bytes.len() != MAX_NODE)
            || &bytes[..8] != MAGIC
            || bytes[8..10] != 1u16.to_le_bytes()
            || bytes[10..12] != self.mode.0.to_le_bytes()
            || bytes[12..28] != *self.archive.as_bytes()
            || u32::from_le_bytes(bytes[40..44].try_into().unwrap()) as usize
                != bytes.len() - HEADER
        {
            return Err(Error::CorruptRecord);
        }
        let body = if let Some(key) = &self.key {
            ZeroizingBytes::new(open_with_nonce(
                &bytes[HEADER..],
                key.as_slice(),
                &bytes[28..40],
                &bytes[..28],
            )?)
        } else {
            if bytes[28..40] != [0; 12] {
                return Err(Error::CorruptRecord);
            }
            ZeroizingBytes::new(bytes[HEADER..].to_vec())
        };
        let mut cursor = Cursor(&body);
        let height = cursor.take(1)?[0];
        let count = cursor.u64()?;
        let items = cursor.u16()? as usize;
        if height > MAX_HEIGHT || count > MAX_ENTRIES || items > MAX_ITEMS {
            return Err(Error::CorruptRecord);
        }
        let node = if height == 0 {
            let mut entries = Vec::with_capacity(items);
            for _ in 0..items {
                let namespace = cursor.take(1)?[0];
                let key_len = cursor.u16()? as usize;
                let value_len = cursor.u32()? as usize;
                if key_len > MAX_KEY || value_len > MAX_VALUE {
                    return Err(Error::CorruptRecord);
                }
                entries.push(EntryRef {
                    namespace,
                    key: cursor.take(key_len)?,
                    value: cursor.take(value_len)?,
                });
            }
            NodeView::Leaf(entries)
        } else {
            let mut children = Vec::with_capacity(items);
            for _ in 0..items {
                children.push(cursor.link(sealed)?);
            }
            NodeView::Branch(Node::Branch { height, children })
        };
        node.validate(sealed)?;
        if count != node.count()
            || if self.mode.unpadded() {
                !cursor.0.is_empty()
            } else {
                cursor.0.iter().any(|b| *b != 0)
            }
        {
            return Err(Error::CorruptRecord);
        }
        consume(node)
    }
    /// Every page is fully decoded and validated before its callbacks. Parent
    /// relationships are checked before page/entry exposure. Callback output
    /// remains staged: a later descendant failure invalidates the entire walk.
    pub(crate) fn visit_borrowed(
        &self,
        storage: &impl Storage,
        root: RootRef,
        sealed: u64,
        mut visitor: impl FnMut(VisitRef<'_>) -> Result<()>,
    ) -> Result<()> {
        self.visit_borrowed_node(storage, root, sealed, None, &mut visitor)
    }
    fn visit_borrowed_node(
        &self,
        storage: &impl Storage,
        reference: RootRef,
        sealed: u64,
        expected: Option<(u8, &Link, Option<&Key>)>,
        visitor: &mut impl FnMut(VisitRef<'_>) -> Result<()>,
    ) -> Result<()> {
        // The page buffers and leaf views drop before descending into a branch;
        // only its bounded child links survive, as in the owned traversal.
        let branch = {
            validate_ref(reference, sealed)?;
            let bytes = ZeroizingBytes::new(reference.read_verified(storage)?);
            self.with_decoded(&bytes, sealed, |node| {
                if let Some((height, link, upper)) = expected {
                    if node.height() + 1 != height
                        || node.count() != link.count
                        || node.first() != Some(link.first.identity())
                        || node
                            .last()
                            .is_some_and(|last| upper.is_some_and(|upper| last >= upper.identity()))
                    {
                        return Err(Error::CorruptRecord);
                    }
                }
                visitor(VisitRef::Page(reference))?;
                match node {
                    NodeView::Leaf(entries) => {
                        for entry in entries {
                            visitor(VisitRef::Entry(entry))?;
                        }
                        Ok(None)
                    }
                    NodeView::Branch(node) => Ok(Some(node)),
                }
            })?
        };
        if let Some(Node::Branch { height, children }) = branch {
            for (position, link) in children.iter().enumerate() {
                let upper = children
                    .get(position + 1)
                    .map(|next| &next.first)
                    .or_else(|| expected.and_then(|(_, _, upper)| upper));
                self.visit_borrowed_node(
                    storage,
                    link.reference,
                    sealed,
                    Some((height, link, upper)),
                    visitor,
                )?;
            }
        }
        Ok(())
    }
}
