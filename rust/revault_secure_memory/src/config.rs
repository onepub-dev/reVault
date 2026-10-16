use std::sync::atomic::{AtomicUsize, Ordering};

use crate::{
    error::{Error, Result},
    memory_region,
};

#[cfg(not(target_os = "android"))]
pub(crate) const SIZE_CLASSES: &[usize] = &[64, 128, 256, 512, 1024, 2048, 4096];
// Each live class requires at least one locked OS page. On 16 KiB Android
// devices the usual 64 KiB allowance cannot back seven classes at once. Share
// the common key/password sizes while retaining the same canaries and guards.
#[cfg(target_os = "android")]
pub(crate) const SIZE_CLASSES: &[usize] = &[256, 1024, 4096];

#[cfg(not(target_os = "android"))]
const DEFAULT_ALLOCATION_CHUNK_BYTES: usize = 64 * 1024;
// Android app processes have a small locked-memory allowance. Reserve one
// actual OS page per size class rather than spending the allowance on the
// first 64 KiB arena. Zero selects the runtime page size (including 16 KiB).
#[cfg(target_os = "android")]
const DEFAULT_ALLOCATION_CHUNK_BYTES: usize = 0;

static ALLOCATION_CHUNK_BYTES: AtomicUsize = AtomicUsize::new(DEFAULT_ALLOCATION_CHUNK_BYTES);

/// Returns the process-wide backing allocation chunk size in bytes.
///
/// The value is always aligned to the platform page size. The default is one
/// page on Android, to conserve its small locked-memory allowance, and 64 KiB
/// on other platforms. Hardened allocations still require successful locking.
pub fn allocation_chunk_bytes() -> usize {
    match ALLOCATION_CHUNK_BYTES.load(Ordering::Relaxed) {
        0 => memory_region::page_size(),
        bytes => bytes,
    }
}

/// Sets the process-wide backing allocation chunk size.
///
/// Values are rounded up to a platform page boundary. Returns
/// [`Error::AllocationFailed`] when `bytes` is smaller than one page and
/// [`Error::CapacityOverflow`] when alignment would overflow.
pub fn set_allocation_chunk_bytes(bytes: usize) -> Result<()> {
    let page_size = memory_region::page_size();
    if bytes < page_size {
        return Err(Error::AllocationFailed);
    }
    let rounded = bytes
        .checked_add(page_size - 1)
        .map(|value| value / page_size * page_size)
        .ok_or(Error::CapacityOverflow)?;
    ALLOCATION_CHUNK_BYTES.store(rounded, Ordering::Relaxed);
    Ok(())
}
