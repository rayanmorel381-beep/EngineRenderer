//! Win32 registry FFI primitives shared by the AMD GPU probe.

use std::ffi::c_void;

pub(super) type Hkey = *mut c_void;
pub(super) const HKEY_LOCAL_MACHINE: Hkey = 0x80000002_usize as Hkey;
pub(super) const KEY_READ: u32 = 0x20019;

unsafe extern "system" {
    pub(super) fn RegOpenKeyExW(
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
    pub(super) fn RegCloseKey(key: Hkey) -> i32;
}

pub(super) fn reg_read_u32(hkey: Hkey, value: &[u16]) -> Option<u32> {
    let mut val: u32 = 0;
    let mut len = core::mem::size_of::<u32>() as u32;
    let mut reg_type: u32 = 0;
    let ret = unsafe {
        RegQueryValueExW(
            hkey,
            value.as_ptr(),
            core::ptr::null_mut(),
            &mut reg_type,
            &mut val as *mut u32 as *mut u8,
            &mut len,
        )
    };
    if ret == 0 { Some(val) } else { None }
}
