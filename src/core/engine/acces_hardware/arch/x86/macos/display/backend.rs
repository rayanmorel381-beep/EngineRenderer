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
    pub retina: bool,
    pub latency_budget_us: u64,
    pub scan_out_latency_us: u64,
    pub width_pixels: u32,
    pub height_pixels: u32,
}

pub(crate) fn detect_main_display() -> DisplayInfo {
    let mem_bytes = sysctl_u64(b"hw.memsize\0").unwrap_or(0);
    let mem_gb = mem_bytes / (1024 * 1024 * 1024);
    let (width_pixels, height_pixels) = if mem_gb >= 32 {
        (3072, 1920)
    } else {
        (2560, 1600)
    };
    DisplayInfo {
        refresh_hz: 60,
        target_render_fps: 120,
        retina: true,
        latency_budget_us: 8_333,
        scan_out_latency_us: 16_666,
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
    pub(crate) retina: bool,
    pub(crate) vsync_slots: usize,
    pub(crate) double_buffered: bool,
}

pub(crate) fn default_backend_config() -> VendorBackendConfig {
    let info = detect_main_display();
    VendorBackendConfig {
        page_size: 4096,
        refresh_rate_hz: info.refresh_hz,
        target_render_fps: info.target_render_fps,
        latency_budget_us: info.latency_budget_us,
        scan_out_latency_us: info.scan_out_latency_us,
        retina: info.retina,
        vsync_slots: 4,
        double_buffered: true,
    }
}

pub(crate) fn clamp_workers(requested: usize) -> usize {
    requested.max(1).min(2)
}
