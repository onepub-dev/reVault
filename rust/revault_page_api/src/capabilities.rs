use std::sync::atomic::{AtomicBool, Ordering};

static WEAKENED_ALLOCATION_ALLOWED: AtomicBool = AtomicBool::new(false);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// Security level provided by the current secret-memory implementation.
pub enum AllocationSecurity {
    /// Secrets use locked, protected pages with platform hardening.
    Hardened,
    /// The target cannot provide the full native page protections.
    Weakened,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// Runtime description of the protections applied to secure allocations.
pub struct SecureMemoryCapabilities {
    /// Overall security level for this target.
    pub security: AllocationSecurity,
    /// Whether the operating system prevents secret pages from being swapped.
    pub memory_locked: bool,
    /// Whether secret pages are inaccessible outside scoped reads and writes.
    pub page_protected: bool,
    /// Whether inaccessible pages surround each allocation to catch overruns.
    pub guard_pages: bool,
    /// Whether secret pages are excluded from process dumps.
    pub dump_excluded: bool,
    /// Whether secret pages are excluded from inherited fork mappings.
    pub fork_excluded: bool,
}

/// Returns whether weakened secure allocations are currently permitted.
pub fn weakened_allocation_allowed() -> bool {
    WEAKENED_ALLOCATION_ALLOWED.load(Ordering::Relaxed)
}

/// Enables or disables allocations on targets without hardened page support.
///
/// The process-wide default is `false`. Enabling this is an explicit security
/// decision and should normally be limited to WebAssembly or another target
/// reported as [`AllocationSecurity::Weakened`].
pub fn set_weakened_allocation_allowed(allowed: bool) {
    WEAKENED_ALLOCATION_ALLOWED.store(allowed, Ordering::Relaxed);
}

/// Reports the secure-memory protections available on the current target.
pub fn secure_memory_capabilities() -> SecureMemoryCapabilities {
    #[cfg(any(unix, windows))]
    {
        SecureMemoryCapabilities {
            security: AllocationSecurity::Hardened,
            memory_locked: true,
            page_protected: true,
            guard_pages: true,
            dump_excluded: cfg!(target_os = "linux"),
            fork_excluded: cfg!(target_os = "linux"),
        }
    }

    #[cfg(not(any(unix, windows)))]
    {
        SecureMemoryCapabilities {
            security: AllocationSecurity::Weakened,
            memory_locked: false,
            page_protected: false,
            guard_pages: false,
            dump_excluded: false,
            fork_excluded: false,
        }
    }
}
