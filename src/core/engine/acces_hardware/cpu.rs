//! CPU profiling and host scheduling helpers used by the rendering backend.

use std::thread::available_parallelism;

use super::arch::compute_dispatch;
use super::arch::native_calls;

/// Runtime SIMD feature flags used to tune CPU rendering code paths.
#[derive(Debug, Clone, Copy, Default)]
pub struct SimdFeatures {
    /// AVX-512 Foundation availability.
    pub avx512f: bool,
    /// AVX2 availability.
    pub avx2: bool,
    /// AVX availability.
    pub avx: bool,
    /// FMA availability.
    pub fma: bool,
    /// SSE 4.2 availability.
    pub sse4_2: bool,
    /// SSE2 availability.
    pub sse2: bool,
    /// NEON availability.
    pub neon: bool,
}

/// Aggregated host CPU profile used by renderer setup and scheduling.
#[derive(Debug, Clone)]
pub struct CpuProfile {
    /// Logical core count selected for rendering.
    pub logical_cores: u8,
    /// L2 cache estimate in KiB.
    pub l2_cache_kb: u32,
    /// Indicates whether SMT/HT appears enabled.
    pub has_ht: bool,
    /// SIMD capabilities detected at runtime.
    pub simd_features: SimdFeatures,
    /// Preferred vector width in bits for kernels.
    pub vector_width_bits: u32,
}

impl CpuProfile {
    /// Detects a host CPU profile using dispatch hints and runtime probing.
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

    /// Returns a preferred tile width based on available SIMD width.
    pub fn optimal_tile_width(&self) -> usize {
        if self.simd_features.avx512f { 16 }
        else if self.simd_features.avx2 || self.simd_features.avx { 8 }
        else if self.simd_features.sse4_2 || self.simd_features.sse2 || self.simd_features.neon { 4 }
        else { 2 }
    }

    /// Prints a one-line CPU profile summary to stderr.
    pub fn log_summary(&self) {
        eprintln!(
            "cpu: cores={} l2={}KB ht={} vec={}bit",
            self.logical_cores, self.l2_cache_kb, self.has_ht, self.vector_width_bits,
        );
    }

    /// Builds a CPU schedule for a given number of work items.
    pub fn schedule(&self, work_items: usize) -> compute_dispatch::Schedule {
        compute_dispatch::build_cpu_schedule(work_items)
    }
}

fn detect_simd() -> SimdFeatures {
    let f = native_calls::host_detect_simd_features();
    SimdFeatures {
        avx512f: f.avx512f,
        avx2: f.avx2,
        avx: f.avx,
        fma: f.fma,
        sse4_2: f.sse4_2,
        sse2: f.sse2,
        neon: f.neon,
    }
}

fn detect_l2_cache_kb() -> u32 {
    native_calls::host_detect_l2_cache_kb()
}

fn detect_physical_cores() -> usize {
    native_calls::host_detect_physical_cores()
}

/// Snapshot of per-core clock information.
#[derive(Debug, Clone)]
pub struct CoreSnapshot {
    /// Logical core identifier.
    pub core_id: u32,
    /// Frequency in hertz.
    pub frequency_hz: u64,
}

/// Detects core frequencies when available from the host OS.
pub fn detect_core_frequencies() -> Vec<CoreSnapshot> {
    native_calls::host_detect_core_frequencies()
        .into_iter()
        .map(|f| CoreSnapshot {
            core_id: f.core_id,
            frequency_hz: f.frequency_hz,
        })
        .collect()
}

/// Returns the current thread affinity mask when supported by the host.
pub fn thread_affinity_mask() -> usize {
    native_calls::host_thread_affinity_mask()
}

/// Attempts to pin the current thread to a single core.
pub fn pin_thread_to_core(core_id: usize) {
    native_calls::host_pin_thread_to_core(core_id);
}
