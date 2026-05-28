//! Win32 registry helpers (UTF-16, `HKEY_LOCAL_MACHINE` queries) shared by the
//! AMD CPU detection paths.

use std::ffi::c_void;

pub(super) type Hkey = *mut c_void;
pub(super) const HKEY_LOCAL_MACHINE: Hkey = 0x80000002_usize as Hkey;
pub(super) const KEY_READ: u32 = 0x20019;

unsafe extern "system" {
    fn RegOpenKeyExW(
        key: Hkey,
        sub_key: *const u16,
        options: u32,
        desired: u32,
        result: *mut Hkey,
    ) -> i32;
    fn RegQueryValueExW(
        key: Hkey,
        value_name: *const u16,
        reserved: *mut u32,
        reg_type: *mut u32,
        data: *mut u8,
        data_len: *mut u32,
    ) -> i32;
    fn RegCloseKey(key: Hkey) -> i32;
}

pub(super) fn to_wide(s: &str) -> Vec<u16> {
    let mut out: Vec<u16> = s.encode_utf16().collect();
    out.push(0);
    out
}

pub(super) fn registry_dword(key: &str, value: &str) -> Option<u32> {
    let sub_key = to_wide(key);
    let value_name = to_wide(value);
    let mut hkey: Hkey = core::ptr::null_mut();
    let open =
        unsafe { RegOpenKeyExW(HKEY_LOCAL_MACHINE, sub_key.as_ptr(), 0, KEY_READ, &mut hkey) };
    if open != 0 {
        return None;
    }
    let mut data: u32 = 0;
    let mut len = core::mem::size_of::<u32>() as u32;
    let mut reg_type: u32 = 0;
    let query = unsafe {
        RegQueryValueExW(
            hkey,
            value_name.as_ptr(),
            core::ptr::null_mut(),
            &mut reg_type,
            &mut data as *mut u32 as *mut u8,
            &mut len,
        )
    };
    unsafe { RegCloseKey(hkey) };
    if query == 0 { Some(data) } else { None }
}

pub(super) fn registry_string(key: &str, value: &str) -> Option<String> {
    let sub_key = to_wide(key);
    let value_name = to_wide(value);
    let mut hkey: Hkey = core::ptr::null_mut();
    let open =
        unsafe { RegOpenKeyExW(HKEY_LOCAL_MACHINE, sub_key.as_ptr(), 0, KEY_READ, &mut hkey) };
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
