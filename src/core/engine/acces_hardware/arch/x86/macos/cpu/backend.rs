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

fn sysctl_string(name: &[u8]) -> Option<String> {
    let mut buf = [0u8; 256];
    let mut len = buf.len();
    let ret = unsafe {
        sysctlbyname(name.as_ptr(), buf.as_mut_ptr(), &mut len, core::ptr::null(), 0)
    };
    if ret != 0 { return None; }
    let s = &buf[..len.saturating_sub(1)];
    core::str::from_utf8(s).ok().map(|v| v.trim_end_matches('\0').to_string())
}

pub(crate) struct IntelMacCpuInfo {
    pub physical_cores: u8,
    pub logical_cores: u8,
    pub freq_max_hz: u64,
    pub freq_base_hz: u64,
    pub cache_l3_bytes: u64,
    pub packages: u8,
    pub brand_string: Option<String>,
    pub hyperthreading: bool,
}

pub(crate) fn detect() -> Option<IntelMacCpuInfo> {
    let physical_cores = sysctl_u64(b"hw.physicalcpu\0")? as u8;
    let logical_cores = sysctl_u64(b"hw.logicalcpu\0")
        .unwrap_or(physical_cores as u64) as u8;
    let freq_max_hz = sysctl_u64(b"hw.cpufrequency_max\0").unwrap_or(0);
    let freq_base_hz = sysctl_u64(b"hw.cpufrequency\0").unwrap_or(freq_max_hz);
    let cache_l3_bytes = sysctl_u64(b"hw.l3cachesize\0").unwrap_or(0);
    let packages = sysctl_u64(b"hw.packages\0").unwrap_or(1) as u8;
    let brand_string = sysctl_string(b"machdep.cpu.brand_string\0");
    let hyperthreading = logical_cores > physical_cores;
    Some(IntelMacCpuInfo {
        physical_cores,
        logical_cores,
        freq_max_hz,
        freq_base_hz,
        cache_l3_bytes,
        packages,
        brand_string,
        hyperthreading,
    })
}

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct VendorBackendConfig {
    pub(crate) physical_workers: usize,
    pub(crate) logical_workers: usize,
    pub(crate) render_workers: usize,
    pub(crate) hyperthreading: bool,
    pub(crate) freq_max_hz: u64,
    pub(crate) freq_base_hz: u64,
    pub(crate) cache_l3_bytes: u64,
    pub(crate) frame_budget_us: u64,
    pub(crate) low_power: bool,
}

pub(crate) fn default_backend_config() -> VendorBackendConfig {
    let physical = sysctl_u64(b"hw.physicalcpu\0").unwrap_or(1).max(1) as usize;
    let logical = sysctl_u64(b"hw.logicalcpu\0").unwrap_or(physical as u64).max(1) as usize;
    let freq_max_hz = sysctl_u64(b"hw.cpufrequency_max\0").unwrap_or(0);
    let freq_base_hz = sysctl_u64(b"hw.cpufrequency\0").unwrap_or(freq_max_hz);
    let cache_l3_bytes = sysctl_u64(b"hw.l3cachesize\0").unwrap_or(0);
    let render_workers = logical.saturating_sub(1).max(1);
    VendorBackendConfig {
        physical_workers: physical,
        logical_workers: logical,
        render_workers,
        hyperthreading: logical > physical,
        freq_max_hz,
        freq_base_hz,
        cache_l3_bytes,
        frame_budget_us: 8_333,
        low_power: physical <= 2,
    }
}

pub(crate) fn clamp_workers(requested: usize) -> usize {
    let logical = sysctl_u64(b"hw.logicalcpu\0").unwrap_or(1).max(1) as usize;
    requested.max(1).min(logical)
}
