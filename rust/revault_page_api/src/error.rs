use std::fmt;

/// Result type returned by secure-memory operations.
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone, PartialEq, Eq)]
/// Failure produced while allocating, protecting, or accessing secret memory.
pub enum Error {
    /// The operating system or secure heap could not allocate memory.
    AllocationFailed,
    /// A requested size or range overflowed the supported address space.
    CapacityOverflow,
    /// Allocation metadata or a memory canary failed validation.
    CorruptAllocation,
    /// Bytes supplied for a [`crate::SecureString`] were not valid UTF-8.
    InvalidUtf8,
    /// The operating system refused to lock secret pages in physical memory.
    LockFailed,
    /// Applying or removing page protections failed.
    ProtectionFailed,
    /// The platform cryptographic random source failed.
    RandomFailed,
    /// A mutation was attempted while a scoped read was active.
    ReadAccessActive,
    /// The target requires weakened allocation and the caller has not opted in.
    WeakAllocationDisabled,
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::AllocationFailed => write!(f, "secure allocation failed"),
            Error::CapacityOverflow => write!(f, "secure allocation capacity overflowed"),
            Error::CorruptAllocation => write!(f, "secure allocation metadata is corrupt"),
            Error::InvalidUtf8 => write!(f, "secure string is not valid utf-8"),
            Error::LockFailed => write!(f, "secure memory lock failed"),
            Error::ProtectionFailed => write!(f, "secure memory protection failed"),
            Error::RandomFailed => write!(f, "secure random source failed"),
            Error::ReadAccessActive => {
                write!(
                    f,
                    "secure memory cannot be mutated while read access is active"
                )
            }
            Error::WeakAllocationDisabled => {
                write!(f, "weakened secure memory allocation is disabled")
            }
        }
    }
}
