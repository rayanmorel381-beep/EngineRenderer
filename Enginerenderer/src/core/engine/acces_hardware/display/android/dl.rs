//! Minimal `libdl` bindings used to load `libEGL.so` / `libGLESv*.so` at
//! runtime without linking against them.

use core::ffi::{c_char, c_int, c_void};

pub(super) const RTLD_NOW: c_int = 2;
pub(super) const RTLD_GLOBAL: c_int = 0x100;

unsafe extern "C" {
    pub(super) fn dlopen(filename: *const c_char, flag: c_int) -> *mut c_void;
    pub(super) fn dlsym(handle: *mut c_void, symbol: *const c_char) -> *mut c_void;
    pub(super) fn dlerror() -> *const c_char;
}

pub(super) unsafe fn load_sym<T: Copy>(handle: *mut c_void, name: &[u8]) -> Option<T> {
    debug_assert_eq!(name.last(), Some(&0));
    let sym = unsafe { dlsym(handle, name.as_ptr() as *const c_char) };
    if sym.is_null() {
        return None;
    }
    debug_assert_eq!(
        core::mem::size_of::<T>(),
        core::mem::size_of::<*mut c_void>()
    );
    Some(unsafe { core::mem::transmute_copy::<*mut c_void, T>(&sym) })
}

pub(super) fn try_open(name: &[u8]) -> Option<*mut c_void> {
    debug_assert_eq!(name.last(), Some(&0));
    let handle = unsafe { dlopen(name.as_ptr() as *const c_char, RTLD_NOW | RTLD_GLOBAL) };
    if handle.is_null() {
        unsafe { dlerror() };
        None
    } else {
        Some(handle)
    }
}
