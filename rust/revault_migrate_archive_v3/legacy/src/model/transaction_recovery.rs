/// Durable cleanup phase for an interrupted transaction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransactionRecoveryPhase {
    /// The commit is published, but obsolete encrypted pages still need zeroing.
    Cleanup,
}

/// Decision returned by a controlled recovery progress callback.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransactionRecoveryControl {
    /// Continue with the next manifest page.
    Continue,
    /// Stop after the current durable checkpoint.
    Cancel,
}

/// Result of an explicit controlled recovery attempt.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransactionRecoveryOutcome {
    /// The archive was already clean and sealed.
    NotRequired,
    /// Cleanup and sealing completed.
    Complete,
    /// Recovery stopped at a durable checkpoint and can be resumed.
    Cancelled(TransactionRecoveryStatus),
}

/// Recovery work advertised by a published lockbox transaction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TransactionRecoveryStatus {
    /// Published transaction that owns the cleanup manifest.
    pub transaction_sequence: u64,
    /// Last transaction whose cleanup was durably completed.
    pub cleanup_sequence: u64,
    /// Current recovery phase.
    pub phase: TransactionRecoveryPhase,
    /// Number of ranges that must be zeroed.
    pub range_count: u32,
    /// Number of ranges durably completed.
    pub completed_ranges: u32,
    /// Number of manifest pages that must be processed.
    pub page_count: u32,
    /// Number of manifest pages durably completed.
    pub completed_pages: u32,
    /// Total bytes that must be zeroed.
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
    /// Total number of ranges in the recovery manifest.
    pub total_ranges: u32,
    /// Number of manifest pages durably processed.
    pub completed_pages: u32,
    /// Total manifest pages in the transaction.
    pub total_pages: u32,
    /// Number of bytes processed so far.
    pub completed_bytes: u64,
    /// Total bytes in the recovery manifest.
    pub total_bytes: u64,
}
