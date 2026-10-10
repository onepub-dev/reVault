//! An admitted selected-payload read capability, not a payload authentication key.
use super::*;
use std::fmt;

pub(crate) struct SelectedSource {
    base: [u8; 32],
    archive: LockboxId,
    mode: FormatMode,
    sealed: u64,
    extents: Vec<Extent>,
}
impl SelectedSource {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn open(
        storage: &impl Storage,
        archive: LockboxId,
        mode: FormatMode,
        authority: &Authority<'_>,
        key: Option<&[u8]>,
        expected_base: [u8; 32],
        mut extents: Vec<Extent>,
    ) -> Result<Self> {
        if extents.len() > MAX_OWNERSHIP_RECORDS {
            return Err(Error::SecurityLimitExceeded(
                "selected source extent budget".into(),
            ));
        }
        extents.sort_by_key(|e| e.start);
        if extents.windows(2).any(|p| p[0].start == p[1].start) {
            return Err(Error::CorruptRecord);
        }
        let (anchor, body) = open_private(storage, archive, mode, authority, key)?;
        let tree = Tree::from_snapshot_visit(
            storage,
            archive,
            mode,
            key,
            anchor,
            &body,
            |_| Ok(()),
            &mut |_| {},
        )?;
        let source = Self {
            base: expected_base,
            archive,
            mode,
            sealed: tree.anchor.sealed_len,
            extents,
        };
        source.validate(&tree)?;
        tree.graph.verify_reclaimed(storage)?;
        Ok(source)
    }
    pub(super) fn validate(&self, tree: &Tree) -> Result<()> {
        if tree.anchor.archive != self.archive
            || tree.anchor.mode != self.mode
            || tree.anchor.sealed_len != self.sealed
            || commitment(&tree.anchor)? != self.base
        {
            return Err(Error::CorruptRecord);
        }
        let live: std::collections::BTreeMap<_, _> = tree
            .graph
            .payloads()
            .into_iter()
            .map(|e| (e.start, e))
            .collect();
        for extent in &self.extents {
            if extent.len == 0
                || extent
                    .start
                    .checked_add(extent.len)
                    .ok_or(Error::CorruptRecord)?
                    > self.sealed
                || live.get(&extent.start) != Some(extent)
            {
                return Err(Error::CorruptRecord);
            }
        }
        Ok(())
    }
    pub(crate) fn view<'a>(&'a self, storage: &'a impl Storage) -> ReadView<'a> {
        ReadView {
            source: storage,
            sealed: self.sealed,
            extents: &self.extents,
        }
    }
    pub(super) fn base(&self) -> [u8; 32] {
        self.base
    }
}
trait SourceRead: fmt::Debug {
    fn source_len(&self) -> Result<u64>;
    fn read_into(&self, at: u64, out: &mut [u8]) -> Result<()>;
}
impl<S: Storage> SourceRead for S {
    fn source_len(&self) -> Result<u64> {
        Storage::len(self)
    }
    fn read_into(&self, at: u64, out: &mut [u8]) -> Result<()> {
        Storage::read_at_into(self, at, out)
    }
}
/// Clone copies only references. No raw constructor bypasses SelectedSource admission.
#[derive(Clone, Copy)]
pub(crate) struct ReadView<'a> {
    source: &'a dyn SourceRead,
    sealed: u64,
    extents: &'a [Extent],
}
impl fmt::Debug for ReadView<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SelectedReadView")
            .field("sealed", &self.sealed)
            .field("extents", &self.extents.len())
            .finish()
    }
}
fn denied<T>() -> Result<T> {
    Err(Error::InvalidOperation(
        "selected source view permits guarded reads only".into(),
    ))
}
impl Storage for ReadView<'_> {
    fn len(&self) -> Result<u64> {
        if self.source.source_len()? < self.sealed {
            return Err(Error::Truncated);
        }
        Ok(self.sealed)
    }
    fn read_at(&self, _: u64, _: usize) -> Result<Vec<u8>> {
        denied()
    }
    fn read_at_into(&self, at: u64, out: &mut [u8]) -> Result<()> {
        let end = at
            .checked_add(out.len() as u64)
            .ok_or(Error::CorruptRecord)?;
        if end > self.len()?
            || !self
                .extents
                .iter()
                .any(|e| at >= e.start && end <= e.start + e.len)
        {
            return Err(Error::CorruptRecord);
        }
        self.source.read_into(at, out)
    }
    fn append(&mut self, _: &[u8]) -> Result<u64> {
        denied()
    }
    fn write_at(&mut self, _: u64, _: &[u8]) -> Result<()> {
        denied()
    }
    fn truncate(&mut self, _: u64) -> Result<()> {
        denied()
    }
    fn sync(&self) -> Result<()> {
        denied()
    }
}
