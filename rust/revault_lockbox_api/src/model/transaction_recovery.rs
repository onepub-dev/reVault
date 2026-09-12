/// Durable recovery phase for an interrupted transaction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransactionRecoveryPhase {
    /// A verified free suffix must be removed from the physical file.
    Truncate,
    /// An unpublished transaction must erase its reservations and restore the base.
    Rollback,
    /// The commit is published, but obsolete encrypted pages still need zeroing.
    Cleanup,
}

/// Decision returned by a controlled recovery progress callback.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransactionRecoveryControl {
    /// Continue with the next recovery unit (reservation range, manifest page or truncation).
    Continue,
    /// Stop after the current durable checkpoint.
    Cancel,
}

/// Result of an explicit controlled recovery attempt.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransactionRecoveryOutcome {
    /// The archive was already clean and sealed.
    NotRequired,
    /// Required rollback, cleanup or truncation completed.
    Complete,
    /// Recovery stopped at a durable checkpoint and can be resumed.
    Cancelled(TransactionRecoveryStatus),
}

/// Authenticated recovery work advertised by the selected lockbox header.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TransactionRecoveryStatus {
    /// Selected transaction sequence; identifies the base during unpublished rollback.
    pub transaction_sequence: u64,
    /// Last transaction whose cleanup was durably completed.
    pub cleanup_sequence: u64,
    /// Current recovery phase.
    pub phase: TransactionRecoveryPhase,
    /// Number of reservation or cleanup ranges; interpreted together with `phase`.
    pub range_count: u32,
    /// Number of ranges durably completed.
    pub completed_ranges: u32,
    /// Recovery units: reservation ranges for rollback, manifest pages for cleanup.
    pub page_count: u32,
    /// Number of phase-specific recovery units durably completed.
    pub completed_pages: u32,
    /// Total recovery bytes for this phase, including truncation where applicable.
    pub total_bytes: u64,
    /// Number of bytes durably completed.
    pub completed_bytes: u64,
}

/// Progress reported while explicit transaction recovery is running.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TransactionRecoveryProgress {
    /// Current recovery phase.
    pub phase: TransactionRecoveryPhase,
    /// Number of ranges durably processed so far.
    pub completed_ranges: u32,
    /// Total ranges for the current recovery phase.
    pub total_ranges: u32,
    /// Number of phase-specific recovery units durably processed.
    pub completed_pages: u32,
    /// Total recovery units for the current phase.
    pub total_pages: u32,
    /// Number of bytes processed so far.
    pub completed_bytes: u64,
    /// Total bytes for the current recovery phase.
    pub total_bytes: u64,
}
