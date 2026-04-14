//! High-level render orchestration: presets, scene dispatch, and
//! report generation.
//!
//! - [`types`] — [`RenderPreset`](types::RenderPreset) and
//!   [`RenderReport`](types::RenderReport).
//! - [`pipeline`] — Heavy rendering methods
//!   (`render_scene_to_file`, `render`, etc.).
//! - [`scene_builder`] — Realistic scene construction helpers.

pub mod pipeline;
pub mod scene_builder;
pub mod types;

use crate::core::engine::acces_hardware::{
    self, DrmDriver, HardwareCapabilities, GpuRenderBackend, CpuProfile,
};
use crate::core::engine::rendering::{
    lod::manager::LodManager,
    raytracing::{CpuRayTracer, RenderConfig},
};

use types::RenderPreset;

/// Hybrid CPU/GPU offline renderer.
///
/// Hardware is probed once at construction. The stored `hw_caps` and
/// `cpu_profile` drive every scheduling and allocation decision —
/// thread count, tile sizing, memory guards, GPU framebuffer, etc.
#[derive(Debug)]
pub struct Renderer {
    /// Target width.
    pub(super) width: usize,
    /// Target height.
    pub(super) height: usize,
    /// Ray-tracing back-end.
    pub(super) tracer: CpuRayTracer,
    /// Level-of-detail manager.
    pub(super) lod_manager: LodManager,
    /// Hardware capabilities snapshot (CPU topology + memory + GPU detect).
    pub hw_caps: HardwareCapabilities,
    /// Detailed CPU profile (SIMD, caches, frequencies).
    pub(super) cpu_profile: CpuProfile,
    /// GPU DRM backend (None if no GPU or DRM unavailable).
    pub(super) gpu: Option<GpuRenderBackend>,
}

impl Renderer {
    /// Creates a renderer targeting `width × height`.
    ///
    /// Probes CPU topology, SIMD features, per-core frequencies, memory,
    /// and attempts to open the GPU via DRM. All subsequent operations
    /// use the stored snapshots — no re-detection.
    pub fn with_resolution(width: usize, height: usize) -> Self {
        let hw_caps = HardwareCapabilities::detect();
        hw_caps.log_summary();

        let cpu_profile = acces_hardware::CpuProfile::detect();
        let native_cpu = acces_hardware::native_cpu_call(&cpu_profile);
        let optimal_workgroup = acces_hardware::arch_optimal_workgroup();
        eprintln!(
            "native-cpu: arch={} logical_cores={} vec={}bit workgroup={}",
            native_cpu.architecture,
            native_cpu.logical_cores,
            native_cpu.vector_width_bits,
            optimal_workgroup,
        );
        cpu_profile.log_summary();

        // ── Memory guard ────────────────────────────────────────────
        let pixel_bytes = (width * height * 24) as u64; // 3×f64
        let max_bytes = hw_caps.max_framebuffer_bytes_for_input(width.saturating_mul(height));
        if pixel_bytes > max_bytes {
            eprintln!(
                "renderer: WARNING {}×{} needs {}MB, only {}MB available — consider lower resolution",
                width, height,
                pixel_bytes / (1024 * 1024),
                max_bytes / (1024 * 1024),
            );
        }

        let mut gpu = GpuRenderBackend::try_init();
        if let Some(ref mut g) = gpu {
            let driver_family = match g.driver() {
                DrmDriver::Amdgpu | DrmDriver::Radeon => "amd",
                DrmDriver::I915 | DrmDriver::Xe => "intel",
                DrmDriver::Nouveau => "nvidia-open",
                DrmDriver::Mali => "arm",
                DrmDriver::Agx => "apple",
                DrmDriver::Msm => "qualcomm",
                DrmDriver::Unknown => "unknown",
            };
            if g.has_valid_metrics() {
                eprintln!(
                    "renderer: GPU DRM backend active (driver={} family={} vram={}MB, CU={}, device={:04x})",
                    g.driver_name(),
                    driver_family,
                    g.vram_bytes() / (1024 * 1024),
                    g.compute_units(),
                    g.info().device_id,
                );
            } else {
                eprintln!(
                    "renderer: GPU DRM backend active (driver={} family={}, telemetry unavailable)",
                    g.driver_name(),
                    driver_family,
                );
            }
            if let Some(dt_compatible) = g.dt_compatible.as_deref() {
                eprintln!("renderer: GPU device-tree node={}", dt_compatible);
            }
            // Allocate a GPU-visible framebuffer in GTT so the DRM
            // command path (submit_ib / sync) is functional.
            if let Some(ptr) = g.alloc_framebuffer(width, height) {
                eprintln!(
                    "renderer: GPU framebuffer allocated ({}×{}, ptr={:p}, gem={})",
                    width,
                    height,
                    ptr,
                    g.has_gem_framebuffer(),
                );
            } else {
                eprintln!("renderer: GPU framebuffer alloc failed, GPU command path disabled");
            }
        } else {
            eprintln!("hardware: no DRM GPU available, using CPU-only rendering");
        }

        let native_gpu = acces_hardware::native_gpu_call(gpu.as_ref(), width.saturating_mul(height));
        eprintln!(
            "native-gpu: init_ok={} dispatch_ok={} framebuffer_ok={}",
            native_gpu.init_ok,
            native_gpu.dispatch_ok,
            native_gpu.framebuffer_ok,
        );

        Self {
            width,
            height,
            tracer: CpuRayTracer,
            lod_manager: LodManager::default(),
            hw_caps,
            cpu_profile,
            gpu,
        }
    }

