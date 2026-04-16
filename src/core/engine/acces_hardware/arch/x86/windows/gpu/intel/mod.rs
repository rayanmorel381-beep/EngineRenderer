pub(super) mod backend;
pub(super) mod scheduler;

use std::ffi::c_void;

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

const INTEL_VENDOR_ID: u16 = 0x8086;

type Hkey = *mut c_void;
const HKEY_LOCAL_MACHINE: Hkey = 0x80000002_usize as Hkey;
const KEY_READ: u32 = 0x20019;

unsafe extern "system" {
    fn RegOpenKeyExW(key: Hkey, sub_key: *const u16, options: u32, desired: u32, result: *mut Hkey) -> i32;
    fn RegQueryValueExW(key: Hkey, value_name: *const u16, reserved: *mut u32, reg_type: *mut u32, data: *mut u8, data_len: *mut u32) -> i32;
    fn RegCloseKey(key: Hkey) -> i32;
}

fn reg_read_u32(hkey: Hkey, value: &[u16]) -> Option<u32> {
    let mut val: u32 = 0;
    let mut len = core::mem::size_of::<u32>() as u32;
    let mut reg_type: u32 = 0;
    let ret = unsafe {
        RegQueryValueExW(hkey, value.as_ptr(), core::ptr::null_mut(), &mut reg_type, &mut val as *mut u32 as *mut u8, &mut len)
    };
    if ret == 0 { Some(val) } else { None }
}

pub(crate) fn probe(adapters: &[GpuProbeResult]) -> Option<GpuProbeResult> {
    let base = adapters.iter().find(|a| a.vendor_id == INTEL_VENDOR_ID)?;
    let mut result = GpuProbeResult {
        name: base.name.clone(),
        vendor_id: base.vendor_id,
        device_id: base.device_id,
        vram_bytes: base.vram_bytes,
        gpu_sclk_mhz: 0,
        gpu_temp: 0,
        compute_units: 0,
    };

    let sub_key: Vec<u16> = "SYSTEM\\CurrentControlSet\\Control\\Class\\{4d36e968-e325-11ce-bfc1-08002be10318}\\0000\0"
        .encode_utf16().collect();
    let mut hkey: Hkey = core::ptr::null_mut();
    let ret = unsafe { RegOpenKeyExW(HKEY_LOCAL_MACHINE, sub_key.as_ptr(), 0, KEY_READ, &mut hkey) };
    if ret == 0 {
        let clock_name: Vec<u16> = "CoreClockMax\0".encode_utf16().collect();
        if let Some(mhz) = reg_read_u32(hkey, &clock_name) {
            result.gpu_sclk_mhz = mhz;
        }
        if result.gpu_sclk_mhz == 0 {
            let alt_clock: Vec<u16> = "CoreClockMaximum\0".encode_utf16().collect();
            if let Some(mhz) = reg_read_u32(hkey, &alt_clock) {
                result.gpu_sclk_mhz = mhz;
            }
        }
        let temp_name: Vec<u16> = "GpuTemp\0".encode_utf16().collect();
        if let Some(temp) = reg_read_u32(hkey, &temp_name) {
            result.gpu_temp = temp as i32;
        }
        let eu_name: Vec<u16> = "EU_Count\0".encode_utf16().collect();
        if let Some(eu) = reg_read_u32(hkey, &eu_name) {
            result.compute_units = eu;
        }
        unsafe { RegCloseKey(hkey) };
    }

    Some(result)
}
