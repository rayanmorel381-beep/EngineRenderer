
use crate::core::engine::acces_hardware::cpu::CpuProfile;
use crate::core::engine::acces_hardware::gpu::{gpu_dispatch_tiles, GpuRenderBackend};

#[derive(Debug, Clone, Copy, Default)]
pub struct HostSimdFeatures {
    pub avx512f: bool,
    pub avx2: bool,
    pub avx: bool,
    pub fma: bool,
    pub sse4_2: bool,
    pub sse2: bool,
    pub neon: bool,
    pub sve: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct HostCoreFrequency {
    pub core_id: u32,
    pub frequency_hz: u64,
}

#[derive(Debug, Clone, Copy)]
pub struct NativeCpuCall {
    pub architecture: &'static str,
    pub logical_cores: u8,
    pub vector_width_bits: u32,
}

pub fn host_architecture_name() -> &'static str {
    if cfg!(target_arch = "x86_64") {
        "x86_64"
    } else if cfg!(target_arch = "aarch64") {
        "aarch64"
    } else if cfg!(target_arch = "arm") {
        "arm"
    } else if cfg!(target_arch = "x86") {
        "x86"
    } else {
        "other"
    }
}

pub fn host_detect_simd_features() -> HostSimdFeatures {
    let mut features = HostSimdFeatures::default();
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    {
        features.avx512f = std::is_x86_feature_detected!("avx512f");
        features.avx2 = std::is_x86_feature_detected!("avx2");
        features.avx = std::is_x86_feature_detected!("avx");
        features.fma = std::is_x86_feature_detected!("fma");
        features.sse4_2 = std::is_x86_feature_detected!("sse4.2");
        features.sse2 = std::is_x86_feature_detected!("sse2");
    }
    #[cfg(target_arch = "aarch64")]
    {
        features.neon = cfg!(target_feature = "neon");
        features.sve = cfg!(target_feature = "sve");
    }
    #[cfg(target_arch = "arm")]
    {
        features.neon = cfg!(target_feature = "neon");
    }
    features
}

pub fn host_has_simd() -> bool {
    let f = host_detect_simd_features();
    f.avx512f || f.avx2 || f.avx || f.fma || f.sse4_2 || f.sse2 || f.neon || f.sve
}

pub fn host_detect_l2_cache_kb() -> u32 {
    #[cfg(target_os = "linux")]
    {
        if let Ok(s) = std::fs::read_to_string("/sys/devices/system/cpu/cpu0/cache/index2/size") {
            let s = s.trim();
            if let Some(stripped) = s.strip_suffix('K')
                && let Ok(n) = stripped.parse::<u32>()
            {
                return n;
            }
            if let Ok(n) = s.parse::<u32>() {
                return n / 1024;
            }
        }
    }
    #[cfg(target_os = "macos")]
    {
        if let Some(v) = sysctl_u64("hw.l2cachesize") {
            return (v / 1024) as u32;
        }
    }
    256
}

pub fn host_detect_physical_cores() -> usize {
    #[cfg(target_os = "linux")]
    {
        let mut ids = std::collections::HashSet::new();
        for cpu in 0usize..512 {
            let path = format!("/sys/devices/system/cpu/cpu{}/topology/core_id", cpu);
            match std::fs::read_to_string(&path) {
                Ok(s) => {
                    if let Ok(id) = s.trim().parse::<u32>() {
                        ids.insert(id);
                    } else {
                        break;
                    }
                }
                Err(_) => break,
            }
        }
        if !ids.is_empty() {
            return ids.len();
        }
    }
    std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1)
}

pub fn host_detect_core_frequencies() -> Vec<HostCoreFrequency> {
    #[cfg(target_os = "linux")]
    {
        let mut result = Vec::new();
        for cpu in 0u32..512 {
            let path = format!("/sys/devices/system/cpu/cpu{}/cpufreq/cpuinfo_max_freq", cpu);
            match std::fs::read_to_string(&path) {
                Ok(s) => {
                    let khz: u64 = s.trim().parse().unwrap_or(0);
                    result.push(HostCoreFrequency {
                        core_id: cpu,
                        frequency_hz: khz * 1000,
                    });
                }
                Err(_) => break,
            }
        }
        if !result.is_empty() {
            return result;
        }
    }
    #[cfg(target_os = "macos")]
    {
        if let Some(hz) = sysctl_u64("hw.cpufrequency") {
            let n = std::thread::available_parallelism()
                .map(|n| n.get() as u32)
                .unwrap_or(1);
            return (0..n)
                .map(|id| HostCoreFrequency {
                    core_id: id,
                    frequency_hz: hz,
                })
                .collect();
        }
    }
    let n = std::thread::available_parallelism()
        .map(|n| n.get() as u32)
        .unwrap_or(1);
    (0..n)
        .map(|id| HostCoreFrequency {
            core_id: id,
            frequency_hz: 2_000_000_000,
        })
        .collect()
}

