//! Cached runtime CPU capabilities used by C-compatible generated paths.

#[cfg(target_arch = "x86_64")]
pub(crate) fn bmi2_supported() -> bool {
    // Miri cannot execute the CPUID intrinsic's inline assembly. Its purpose
    // here is to validate the portable implementation and unsafe contracts.
    if cfg!(feature = "force-scalar") || cfg!(miri) {
        return false;
    }

    use core::sync::atomic::{AtomicU8, Ordering};

    const UNKNOWN: u8 = 0;
    const ABSENT: u8 = 1;
    const PRESENT: u8 = 2;
    static BMI2: AtomicU8 = AtomicU8::new(UNKNOWN);

    match BMI2.load(Ordering::Relaxed) {
        PRESENT => true,
        ABSENT => false,
        _ => {
            // SAFETY: CPUID leaf 7 is available on every x86-64 processor
            // supported by Rust and has no memory-safety preconditions. The
            // intrinsic became safe after the crate's Rust 1.87 MSRV.
            #[allow(unused_unsafe)]
            let leaf = unsafe { core::arch::x86_64::__cpuid_count(7, 0) };
            let supported = leaf.ebx & (1 << 8) != 0;
            BMI2.store(if supported { PRESENT } else { ABSENT }, Ordering::Relaxed);
            supported
        }
    }
}

#[cfg(not(target_arch = "x86_64"))]
#[inline(always)]
pub(crate) const fn bmi2_supported() -> bool {
    false
}

#[cfg(test)]
mod tests {
    #[test]
    fn cached_bmi2_result_is_stable() {
        assert_eq!(super::bmi2_supported(), super::bmi2_supported());
    }
}
