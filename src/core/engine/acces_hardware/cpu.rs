//! CPU topology, features, affinity, and vector capabilities.

use std::fs;

use hardware::sys;

/// Detailed CPU information for render scheduling decisions.
#[derive(Debug, Clone)]
pub struct CpuProfile {
    /// Vendor name (Intel, AMD, ARM, …).
    pub vendor: &'static str,
    /// Model name string.
    pub model_name: String,
    /// Base frequency in MHz.
    pub frequency_mhz: u64,
    /// Physical core count.
    pub physical_cores: u8,
    /// Logical core count.
    pub logical_cores: u8,
    /// Threads per physical core.
    pub threads_per_core: u8,
    /// Number of CPU sockets.
    pub sockets: u8,
    /// L1 cache size in KB.
    pub l1_cache_kb: u16,
    /// L2 cache size in KB.
    pub l2_cache_kb: u16,
    /// L3 cache size in KB.
    pub l3_cache_kb: u16,
    /// Hyper-Threading / SMT enabled.
    pub has_ht: bool,
    /// SIMD vector width in bits (e.g. 256 for AVX2).
    pub vector_width_bits: u32,
    /// Available SIMD features.
    pub simd_features: SimdFeatures,
}

/// SIMD instruction-set availability.
#[derive(Debug, Clone, Default)]
pub struct SimdFeatures {
    pub sse2: bool,
    pub sse4_2: bool,
    pub avx: bool,
    pub avx2: bool,
    pub avx512f: bool,
    pub fma: bool,
    pub neon: bool,
}

fn fallback_proc_cpu_info() -> Option<(&'static str, String, u64)> {
    let cpuinfo = fs::read_to_string("/proc/cpuinfo").ok()?;
    let mut vendor = "unknown";
    let mut model_name = None;
    let mut frequency_mhz = None;

    for line in cpuinfo.lines() {
        if let Some(value) = line.split(':').nth(1) {
            if line.starts_with("vendor_id") {
                let raw = value.trim();
                vendor = if raw.contains("AMD") {
                    "AMD"
                } else if raw.contains("Intel") {
                    "Intel"
                } else {
                    "unknown"
                };
            } else if line.starts_with("model name") && model_name.is_none() {
                model_name = Some(value.trim().to_string());
            } else if line.starts_with("cpu MHz") && frequency_mhz.is_none() {
                frequency_mhz = value
                    .trim()
                    .parse::<f64>()
                    .ok()
                    .map(|mhz| mhz.round() as u64);
            }
        }
    }

    Some((vendor, model_name.unwrap_or_default(), frequency_mhz.unwrap_or(0)))
}

impl CpuProfile {
    /// Probes the CPU via the hardware crate.
    pub fn detect() -> Self {
        let topo = sys::cpu::topology::detect();

        let (vendor, model_name, freq_hz, l1, l2, l3, has_ht) =
            match sys::cpu::detect_cpu_info() {
                Some(info) => {
                    let name = sys::cpu::info::model_name_str(&info).to_string();
                    (
                        info.vendor,
                        name,
                        info.frequency_hz,
                        info.l1_cache_kb,
                        info.l2_cache_kb,
                        info.l3_cache_kb,
                        info.has_ht,
                    )
                }
                None => ("unknown", String::new(), 0, 0, 0, 0, false),
            };

        let fallback = fallback_proc_cpu_info();
        let vendor = if vendor == "unknown" {
            fallback.as_ref().map(|(value, _, _)| *value).unwrap_or(vendor)
        } else {
            vendor
        };
        let model_name = if model_name.trim().is_empty() {
            fallback
                .as_ref()
                .map(|(_, value, _)| value.clone())
                .unwrap_or_default()
        } else {
            model_name
        };
        let frequency_mhz = (freq_hz / 1_000_000)
            .max(fallback.as_ref().map(|(_, _, mhz)| *mhz).unwrap_or(0));
        let logical_cores = if topo.logical_cores == 0 {
            std::thread::available_parallelism()
                .map(|value| value.get().min(u8::MAX as usize) as u8)
                .unwrap_or(1)
        } else {
            topo.logical_cores
        }
        .max(1);
        let physical_cores = topo.physical_cores.max(1).min(logical_cores);
        let threads_per_core = if topo.threads_per_core == 0 {
            (logical_cores / physical_cores.max(1)).max(1)
        } else {
            topo.threads_per_core.max(1)
        };

        let vec = sys::cpu::vector::detect();

        let simd_features = SimdFeatures {
            sse2: sys::cpu::features::has_feature("sse2"),
            sse4_2: sys::cpu::features::has_feature("sse4.2"),
            avx: sys::cpu::features::has_feature("avx"),
            avx2: sys::cpu::features::has_feature("avx2"),
            avx512f: sys::cpu::features::has_feature("avx512f"),
            fma: sys::cpu::features::has_feature("fma"),
            neon: sys::cpu::features::has_feature("neon"),
        };

        Self {
            vendor,
            model_name,
            frequency_mhz,
            physical_cores,
            logical_cores,
            threads_per_core,
            sockets: topo.sockets.max(1),
            l1_cache_kb: l1,
            l2_cache_kb: l2,
            l3_cache_kb: l3,
            has_ht,
            vector_width_bits: vec.width_bits,
            simd_features,
        }
    }

    /// Optimal tile width in pixels based on vector register width.
    ///
    /// Aligns tiles to SIMD boundaries so the inner loop can process
    /// multiple pixels per instruction without remainder.
    pub fn optimal_tile_width(&self) -> usize {
        // Each pixel = 3×f64 = 24 bytes; vector register holds N bytes.
        let vec_bytes = (self.vector_width_bits / 8) as usize;
        // Number of f64 lanes per register.
        let lanes = vec_bytes / 8;
        // Round up to nearest multiple of lanes, minimum 8.
        (lanes * 4).max(8)
    }

    /// Log CPU profile to stderr.
    pub fn log_summary(&self) {
        eprintln!(
            "cpu: {} {} @ {}MHz | {}C/{}T ({}s) | L1={}K L2={}K L3={}K | vec={}bit",
            self.vendor,
            self.model_name,
            self.frequency_mhz,
            self.physical_cores,
            self.logical_cores,
            self.sockets,
            self.l1_cache_kb,
            self.l2_cache_kb,
            self.l3_cache_kb,
            self.vector_width_bits,
        );
    }
}

// ─── Thread affinity ────────────────────────────────────────────────────

/// Pins the calling thread to a specific physical core.
///
/// Useful for render workers to avoid cache-thrashing from OS migration.
pub fn pin_thread_to_core(core_id: usize) {
    sys::cpu::affinity::pin_to_core(core_id);
}

/// Returns the current thread's affinity mask.
pub fn thread_affinity_mask() -> usize {
    sys::cpu::affinity::get_affinity()
}

/// Per-core frequency and temperature snapshot.
#[derive(Debug, Clone)]
pub struct CoreSnapshot {
    pub core_id: u32,
    pub frequency_hz: u64,
}

/// Detect individual core frequencies for load-aware scheduling.
pub fn detect_core_frequencies() -> Vec<CoreSnapshot> {
    let mut cores = [sys::cpu::cores::CoreInfo {
        core_id: 0,
        frequency_hz: 0,
        raw_temp: None,
    }; 128];
    let count = sys::cpu::cores::detect_cores(&mut cores);
    cores[..count]
        .iter()
        .map(|c| CoreSnapshot {
            core_id: c.core_id,
            frequency_hz: c.frequency_hz,
        })
        .collect()
}
