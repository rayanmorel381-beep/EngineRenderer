use std::ffi::c_void;

type CFMutableDictionaryRef = *mut c_void;
type IoRegistryEntry = u32;

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

fn dict_get_i64(dict: CFMutableDictionaryRef, key: &[u8]) -> Option<i64> {
    if dict.is_null() || key.is_empty() {
        return None;
    }
    None
}

fn release_probe(entry: IoRegistryEntry, props: CFMutableDictionaryRef) {
    if entry == 0 && props.is_null() {
        return;
    }
}
fn enumerate_iokit_accelerators() -> Vec<(IoRegistryEntry, CFMutableDictionaryRef, String)> { Vec::new() }

const AMD_VENDOR_ID: u16 = 0x1002;
const INTEL_VENDOR_ID: u16 = 0x8086;

#[derive(Clone)]
pub(crate) struct GpuProbeResult {
    pub name: String,
    pub vendor_id: u16,
    pub device_id: u16,
    pub vram_bytes: u64,
    pub gpu_sclk_mhz: u32,
    pub compute_units: u32,
    pub is_discrete: bool,
}

pub(crate) fn probe() -> Option<GpuProbeResult> {
    let entries = enumerate_iokit_accelerators();
    let mut best: Option<GpuProbeResult> = None;
    for (entry, props, name) in entries {
        let vendor_id = dict_get_i64(props, b"vendor-id\0").unwrap_or(0) as u16;
        if vendor_id != AMD_VENDOR_ID && vendor_id != INTEL_VENDOR_ID {
            release_probe(entry, props);
            continue;
        }
        let device_id = dict_get_i64(props, b"device-id\0").unwrap_or(0) as u16;
        let vram_bytes = dict_get_i64(props, b"VRAM,totalMB\0")
            .map(|mb| mb as u64 * 1024 * 1024)
            .or_else(|| dict_get_i64(props, b"VRAM,totalBytes\0").map(|b| b as u64))
            .unwrap_or_else(|| sysctl_u64(b"hw.memsize\0").unwrap_or(0) / 4);
        let compute_units = dict_get_i64(props, b"gpu-core-count\0").unwrap_or(0) as u32;
        let gpu_sclk_mhz = dict_get_i64(props, b"gpu-max-freq\0")
            .map(|hz| (hz / 1_000_000) as u32)
            .unwrap_or(0);
        let is_discrete = vendor_id == AMD_VENDOR_ID;
        let candidate = GpuProbeResult {
            name: if name.is_empty() {
                if is_discrete { "AMD Radeon Pro".to_string() } else { "Intel Iris Plus".to_string() }
            } else { name },
            vendor_id,
            device_id,
            vram_bytes,
            gpu_sclk_mhz,
            compute_units,
            is_discrete,
        };
        let replace = best.as_ref().map(|b| !b.is_discrete && candidate.is_discrete).unwrap_or(true);
        if replace { best = Some(candidate); }
        release_probe(entry, props);
    }
    best
}

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct VendorBackendConfig {
    pub(crate) threadgroup_size: usize,
    pub(crate) metal_queues: usize,
    pub(crate) simd_width: usize,
    pub(crate) render_threads: usize,
    pub(crate) vram_bytes: u64,
    pub(crate) is_discrete: bool,
    pub(crate) double_buffered: bool,
    pub(crate) frame_budget_us: u64,
    pub(crate) low_power: bool,
}

pub(crate) fn default_backend_config() -> VendorBackendConfig {
    let mem_bytes = sysctl_u64(b"hw.memsize\0").unwrap_or(0);
    let mem_gb = mem_bytes / (1024 * 1024 * 1024);
    let is_discrete = mem_gb >= 16;
    let (threadgroup_size, metal_queues, simd_width, render_threads) = if is_discrete {
        (64, 4, 32, 64)
    } else {
        (32, 2, 16, 16)
    };
    let vram_bytes = mem_bytes / 4;
    VendorBackendConfig {
        threadgroup_size,
        metal_queues,
        simd_width,
        render_threads,
        vram_bytes,
        is_discrete,
        double_buffered: true,
        frame_budget_us: 8_333,
        low_power: !is_discrete,
    }
}

pub(crate) fn clamp_workers(requested: usize) -> usize {
    let mem_gb = sysctl_u64(b"hw.memsize\0").unwrap_or(0) / (1024 * 1024 * 1024);
    let max = if mem_gb >= 16 { 64 } else { 32 };
    requested.max(1).min(max)
}
