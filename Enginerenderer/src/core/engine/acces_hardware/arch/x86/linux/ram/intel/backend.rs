unsafe extern "C" {
    fn getpagesize() -> i32;
}

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct VendorBackendConfig {
    pub(crate) page_size: usize,
    pub(crate) total_bytes: u64,
    pub(crate) available_bytes: Option<u64>,
    pub(crate) frame_budget_us: u64,
    pub(crate) low_power: bool,
}

fn parse_kib_line(raw: &str) -> u64 {
    raw.split_whitespace()
        .next()
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(0)
}

pub(crate) fn default_backend_config() -> VendorBackendConfig {
    let page_size = {
        let v = unsafe { getpagesize() };
        if v > 0 { v as usize } else { 4096 }
    };

    let mut total_kib = 0_u64;
    let mut avail_kib = None;

    if let Ok(meminfo) = std::fs::read_to_string("/proc/meminfo") {
        for line in meminfo.lines() {
            if let Some(rest) = line.strip_prefix("MemTotal:") {
                total_kib = parse_kib_line(rest);
            } else if let Some(rest) = line.strip_prefix("MemAvailable:") {
                avail_kib = Some(parse_kib_line(rest));
            }
        }
    }

    VendorBackendConfig {
        page_size,
        total_bytes: total_kib.saturating_mul(1024),
        available_bytes: avail_kib.map(|v| v.saturating_mul(1024)),
        frame_budget_us: 8_333,
        low_power: false,
    }
}

pub(crate) fn clamp_workers(requested: usize) -> usize {
    requested.max(1)
}
