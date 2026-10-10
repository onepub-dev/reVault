//! Admission accounting for the experimental materialized tree audit/export.
//! These are memory budgets, not wire-format limits or a promise of arbitrary
//! archive scale. Charges include vector growth slack and simultaneous typed,
//! join, coverage and source-audit representations. Payload/codec buffers remain
//! independently bounded. Caller-owned input and process RSS must be measured.
use crate::{Error, Result};

pub(crate) const TYPED_BYTES: usize = 24 * 1024 * 1024;
pub(crate) const TREE_BYTES: usize = 4 * 1024 * 1024;
pub(crate) const GRAPH_BYTES: usize = 8 * 1024 * 1024;
pub(crate) const FILE: usize = 768;
pub(crate) const FRAGMENT: usize = 512;
pub(crate) const PACK: usize = 512;
pub(crate) const PAGE: usize = 1024;
pub(crate) const OWNERSHIP: usize = 128;
pub(crate) const CLAIM: usize = 256;
pub(crate) const VARIABLE: usize = 4096;
// Form rows encode at most 1850 bytes (Forms::admit_row). This covers the
// encoded row, decoded layouts, joins and capacity slack; keys are additional.
pub(crate) const FORM: usize = 8192;

pub(crate) struct Budget {
    remaining: usize,
}
impl Budget {
    pub(crate) fn new(bytes: usize) -> Self {
        Self { remaining: bytes }
    }
    /// Charge before retaining metadata; a rejected charge leaves the budget
    /// unchanged so failures cannot wrap into a successful admission.
    pub(crate) fn take(&mut self, bytes: usize) -> Result<()> {
        self.remaining = self.remaining.checked_sub(bytes).ok_or_else(|| {
            Error::SecurityLimitExceeded("experimental tree metadata byte budget".into())
        })?;
        Ok(())
    }
    pub(crate) fn paths(&mut self, bytes: usize) -> Result<()> {
        // Source metadata, typed copy, validation scratch and allocation slack.
        self.take(bytes.checked_mul(4).ok_or(Error::CorruptRecord)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn budget_refuses_before_growth_and_does_not_wrap() {
        let mut budget = Budget::new(10);
        budget.take(9).unwrap();
        assert!(budget.take(2).is_err());
        budget.take(1).unwrap();
        assert!(budget.take(1).is_err());
        assert!(budget.paths(usize::MAX).is_err());
    }
}
