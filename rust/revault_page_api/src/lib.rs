#![deny(unsafe_op_in_unsafe_fn)]
#![deny(clippy::undocumented_unsafe_blocks)]
#![deny(missing_docs)]
//! Hardened memory primitives for secrets used by reVault.
//!
//! This crate provides zeroizing byte and UTF-8 containers backed by guarded,
//! page-protected memory. Secret contents are available only inside scoped
//! callbacks; when the callback returns, the underlying pages are protected
//! again. Mutating a secure value while a read scope is active fails closed.
//!
//! Native Unix and Windows targets use locked pages and guard pages. Targets
//! without those facilities, including WebAssembly, require an explicit call to
//! [`set_weakened_allocation_allowed`] before allocating secret memory.
//!
//! # Example
//!
//! ```
//! use revault_page_api::SecureString;
//!
//! let secret = SecureString::try_from_slice(b"correct horse")?;
//! let length = secret.with_str(str::len)?;
//! assert_eq!(length, 13);
//! # Ok::<(), revault_page_api::Error>(())
//! ```
//!
//! See the [reVault repository README](https://github.com/onepub-dev/reVault#readme)
//! for the project overview, security model, and related crates.

mod allocation;
mod arena;
mod canary;
mod capabilities;
mod config;
mod error;
mod memory_region;
mod page_permission;
mod secure_access;
mod secure_heap;
mod secure_string;
mod secure_vec;

#[cfg(test)]
mod tests;

pub use capabilities::{
    secure_memory_capabilities, set_weakened_allocation_allowed, weakened_allocation_allowed,
    AllocationSecurity, SecureMemoryCapabilities,
};
pub use config::{allocation_chunk_bytes, set_allocation_chunk_bytes};
pub use error::{Error, Result};
pub use secure_access::{read_access, SecureReadAccess};
pub use secure_string::SecureString;
pub use secure_vec::SecureVec;
