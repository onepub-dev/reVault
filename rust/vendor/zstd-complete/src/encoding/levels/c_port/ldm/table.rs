use super::super::cctx_params::LdmParameters;
use crate::workspace::{Arena, ArenaError, ArenaSize, ReusableVec};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct LdmEntry {
    pub(crate) offset: u32,
    pub(crate) checksum: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct LdmHashTable {
    hash_table: ReusableVec<LdmEntry>,
    bucket_offsets: ReusableVec<u8>,
    bucket_size_log: u32,
}

impl LdmHashTable {
    pub(crate) fn new(params: LdmParameters) -> Self {
        debug_assert!(params.hash_log >= params.bucket_size_log);
        let table_entries = 1_usize << params.hash_log;
        let bucket_count = 1_usize << (params.hash_log - params.bucket_size_log);

        let mut hash_table = ReusableVec::with_capacity(table_entries);
        hash_table.resize(table_entries, LdmEntry::default());
        let mut bucket_offsets = ReusableVec::with_capacity(bucket_count);
        bucket_offsets.resize(bucket_count, 0);
        Self {
            hash_table,
            bucket_offsets,
            bucket_size_log: params.bucket_size_log,
        }
    }

    pub(crate) fn add_workspace_size(
        size: &mut ArenaSize,
        params: LdmParameters,
    ) -> Result<(), ArenaError> {
        size.add::<LdmEntry>(1_usize << params.hash_log)?;
        size.add::<u8>(1_usize << (params.hash_log - params.bucket_size_log))
    }

    pub(crate) fn new_in(arena: &mut Arena<'_>, params: LdmParameters) -> Result<Self, ArenaError> {
        let table_entries = 1_usize << params.hash_log;
        let bucket_count = 1_usize << (params.hash_log - params.bucket_size_log);
        let mut hash_table = arena.allocate_reusable_vec(table_entries)?;
        hash_table.resize(table_entries, LdmEntry::default());
        let mut bucket_offsets = arena.allocate_reusable_vec(bucket_count)?;
        bucket_offsets.resize(bucket_count, 0);
        Ok(Self {
            hash_table,
            bucket_offsets,
            bucket_size_log: params.bucket_size_log,
        })
    }

    pub(crate) fn insert_entry(&mut self, hash: usize, entry: LdmEntry) {
        debug_assert!(hash < self.bucket_offsets.len());
        let bucket_size = 1_usize << self.bucket_size_log;
        let offset = self.bucket_offsets[hash] as usize;
        let table_index = (hash << self.bucket_size_log) + offset;

        self.hash_table[table_index] = entry;
        self.bucket_offsets[hash] = ((offset + 1) & (bucket_size - 1)) as u8;
    }

    pub(crate) fn bucket(&self, hash: usize) -> &[LdmEntry] {
        debug_assert!(hash < self.bucket_offsets.len());
        let bucket_size = 1_usize << self.bucket_size_log;
        let bucket_start = hash << self.bucket_size_log;
        &self.hash_table[bucket_start..bucket_start + bucket_size]
    }

    pub(crate) fn bucket_offset(&self, hash: usize) -> u8 {
        self.bucket_offsets[hash]
    }

    pub(crate) fn table_len(&self) -> usize {
        self.hash_table.len()
    }

    pub(crate) fn bucket_count(&self) -> usize {
        self.bucket_offsets.len()
    }

    pub(crate) fn reset(&mut self) {
        self.hash_table.fill(LdmEntry::default());
        self.bucket_offsets.fill(0);
    }
}
