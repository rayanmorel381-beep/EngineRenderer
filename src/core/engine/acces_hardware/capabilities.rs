//! CPU topology, memory, GPU detection — single-shot hardware probe.

use std::fs;

use hardware::sys;

/// Snapshot of the machine's capabilities, detected once at startup.
#[derive(Debug, Clone)]
pub struct HardwareCapabilities {
    /// Number of physical CPU cores.
    pub physical_cores: u8,
    /// Number of logical (hyper-threaded) cores.
    pub logical_cores: u8,
    /// Threads per physical core (1 = no HT, 2 = HT enabled).
    pub threads_per_core: u8,
    /// Total installed RAM in bytes.
    pub total_ram_bytes: u64,
    /// Available RAM in bytes at probe time.
    pub available_ram_bytes: u64,
    /// Whether a GPU was detected on the PCI bus.
    pub gpu_detected: bool,
    /// GPU vendor:device IDs (e.g. 0x1002:0x67df) if detected.
    pub gpu_id: Option<(u16, u16)>,
    /// CPU architecture.
    pub arch: &'static str,
}

fn fallback_cpu_counts() -> Option<(u8, u8, u8)> {
    let logical = std::thread::available_parallelism().ok()?.get().min(u8::MAX as usize) as u8;
    let cpuinfo = fs::read_to_string("/proc/cpuinfo").ok();
    let physical = cpuinfo
        .as_deref()
        .and_then(|text| {
            text.lines().find_map(|line| {
                line.strip_prefix("cpu cores")
                    .and_then(|value| value.split(':').nth(1))
                    .and_then(|value| value.trim().parse::<u8>().ok())
            })
        })
        .unwrap_or(logical.max(1));
    let threads_per_core = (logical / physical.max(1)).max(1);
    Some((physical.max(1), logical.max(physical), threads_per_core))
}

fn fallback_memory_info() -> Option<(u64, u64)> {
    let meminfo = fs::read_to_string("/proc/meminfo").ok()?;
    let mut total_kib = None;
    let mut available_kib = None;

    for line in meminfo.lines() {
        if let Some(rest) = line.strip_prefix("MemTotal:") {
            total_kib = rest.split_whitespace().next().and_then(|v| v.parse::<u64>().ok());
        } else if let Some(rest) = line.strip_prefix("MemAvailable:") {
            available_kib = rest.split_whitespace().next().and_then(|v| v.parse::<u64>().ok());
        } else if available_kib.is_none()
            && let Some(rest) = line.strip_prefix("MemFree:") {
            available_kib = rest.split_whitespace().next().and_then(|v| v.parse::<u64>().ok());
        }
    }

    Some((total_kib? * 1024, available_kib.unwrap_or(0) * 1024))
}

impl HardwareCapabilities {
    /// Configures the Linux syscall / OS / DRM tables required by the
    /// `hardware` crate.  Must be called once before any syscall-based
    /// probe.  The crate ships with NO hardcoded values — every constant
    /// is injected here.
    fn configure_linux_tables() {
        let arch = sys::detect_arch();
        match arch {
            sys::Architecture::X86_64 => {
                sys::set_syscall_nrs(&sys::SyscallNrTable {
                    read: 0,
                    write: 1,
                    openat: 257,
                    close: 3,
                    mmap: 9,
                    munmap: 11,
                    ioctl: 16,
                    sched_yield: 24,
                    nanosleep: 35,
                    clone: 56,
                    exit: 60,
                    wait4: 61,
                    kill: 62,
                    fsync: 74,
                    unlinkat: 263,
                    getdents64: 217,
                    clock_gettime: 228,
                    sched_setaffinity: 203,
                    sched_getaffinity: 204,
                    stat: 4,
                    socket: 41,
                    connect: 42,
                    accept: 43,
                    bind: 49,
                    listen: 50,
                    execve: 59,
                    fcntl: 72,
                    getcwd: 79,
                    rt_sigaction: 13,
                    iopl: 172,
                    mkdirat: 258,
                    sysinfo: 99,
                });
            }
            sys::Architecture::AArch64 => {
                sys::set_syscall_nrs(&sys::SyscallNrTable {
                    read: 63,
                    write: 64,
                    openat: 56,
                    close: 57,
                    mmap: 222,
                    munmap: 215,
                    ioctl: 29,
                    sched_yield: 124,
                    nanosleep: 101,
                    clone: 220,
                    exit: 93,
                    wait4: 260,
                    kill: 129,
                    fsync: 82,
                    unlinkat: 35,
                    getdents64: 61,
                    clock_gettime: 113,
                    sched_setaffinity: 122,
                    sched_getaffinity: 123,
                    stat: 79,
                    socket: 198,
                    connect: 203,
                    accept: 202,
                    bind: 200,
                    listen: 201,
                    execve: 221,
                    fcntl: 25,
                    getcwd: 17,
                    rt_sigaction: 134,
                    iopl: -1,
                    mkdirat: 34,
                    sysinfo: 179,
                });
            }
            _ => {}
        }
        sys::set_os_constants(&sys::OsConstants {
            at_fdcwd: -100,
            sigchld: 17,
            map_private_anon: 0x22,
            map_shared_anon: 0x21,
            map_shared: 0x01,
            prot_read_write: 0x3,
            clock_monotonic: 1,
            o_creat: 0o100,
            o_trunc: 0o1000,
            o_nonblock: 0o4000,
            o_excl: 0o200,
            o_directory: 0o200000,
        });
        sys::gpu::set_drm_constants(sys::gpu::DrmConstants {
            ioctl_version: 0xC040_6400,
            ioctl_gem_close: 0x4008_6409,
            ioctl_radeon_info: 0xC010_6467,
            ioctl_radeon_gem_info: 0xC018_645C,
            ioctl_radeon_gem_create: 0xC020_645D,
            ioctl_radeon_gem_mmap: 0xC020_645E,
            ioctl_radeon_gem_wait_idle: 0x4008_6464,
            ioctl_radeon_cs: 0xC020_6466,
            radeon_info_device_id: 0x00,
            radeon_info_num_gb_pipes: 0x01,
            radeon_info_vram_usage: 0x1E,
            radeon_info_active_cu_count: 0x20,
            radeon_info_current_gpu_sclk: 0x22,
            radeon_info_current_gpu_mclk: 0x23,
            radeon_info_current_gpu_temp: 0x21,
            radeon_info_max_se: 0x12,
            radeon_info_max_sh_per_se: 0x13,
            radeon_gem_domain_vram: 0x4,
            radeon_gem_domain_gtt: 0x2,
            radeon_chunk_id_relocs: 0x01,
            radeon_chunk_id_ib: 0x02,
            radeon_chunk_id_flags: 0x03,
            radeon_cs_ring_gfx: 0,
            radeon_cs_use_vm: 0x02,
        });

        // ── DRM device open callback ────────────────────────────────
        // The hardware crate has no hardcoded paths. We supply a
        // callback that opens /dev/dri/card0 via the crate's own
        // sys_open (which uses the syscall numbers we just injected).
        sys::gpu::drm::set_open_drm_fn(|| {
            sys::sys_open(b"/dev/dri/card0\0", 2, 0) // O_RDWR = 2
        });
    }

