//! Common DRM userspace primitives for Intel GPUs: raw `ioctl`, sysfs helpers,
//! GEM buffer handle.

use std::fs;

pub(crate) type RawFd = i32;

pub(crate) struct GemBuffer {
    pub fd: RawFd,
    pub handle: u32,
    pub size: u64,
    pub mmap_offset: u64,
}

unsafe extern "C" {
    fn ioctl(fd: i32, request: u64, arg: *mut u8) -> i32;
}

pub(crate) unsafe fn raw_ioctl(fd: RawFd, request: u64, arg: *mut u8) -> i64 {
    unsafe { ioctl(fd, request, arg) as i64 }
}

pub(super) fn read_sysfs_u64(path: &str) -> Option<u64> {
    fs::read_to_string(path)
        .ok()
        .and_then(|v| v.trim().parse::<u64>().ok())
}

pub(super) fn read_sysfs_i32(path: &str) -> Option<i32> {
    fs::read_to_string(path)
        .ok()
        .and_then(|v| v.trim().parse::<i32>().ok())
}
