//! AMD GPU probe on Windows: matches an adapter by `AMD_VENDOR_ID` and
//! enriches it with display-class registry telemetry (clock, temp, CU count).

use super::registry::{
    HKEY_LOCAL_MACHINE, Hkey, KEY_READ, RegCloseKey, RegOpenKeyExW, reg_read_u32,
};
use super::types::GpuProbeResult;

const AMD_VENDOR_ID: u16 = 0x1002;

pub(crate) fn probe(adapters: &[GpuProbeResult]) -> Option<GpuProbeResult> {
    let base = adapters.iter().find(|a| a.vendor_id == AMD_VENDOR_ID)?;
    let mut result = GpuProbeResult {
        name: base.name.clone(),
        vendor_id: base.vendor_id,
        device_id: base.device_id,
        vram_bytes: base.vram_bytes,
        gpu_sclk_mhz: 0,
        gpu_temp: 0,
        compute_units: 0,
    };

    let sub_key: Vec<u16> =
        "SYSTEM\\CurrentControlSet\\Control\\Class\\{4d36e968-e325-11ce-bfc1-08002be10318}\\0000\0"
            .encode_utf16()
            .collect();
    let mut hkey: Hkey = core::ptr::null_mut();
    let ret =
        unsafe { RegOpenKeyExW(HKEY_LOCAL_MACHINE, sub_key.as_ptr(), 0, KEY_READ, &mut hkey) };
    if ret == 0 {
        let clock_name: Vec<u16> = "CoreClockMax\0".encode_utf16().collect();
        if let Some(mhz) = reg_read_u32(hkey, &clock_name) {
            result.gpu_sclk_mhz = mhz;
        }
        let temp_name: Vec<u16> = "GpuTemp\0".encode_utf16().collect();
        if let Some(temp) = reg_read_u32(hkey, &temp_name) {
            result.gpu_temp = temp as i32;
        }
        let cu_name: Vec<u16> = "ComputeUnits\0".encode_utf16().collect();
        if let Some(cu) = reg_read_u32(hkey, &cu_name) {
            result.compute_units = cu;
        }
        unsafe { RegCloseKey(hkey) };
    }

    Some(result)
}
