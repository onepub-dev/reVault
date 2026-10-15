//! Optional elapsed-time instrumentation, never a correctness deadline.

#[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
pub(super) use std::time::Instant as MetricTimer;

// std::time::Instant traps on the portable WASM target. Keep archive operations
// available there without claiming elapsed-time measurements or adding a host
// clock requirement. Native instrumentation retains its existing clock.
#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
#[derive(Clone, Copy)]
pub(super) struct MetricTimer;

#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
impl MetricTimer {
    pub(super) fn now() -> Self {
        Self
    }
    pub(super) fn elapsed(&self) -> std::time::Duration {
        std::time::Duration::ZERO
    }
}
