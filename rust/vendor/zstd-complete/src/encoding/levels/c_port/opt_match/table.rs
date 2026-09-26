use crate::workspace::{Arena, ArenaError, ArenaSize, ReusableVec};
use core::ops::Index;

const ZSTD_OPT_NUM: usize = 1 << 12;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::encoding::levels::c_port) struct OptMatch {
    pub(in crate::encoding::levels::c_port) off_base: u32,
    pub(in crate::encoding::levels::c_port) len: u32,
}

impl OptMatch {
    const EMPTY: Self = Self {
        off_base: 0,
        len: 0,
    };
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::encoding::levels::c_port) struct OptMatchTable {
    entries: ReusableVec<OptMatch>,
    len: usize,
}

impl OptMatchTable {
    pub(in crate::encoding::levels::c_port) fn add_workspace_size(
        size: &mut ArenaSize,
    ) -> Result<(), ArenaError> {
        size.add::<OptMatch>(ZSTD_OPT_NUM)
    }

    pub(in crate::encoding::levels::c_port) fn new_in(
        arena: &mut Arena<'_>,
    ) -> Result<Self, ArenaError> {
        let mut entries = arena.allocate_reusable_vec(ZSTD_OPT_NUM)?;
        entries.resize(ZSTD_OPT_NUM, OptMatch::EMPTY);
        Ok(Self { entries, len: 0 })
    }

    pub(in crate::encoding::levels::c_port) fn new() -> Self {
        let mut entries = ReusableVec::with_capacity(ZSTD_OPT_NUM);
        entries.resize(ZSTD_OPT_NUM, OptMatch::EMPTY);
        Self { entries, len: 0 }
    }

    #[inline(always)]
    pub(in crate::encoding::levels::c_port) fn clear(&mut self) {
        self.len = 0;
    }

    #[inline(always)]
    pub(in crate::encoding::levels::c_port) fn len(&self) -> usize {
        self.len
    }

    #[inline(always)]
    pub(in crate::encoding::levels::c_port) fn is_empty(&self) -> bool {
        self.len == 0
    }

    #[inline(always)]
    pub(in crate::encoding::levels::c_port) fn push(&mut self, value: OptMatch) {
        debug_assert!(self.len < ZSTD_OPT_NUM);
        self.entries[self.len] = value;
        self.len += 1;
    }

    #[inline(always)]
    pub(in crate::encoding::levels::c_port) fn last(&self) -> Option<&OptMatch> {
        if self.len == 0 {
            None
        } else {
            Some(&self.entries[self.len - 1])
        }
    }

    #[inline(always)]
    pub(in crate::encoding::levels::c_port) fn as_slice(&self) -> &[OptMatch] {
        &self.entries[..self.len]
    }

    #[inline(always)]
    pub(in crate::encoding::levels::c_port) fn get(&self, index: usize) -> OptMatch {
        debug_assert!(index < self.len);
        self.entries[index]
    }
}

impl Index<usize> for OptMatchTable {
    type Output = OptMatch;

    #[inline(always)]
    fn index(&self, index: usize) -> &Self::Output {
        &self.as_slice()[index]
    }
}
