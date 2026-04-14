unsafe extern "C" {
    fn sysctlbyname(
        name: *const u8,
        oldp: *mut u8,
        oldlenp: *mut usize,
        newp: *const u8,
        newlen: usize,
    ) -> i32;
}

fn sysctl_u64(name: &[u8]) -> Option<u64> {
    let mut out: u64 = 0;
    let mut len = core::mem::size_of::<u64>();
    let ret = unsafe {
        sysctlbyname(name.as_ptr(), &mut out as *mut u64 as *mut u8, &mut len, core::ptr::null(), 0)
    };
    if ret == 0 { Some(out) } else { None }
}

pub(crate) struct DisplayInfo {
    pub refresh_hz: u32,
    pub target_render_fps: u32,
    pub promotion_enabled: bool,
    pub retina: bool,
    pub latency_budget_us: u64,
    pub scan_out_latency_us: u64,
    pub width_pixels: u32,
    pub height_pixels: u32,
}

pub(crate) fn detect_main_display() -> DisplayInfo {
    let unified_mb = sysctl_u64(b"hw.memsize\0").unwrap_or(0) / (1024 * 1024);
    let promotion_enabled = unified_mb >= 16384;
    let refresh_hz = if promotion_enabled { 120 } else { 60 };
    let scan_out_latency_us = 1_000_000 / refresh_hz as u64;
    let (width_pixels, height_pixels) = if unified_mb >= 32768 {
        (3456, 2234)
    } else if promotion_enabled {
        (3024, 1964)
    } else {
        (2560, 1600)
    };
    DisplayInfo {
        refresh_hz,
        target_render_fps: 120,
        promotion_enabled,
        retina: true,
        latency_budget_us: 8_333,
        scan_out_latency_us,
        width_pixels,
        height_pixels,
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct VendorBackendConfig {
    pub(crate) page_size: usize,
    pub(crate) refresh_rate_hz: u32,
    pub(crate) target_render_fps: u32,
    pub(crate) latency_budget_us: u64,
    pub(crate) scan_out_latency_us: u64,
    pub(crate) promotion_enabled: bool,
    pub(crate) retina: bool,
    pub(crate) vsync_slots: usize,
    pub(crate) double_buffered: bool,
    pub(crate) width_pixels: u32,
    pub(crate) height_pixels: u32,
}

pub(crate) fn default_backend_config() -> VendorBackendConfig {
    let info = detect_main_display();
    let vsync_slots = if info.promotion_enabled { 4 } else { 3 };
    VendorBackendConfig {
        page_size: 4096,
        refresh_rate_hz: info.refresh_hz,
        target_render_fps: info.target_render_fps,
        latency_budget_us: info.latency_budget_us,
        scan_out_latency_us: info.scan_out_latency_us,
        promotion_enabled: info.promotion_enabled,
        retina: true,
        vsync_slots,
        double_buffered: true,
        width_pixels: info.width_pixels,
        height_pixels: info.height_pixels,
    }
}

pub(crate) fn clamp_workers(requested: usize) -> usize {
    requested.max(1).min(2)
}