pub fn host_thread_affinity_mask() -> usize {
    #[cfg(all(target_os = "linux", any(target_arch = "x86_64", target_arch = "aarch64")))]
    {
        let mut set = [0u8; 128];
        let ret = raw_sched_getaffinity(0, 128, set.as_mut_ptr());
        if ret == 0 {
            let mut mask: usize = 0;
            for (i, byte) in set.iter().enumerate().take(core::mem::size_of::<usize>()) {
                mask |= (*byte as usize) << (i * 8);
            }
            return mask;
        }
    }
    usize::MAX
}

pub fn host_pin_thread_to_core(core_id: usize) {
    #[cfg(all(target_os = "linux", any(target_arch = "x86_64", target_arch = "aarch64")))]
    {
        let mut set = [0u8; 128];
        let byte = core_id / 8;
        let bit = core_id % 8;
        if byte < 128 {
            set[byte] = 1 << bit;
            raw_sched_setaffinity(0, 128, set.as_ptr());
        }
    }
    #[cfg(not(all(target_os = "linux", any(target_arch = "x86_64", target_arch = "aarch64"))))]
    {
        if core_id == usize::MAX {}
    }
}

pub fn native_cpu_call(cpu: &CpuProfile) -> NativeCpuCall {
    NativeCpuCall {
        architecture: host_architecture_name(),
        logical_cores: cpu.logical_cores,
        vector_width_bits: cpu.vector_width_bits,
    }
}

#[derive(Debug, Clone, Copy)]
pub struct NativeGpuCall {
    pub init_ok: bool,
    pub dispatch_ok: bool,
    pub framebuffer_ok: bool,
}

pub fn native_gpu_call(gpu: Option<&GpuRenderBackend>, workgroup_size: usize) -> NativeGpuCall {
    let dispatched = gpu_dispatch_tiles(1, workgroup_size.max(1) as u32);
    let init_ok = gpu.is_some();
    let framebuffer_ok = gpu.map(|g| g.has_active_framebuffer()).unwrap_or(false);

    NativeGpuCall {
        init_ok,
        dispatch_ok: dispatched > 0,
        framebuffer_ok,
    }
}

#[cfg(target_os = "macos")]
#[allow(clashing_extern_declarations)]
unsafe extern "C" {
    fn sysctlbyname(
        name: *const core::ffi::c_char,
        oldp: *mut core::ffi::c_void,
        oldlenp: *mut usize,
        newp: *const core::ffi::c_void,
        newlen: usize,
    ) -> i32;
}

#[cfg(target_os = "macos")]
fn sysctl_u64(name: &str) -> Option<u64> {
    use std::ffi::CString;
    let cname = CString::new(name).ok()?;
    let mut val: u64 = 0;
    let mut sz = core::mem::size_of::<u64>();
    let ret = unsafe {
        sysctlbyname(
            cname.as_ptr(),
            &mut val as *mut u64 as *mut core::ffi::c_void,
            &mut sz,
            core::ptr::null(),
            0,
        )
    };
    if ret == 0 { Some(val) } else { None }
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
fn raw_sched_setaffinity(pid: i32, cpusetsize: usize, mask: *const u8) -> i32 {
    let ret: i64;
    unsafe {
        core::arch::asm!(
            "syscall",
            inlateout("rax") 203_i64 => ret,
            in("rdi") pid as i64,
            in("rsi") cpusetsize,
            in("rdx") mask as u64,
            out("rcx") _,
            out("r11") _,
            options(nostack),
        );
    }
    ret as i32
}

#[cfg(all(target_os = "linux", target_arch = "aarch64"))]
fn raw_sched_setaffinity(pid: i32, cpusetsize: usize, mask: *const u8) -> i32 {
    let ret: i64;
    unsafe {
        core::arch::asm!(
            "svc #0",
            inlateout("x0") pid as i64 => ret,
            in("x1") cpusetsize as u64,
            in("x2") mask as u64,
            in("x8") 122_u64,
            options(nostack),
        );
    }
    ret as i32
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
fn raw_sched_getaffinity(pid: i32, cpusetsize: usize, mask: *mut u8) -> i32 {
    let ret: i64;
    unsafe {
        core::arch::asm!(
            "syscall",
            inlateout("rax") 204_i64 => ret,
            in("rdi") pid as i64,
            in("rsi") cpusetsize,
            in("rdx") mask as u64,
            out("rcx") _,
            out("r11") _,
            options(nostack),
        );
    }
    ret as i32
}

#[cfg(all(target_os = "linux", target_arch = "aarch64"))]
fn raw_sched_getaffinity(pid: i32, cpusetsize: usize, mask: *mut u8) -> i32 {
    let ret: i64;
    unsafe {
        core::arch::asm!(
            "svc #0",
            inlateout("x0") pid as i64 => ret,
            in("x1") cpusetsize as u64,
            in("x2") mask as u64,
            in("x8") 123_u64,
            options(nostack),
        );
    }
    ret as i32
}
