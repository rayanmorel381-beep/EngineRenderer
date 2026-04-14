pub(super) mod backend;
pub(super) mod scheduler;

use std::ffi::c_void;

type HKEY = *mut c_void;
const HKEY_LOCAL_MACHINE: HKEY = 0x80000002_usize as HKEY;
const KEY_READ: u32 = 0x20019;

#[repr(C)]
struct SystemInfo {
    processor_architecture: u16,
    _reserved: u16,
    _page_size: u32,
    _minimum_application_address: *mut c_void,
    _maximum_application_address: *mut c_void,
    _active_processor_mask: usize,
    number_of_processors: u32,
    _processor_type: u32,
    _allocation_granularity: u32,
    _processor_level: u16,
    _processor_revision: u16,
}

unsafe extern "system" {
    fn RegOpenKeyExW(key: HKEY, sub_key: *const u16, options: u32, desired: u32, result: *mut HKEY) -> i32;
    fn RegQueryValueExW(key: HKEY, value_name: *const u16, reserved: *mut u32, reg_type: *mut u32, data: *mut u8, data_len: *mut u32) -> i32;
    fn RegCloseKey(key: HKEY) -> i32;
    fn GetSystemInfo(lp_system_info: *mut SystemInfo);
}

fn to_wide(s: &str) -> Vec<u16> {
    let mut out: Vec<u16> = s.encode_utf16().collect();
    out.push(0);
    out
}

fn registry_string(key: &str, value: &str) -> Option<String> {
    let sub_key = to_wide(key);
    let value_name = to_wide(value);
    let mut hkey: HKEY = core::ptr::null_mut();
    let open = unsafe { RegOpenKeyExW(HKEY_LOCAL_MACHINE, sub_key.as_ptr(), 0, KEY_READ, &mut hkey) };
    if open != 0 {
        return None;
    }
    let mut len: u32 = 0;
    let mut reg_type: u32 = 0;
    let first = unsafe {
        RegQueryValueExW(
            hkey,
            value_name.as_ptr(),
            core::ptr::null_mut(),
            &mut reg_type,
            core::ptr::null_mut(),
            &mut len,
        )
    };
    if first != 0 || len == 0 {
        unsafe { RegCloseKey(hkey) };
        return None;
    }
    let mut buf = vec![0u8; len as usize];
    let second = unsafe {
        RegQueryValueExW(
            hkey,
            value_name.as_ptr(),
            core::ptr::null_mut(),
            &mut reg_type,
            buf.as_mut_ptr(),
            &mut len,
        )
    };
    unsafe { RegCloseKey(hkey) };
    if second != 0 {
        return None;
    }
    let u16_len = (len as usize) / 2;
    let wide = unsafe { core::slice::from_raw_parts(buf.as_ptr() as *const u16, u16_len) };
    let used = wide.iter().position(|&c| c == 0).unwrap_or(wide.len());
    Some(String::from_utf16_lossy(&wide[..used]))
}

fn get_system_info() -> SystemInfo {
    let mut info = SystemInfo {
        processor_architecture: 0,
        _reserved: 0,
        _page_size: 0,
        _minimum_application_address: core::ptr::null_mut(),
        _maximum_application_address: core::ptr::null_mut(),
        _active_processor_mask: 0,
        number_of_processors: 0,
        _processor_type: 0,
        _allocation_granularity: 0,
        _processor_level: 0,
        _processor_revision: 0,
    };
    unsafe { GetSystemInfo(&mut info) };
    info
}

const CPU_KEY: &str = r"HARDWARE\DESCRIPTION\System\CentralProcessor\0";
const PROCESSOR_ARCHITECTURE_ARM64: u16 = 12;

pub(crate) struct ArmWinInfo {
    pub brand: String,
    pub core_count: u32,
}

pub(crate) fn detect_arm() -> Option<ArmWinInfo> {
    let info = get_system_info();
    if info.processor_architecture != PROCESSOR_ARCHITECTURE_ARM64 {
        return None;
    }

    let brand = registry_string(CPU_KEY, "ProcessorNameString").unwrap_or_default();

    Some(ArmWinInfo {
        brand,
        core_count: info.number_of_processors,
    })
}
