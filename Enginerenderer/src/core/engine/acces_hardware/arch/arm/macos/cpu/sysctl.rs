//! `sysctlbyname` FFI binding and a typed `u64` reader.

unsafe extern "C" {
    fn sysctlbyname(
        name: *const u8,
        oldp: *mut u8,
        oldlenp: *mut usize,
        newp: *const u8,
        newlen: usize,
    ) -> i32;
}

pub(super) fn sysctl_u64(name: &[u8]) -> Option<u64> {
    let mut out: u64 = 0;
    let mut len = core::mem::size_of::<u64>();
    let ret = unsafe {
        sysctlbyname(
            name.as_ptr(),
            &mut out as *mut u64 as *mut u8,
            &mut len,
            core::ptr::null(),
            0,
        )
    };
    if ret == 0 { Some(out) } else { None }
}
