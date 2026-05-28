use std::sync::OnceLock;
use std::time::Instant;

fn base_instant() -> &'static Instant {
    static BASE: OnceLock<Instant> = OnceLock::new();
    BASE.get_or_init(Instant::now)
}

pub fn precise_timestamp_ns() -> u64 {
    base_instant().elapsed().as_nanos() as u64
}

pub fn elapsed_ms(start_ns: u64, end_ns: u64) -> f64 {
    (end_ns.saturating_sub(start_ns)) as f64 / 1_000_000.0
}

#[derive(Debug, Clone, Copy)]
pub struct HwInstant {
    ns: u64,
}

impl HwInstant {
    pub fn now() -> Self {
        Self { ns: precise_timestamp_ns() }
    }

    pub fn elapsed_ms(&self) -> u128 {
        let now = precise_timestamp_ns();
        now.saturating_sub(self.ns) as u128 / 1_000_000
    }

    pub fn duration_since_ms(&self, earlier: &HwInstant) -> u128 {
        self.ns.saturating_sub(earlier.ns) as u128 / 1_000_000
    }
}
