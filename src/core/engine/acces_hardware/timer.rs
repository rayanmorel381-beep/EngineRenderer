//! High-resolution hardware timing via monotonic clock.

use hardware::sys;

/// Returns a monotonic nanosecond timestamp from the hardware clock.
pub fn precise_timestamp_ns() -> u64 {
    sys::monotonic_ns()
}

/// Returns elapsed milliseconds between two nanosecond timestamps.
pub fn elapsed_ms(start_ns: u64, end_ns: u64) -> f64 {
    (end_ns.saturating_sub(start_ns)) as f64 / 1_000_000.0
}

/// Drop-in replacement for `std::time::Instant` backed by the hardware
/// crate's monotonic clock (clock_gettime CLOCK_MONOTONIC via syscall).
#[derive(Debug, Clone, Copy)]
pub struct HwInstant {
    ns: u64,
}

impl HwInstant {
    /// Capture the current timestamp.
    pub fn now() -> Self {
        Self {
            ns: sys::monotonic_ns(),
        }
    }

    /// Milliseconds elapsed since this instant.
    pub fn elapsed_ms(&self) -> u128 {
        let now = sys::monotonic_ns();
        now.saturating_sub(self.ns) as u128 / 1_000_000
    }

    /// Duration in milliseconds from `earlier` to `self`.
    pub fn duration_since_ms(&self, earlier: &HwInstant) -> u128 {
        self.ns.saturating_sub(earlier.ns) as u128 / 1_000_000
    }
}
