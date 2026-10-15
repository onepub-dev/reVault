use alloc::vec::Vec;

use crate::{
    encoding::frame_compressor::{FseTables, OffsetHistory},
    fse::fse_encoder::SharedFSETable,
    huff0::huff0_encoder::HuffmanTable,
};

pub(super) struct CandidateResult {
    pub(super) bytes: Vec<u8>,
    pub(super) final_fse_previous: FsePreviousState,
    pub(super) final_offset_history: OffsetHistory,
    pub(super) final_last_huff: Option<HuffmanTable>,
}

pub(super) struct CandidateEncodeState<'a> {
    pub(super) fse_tables: &'a mut FseTables,
    pub(super) offset_history: &'a mut OffsetHistory,
}

#[derive(Clone)]
pub(super) struct FsePreviousState {
    ll_previous: Option<SharedFSETable>,
    ml_previous: Option<SharedFSETable>,
    of_previous: Option<SharedFSETable>,
}

impl FsePreviousState {
    pub(super) fn snapshot(fse_tables: &FseTables) -> Self {
        Self {
            ll_previous: fse_tables.ll_previous.clone(),
            ml_previous: fse_tables.ml_previous.clone(),
            of_previous: fse_tables.of_previous.clone(),
        }
    }

    pub(super) fn restore(self, fse_tables: &mut FseTables) {
        fse_tables.ll_previous = self.ll_previous;
        fse_tables.ml_previous = self.ml_previous;
        fse_tables.of_previous = self.of_previous;
    }
}

pub(super) enum CandidateHuffmanState<'a> {
    Unchanged(Option<&'a HuffmanTable>),
    Updated(HuffmanTable),
}

impl CandidateHuffmanState<'_> {
    pub(super) fn as_ref(&self) -> Option<&HuffmanTable> {
        match self {
            Self::Unchanged(table) => *table,
            Self::Updated(table) => Some(table),
        }
    }

    pub(super) fn update(&mut self, table: HuffmanTable) {
        *self = Self::Updated(table);
    }

    pub(super) fn into_owned(self) -> Option<HuffmanTable> {
        match self {
            Self::Unchanged(_) => None,
            Self::Updated(table) => Some(table),
        }
    }
}