    /// Detects hardware via the `hardware` crate's syscall-based probes.
    ///
    /// Sequence: `init_shims()` registers arch-specific shim function
    /// pointers, then `configure_linux_tables()` injects syscall numbers,
    /// OS constants, and DRM ioctl values.  We do NOT call `sys::init()`
    /// — that runs 17 bare-metal boot phases (firmware, interrupts, bus
    /// scan, accelerator lifecycle…) which are not safe/relevant in
    /// userspace.
    pub fn detect() -> Self {
        sys::init_shims();
        Self::configure_linux_tables();
        sys::request_hw_privilege();

        let arch = sys::detect_arch();
        let arch_str = match arch {
            sys::Architecture::X86_64 => "x86_64",
            sys::Architecture::AArch64 => "aarch64",
            _ => "unknown",
        };

        // ── CPU topology ────────────────────────────────────────────
        let topo = sys::cpu::topology::detect();
        let (physical_cores, logical_cores, threads_per_core) =
            if topo.physical_cores == 0 || topo.logical_cores == 0 {
                fallback_cpu_counts().unwrap_or((1, 1, 1))
            } else {
                (
                    topo.physical_cores.max(1),
                    topo.logical_cores.max(topo.physical_cores.max(1)),
                    topo.threads_per_core.max(1),
                )
            };

        // ── Memory ──────────────────────────────────────────────────
        let (total_ram, available_ram) = match sys::detect_memory_info() {
            Some(info) if info.total_bytes > 0 => (info.total_bytes, info.available_bytes),
            _ => fallback_memory_info().unwrap_or((0, 0)),
        };

        // ── GPU detection ───────────────────────────────────────────
        let mut gpus = [sys::gpu::GpuDevice {
            bus: 0,
            device: 0,
            function: 0,
            vendor_id: 0,
            device_id: 0,
            class: 0,
            subclass: 0,
            prog_if: 0,
            bar0: 0,
        }; 8];
        let gpu_count = sys::gpu::detect_gpus(&mut gpus);
        let (gpu_detected, gpu_id) = if gpu_count > 0 {
            (true, Some((gpus[0].vendor_id, gpus[0].device_id)))
        } else if sys::gpu::drm::open().is_some() {
            (true, None)
        } else {
            (false, None)
        };

        Self {
            physical_cores,
            logical_cores,
            threads_per_core,
            total_ram_bytes: total_ram,
            available_ram_bytes: available_ram,
            gpu_detected,
            gpu_id,
            arch: arch_str,
        }
    }

    /// Returns the optimal number of render worker threads.
    ///
    /// Prefer all logical threads when SMT/HT is available; this renderer
    /// is latency-heavy on BVH traversal and benefits from keeping the core
    /// pipelines occupied while other rays stall on memory.
    pub fn optimal_render_threads(&self) -> usize {
        let physical = self.physical_cores as usize;
        let logical = self.logical_cores as usize;
        if self.threads_per_core > 1 && logical > physical {
            logical.max(1)
        } else {
            physical.max(1)
        }
    }

    /// Maximum framebuffer bytes we should allocate, keeping a safety
    /// margin of 512 MB for the OS and other allocations.
    pub fn max_framebuffer_bytes(&self) -> u64 {
        let margin = 512 * 1024 * 1024;
        self.available_ram_bytes.saturating_sub(margin)
    }

    /// Log a summary to stderr for diagnostics.
    pub fn log_summary(&self) {
        eprintln!(
            "hardware: arch={} cores={}/{} ({}T/core) ram={:.1}GB avail={:.1}GB gpu={}",
            self.arch,
            self.physical_cores,
            self.logical_cores,
            self.threads_per_core,
            self.total_ram_bytes as f64 / (1024.0 * 1024.0 * 1024.0),
            self.available_ram_bytes as f64 / (1024.0 * 1024.0 * 1024.0),
            if self.gpu_detected {
                match self.gpu_id {
                    Some((v, d)) if d != 0 => format!("yes ({v:04x}:{d:04x})"),
                    Some((v, _)) => format!("yes ({v:04x}:????)"),
                    None => "yes".to_string(),
                }
            } else {
                "none".to_string()
            },
        );
    }
}
