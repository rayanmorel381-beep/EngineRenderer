pub(super) mod backend;
pub(super) mod scheduler;

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

pub(crate) struct AppleSiliconInfo {
    pub p_cores: u8,
    pub e_cores: u8,
    pub p_freq_hz: u64,
    pub e_freq_hz: u64,
    pub unified_memory_bytes: u64,
}

pub(crate) fn detect_apple_silicon() -> Option<AppleSiliconInfo> {
    if cfg!(not(target_arch = "aarch64")) {
        return None;
    }

    let p_cores = sysctl_u64(b"hw.perflevel0.physicalcpu\0").unwrap_or(0) as u8;
    let e_cores = sysctl_u64(b"hw.perflevel1.physicalcpu\0").unwrap_or(0) as u8;

    if p_cores == 0 && e_cores == 0 {
        return None;
    }

    let p_freq_hz = sysctl_u64(b"hw.perflevel0.cpuspeeds\0")
        .or_else(|| sysctl_u64(b"hw.cpufrequency_max\0"))
        .unwrap_or(0);

    let e_freq_hz = sysctl_u64(b"hw.perflevel1.cpuspeeds\0").unwrap_or(0);

    let unified_memory_bytes = sysctl_u64(b"hw.memsize\0").unwrap_or(0);

    Some(AppleSiliconInfo {
        p_cores,
        e_cores,
        p_freq_hz,
        e_freq_hz,
        unified_memory_bytes,
    })
}