    /// Convenience constructor for 1920 × 1080.
    pub fn default_cpu_hd() -> Self {
        Self::with_resolution(1920, 1080)
    }

    pub fn worker_threads(&self) -> usize {
        self.hw_caps
            .optimal_render_threads_for_input(self.width.saturating_mul(self.height))
    }

    /// Creates a renderer configured for the given [`RenderPreset`].
    pub fn from_preset(preset: RenderPreset) -> Self {
        match preset {
            RenderPreset::AnimationFast | RenderPreset::PreviewCpu => Self::with_resolution(1920, 1080),
            RenderPreset::UltraHdCpu => Self::with_resolution(2560, 1440),
            RenderPreset::ProductionReference => Self::with_resolution(3840, 2160),
        }
    }

    /// Maps a [`RenderPreset`] to a concrete [`RenderConfig`].
    ///
    /// Uses stored `hw_caps` — no re-detection per frame.
    pub fn config_for(&self, preset: RenderPreset) -> RenderConfig {
        let (cfg_w, cfg_h) = match preset {
            RenderPreset::AnimationFast => (self.width, self.height),
            RenderPreset::PreviewCpu => (self.width.min(1920), self.height.min(1080)),
            RenderPreset::UltraHdCpu => (self.width, self.height),
            RenderPreset::ProductionReference => (self.width, self.height),
        };
        let threads = self
            .hw_caps
            .optimal_render_threads_for_input(cfg_w.saturating_mul(cfg_h));
        match preset {
            RenderPreset::AnimationFast => RenderConfig {
                width: self.width,
                height: self.height,
                base_samples_per_pixel: 1,
                max_bounces: 1,
                max_distance: 300.0,
                thread_count: threads,
                denoise_strength: 0.22,
                adaptive_sampling: false,
                firefly_threshold: 3.0,
                denoise_radius: 2,
            },
            RenderPreset::PreviewCpu => RenderConfig {
                width: cfg_w,
                height: cfg_h,
                base_samples_per_pixel: 4,
                max_bounces: 1,
                max_distance: 480.0,
                thread_count: threads,
                denoise_strength: 0.10,
                adaptive_sampling: true,
                firefly_threshold: 2.20,
                denoise_radius: 1,
            },
            RenderPreset::UltraHdCpu => RenderConfig {
                width: self.width,
                height: self.height,
                base_samples_per_pixel: 16,
                max_bounces: 3,
                max_distance: 1200.0,
                thread_count: threads,
                denoise_strength: 0.12,
                adaptive_sampling: true,
                firefly_threshold: 1.90,
                denoise_radius: 1,
            },
            RenderPreset::ProductionReference => RenderConfig {
                width: self.width,
                height: self.height,
                base_samples_per_pixel: 32,
                max_bounces: 4,
                max_distance: 1500.0,
                thread_count: threads,
                denoise_strength: 0.14,
                adaptive_sampling: true,
                firefly_threshold: 1.60,
                denoise_radius: 1,
            },
        }
    }

    /// GPU info string for logging.
    pub(super) fn gpu_info_tag(&self) -> String {
        match &self.gpu {
            Some(g) if g.has_valid_metrics() => format!(
                "gpu(driver={} vram={}MB cu={} {:04x})",
                g.driver_name(),
                g.vram_bytes() / (1024 * 1024),
                g.compute_units(),
                g.info().device_id,
            ),
            Some(g) => format!("gpu(driver={} telemetry=unavailable)", g.driver_name()),
            None => "cpu-only".to_string(),
        }
    }

    /// SIMD tag from the stored CPU profile (e.g. "AVX2", "SSE4.2").
    pub(super) fn simd_tag(&self) -> &'static str {
        let s = &self.cpu_profile.simd_features;
        if s.avx512f { "AVX-512" }
        else if s.avx2 { "AVX2" }
        else if s.avx { "AVX" }
        else if s.fma { "FMA" }
        else if s.sse4_2 { "SSE4.2" }
        else if s.sse2 { "SSE2" }
        else if s.neon { "NEON" }
        else { "scalar" }
    }

    /// Submit a GPU fence instruction buffer and sync, reporting timing.
    ///
    /// Only submits the NOP IB if a GPU framebuffer GEM object was
    /// successfully allocated (confirming the DRM command path is functional).
    /// Otherwise just does a passive sync.
    pub(super) fn gpu_fence_and_sync(&self) -> Option<f64> {
        if let Some(ref g) = self.gpu {
            let t0 = acces_hardware::precise_timestamp_ns();

            if g.has_active_framebuffer() && !matches!(g.driver(), DrmDriver::Radeon) {
                let nop_ib: [u32; 4] = [
                    0xC0021000,
                    0x00000000,
                    0x00000000,
                    0x00000000,
                ];
                match g.submit_ib(&nop_ib) {
                    Ok(cs_id) => {
                        g.sync_framebuffer();
                        let elapsed = acces_hardware::elapsed_ms(t0, acces_hardware::precise_timestamp_ns());
                        eprintln!("gpu: fence cs_id={} sync={:.2}ms", cs_id, elapsed);
                        return Some(elapsed);
                    }
                    Err(e) => {
                        eprintln!("gpu: fence submit failed ({}), fallback sync", e);
                    }
                }
            }

            // Passive sync (no IB submission)
            g.sync_framebuffer();
            let elapsed = acces_hardware::elapsed_ms(t0, acces_hardware::precise_timestamp_ns());
            Some(elapsed)
        } else {
            None
        }
    }
}
