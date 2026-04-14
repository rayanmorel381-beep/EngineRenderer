use std::thread::available_parallelism;

use super::arch::compute_dispatch;

#[derive(Debug, Clone, Copy, Default)]
pub struct SimdFeatures {
    pub avx512f: bool,
    pub avx2: bool,
    pub avx: bool,
    pub fma: bool,
    pub sse4_2: bool,
    pub sse2: bool,
    pub neon: bool,
}

#[derive(Debug, Clone)]
pub struct CpuProfile {
    pub logical_cores: u8,
    pub l2_cache_kb: u32,
    pub has_ht: bool,
    pub simd_features: SimdFeatures,
    pub vector_width_bits: u32,
}

impl CpuProfile {
    pub fn detect() -> Self {
        let cfg = compute_dispatch::default_cpu_config();
        let simd = detect_simd();
        let vendor_scale = match cfg.vendor {
            compute_dispatch::Vendor::Amd => 1usize,
            compute_dispatch::Vendor::Intel => 1usize,
            compute_dispatch::Vendor::Apple => 1usize,
            compute_dispatch::Vendor::Unknown => 1usize,
        };
        let power_scale = 1usize;
        let budget_scale = ((cfg.frame_budget_us / 8_333).max(1)) as usize;
        let requested = available_parallelism()
            .map(|n| n.get())
            .unwrap_or(1)
            .max(cfg.worker_hint.max(1))
            .saturating_mul(vendor_scale)
            .saturating_mul(power_scale)
            .saturating_mul(budget_scale);
        let logical_cores = compute_dispatch::clamp_cpu_workers(
            requested.min(cfg.render_workers.max(1))
        ) as u8;
        let l2_cache_kb = detect_l2_cache_kb();
        let physical = detect_physical_cores();
        let has_ht = (logical_cores as usize) > physical;
        let vector_width_bits = if simd.avx512f { 512 }
            else if simd.avx2 || simd.avx { 256 }
            else if simd.sse4_2 || simd.sse2 || simd.neon { 128 }
            else { 64 };
        Self { logical_cores, l2_cache_kb, has_ht, simd_features: simd, vector_width_bits }
    }

    pub fn optimal_tile_width(&self) -> usize {
        if self.simd_features.avx512f { 16 }
        else if self.simd_features.avx2 || self.simd_features.avx { 8 }
        else if self.simd_features.sse4_2 || self.simd_features.sse2 || self.simd_features.neon { 4 }
        else { 2 }
    }

    pub fn log_summary(&self) {
        eprintln!(
            "cpu: cores={} l2={}KB ht={} vec={}bit",
            self.logical_cores, self.l2_cache_kb, self.has_ht, self.vector_width_bits,
        );
    }

    pub fn schedule(&self, work_items: usize) -> compute_dispatch::Schedule {
        compute_dispatch::build_cpu_schedule(work_items)
    }
}

fn detect_simd() -> SimdFeatures {
    let mut f = SimdFeatures::default();
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    {
        f.avx512f = std::is_x86_feature_detected!("avx512f");
        f.avx2 = std::is_x86_feature_detected!("avx2");
        f.avx = std::is_x86_feature_detected!("avx");
        f.fma = std::is_x86_feature_detected!("fma");
        f.sse4_2 = std::is_x86_feature_detected!("sse4.2");
        f.sse2 = std::is_x86_feature_detected!("sse2");
    }
    #[cfg(target_arch = "aarch64")]
    {
        f.neon = true;
    }
    #[cfg(target_arch = "arm")]
    {
        f.neon = cfg!(target_feature = "neon");
    }
    f
}

fn detect_l2_cache_kb() -> u32 {
    #[cfg(target_os = "linux")]
    {
        if let Ok(s) = std::fs::read_to_string("/sys/devices/system/cpu/cpu0/cache/index2/size") {
            let s = s.trim();
            if let Some(stripped) = s.strip_suffix('K')
                && let Ok(n) = stripped.parse::<u32>()
            {
                return n;
            }
            if let Ok(n) = s.parse::<u32>() { return n / 1024; }
        }
    }
    #[cfg(target_os = "macos")]
    {
        if let Some(v) = sysctl_u64("hw.l2cachesize") { return (v / 1024) as u32; }
    }
    256
}

fn detect_physical_cores() -> usize {
    #[cfg(target_os = "linux")]
    {
        let mut ids = std::collections::HashSet::new();
        for cpu in 0usize..512 {
            let path = format!("/sys/devices/system/cpu/cpu{}/topology/core_id", cpu);
            match std::fs::read_to_string(&path) {
                Ok(s) => {
                    if let Ok(id) = s.trim().parse::<u32>() { ids.insert(id); } else { break; }
                }
                Err(_) => break,
            }
        }
        if !ids.is_empty() { return ids.len(); }
    }
    available_parallelism().map(|n| n.get()).unwrap_or(1)
}

#[cfg(target_os = "macos")]
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

#[derive(Debug, Clone)]
pub struct CoreSnapshot {
    pub core_id: u32,
    pub frequency_hz: u64,
}

pub fn detect_core_frequencies() -> Vec<CoreSnapshot> {
    #[cfg(target_os = "linux")]
    {
        let mut result = Vec::new();
        for cpu in 0u32..512 {
            let path = format!("/sys/devices/system/cpu/cpu{}/cpufreq/cpuinfo_max_freq", cpu);
            match std::fs::read_to_string(&path) {
                Ok(s) => {
                    let khz: u64 = s.trim().parse().unwrap_or(0);
                    result.push(CoreSnapshot { core_id: cpu, frequency_hz: khz * 1000 });
                }
                Err(_) => break,
            }
        }
        if !result.is_empty() { return result; }
    }
    #[cfg(target_os = "macos")]
    {
        if let Some(hz) = sysctl_u64("hw.cpufrequency") {
            let n = available_parallelism().map(|n| n.get() as u32).unwrap_or(1);
            return (0..n).map(|id| CoreSnapshot { core_id: id, frequency_hz: hz }).collect();
        }
    }
    let n = available_parallelism().map(|n| n.get() as u32).unwrap_or(1);
    (0..n).map(|id| CoreSnapshot { core_id: id, frequency_hz: 2_000_000_000 }).collect()
}

pub fn thread_affinity_mask() -> usize {
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

pub fn pin_thread_to_core(core_id: usize) {
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
        if core_id == usize::MAX {
            return;
        }
    }
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
