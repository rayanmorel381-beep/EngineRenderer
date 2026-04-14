#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct VendorBackendConfig {
    pub(crate) page_size: usize,
    pub(crate) target_render_fps: u32,
    pub(crate) latency_budget_us: u64,
    pub(crate) scan_out_latency_us: u64,
    pub(crate) vsync_slots: usize,
    pub(crate) double_buffered: bool,
    pub(crate) low_power: bool,
}

pub(crate) fn default_backend_config() -> VendorBackendConfig {
    VendorBackendConfig {
        page_size: 4096,
        target_render_fps: 120,
        latency_budget_us: 8_333,
        scan_out_latency_us: 16_666,
        vsync_slots: 4,
        double_buffered: true,
        low_power: false,
    }
}

pub(crate) fn clamp_workers(requested: usize) -> usize {
    requested.max(1).min(4)
}
