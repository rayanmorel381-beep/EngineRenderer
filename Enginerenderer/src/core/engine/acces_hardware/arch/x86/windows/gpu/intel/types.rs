//! `GpuProbeResult`: portable GPU probe descriptor used by the Windows
//! adapter-enumeration paths.

#[derive(Clone)]
pub(crate) struct GpuProbeResult {
    pub name: String,
    pub vendor_id: u16,
    pub device_id: u16,
    pub vram_bytes: u64,
    pub gpu_sclk_mhz: u32,
    pub gpu_temp: i32,
    pub compute_units: u32,
}
