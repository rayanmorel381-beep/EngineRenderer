
pub mod pipeline;
pub mod scene_builder;
pub mod types;

use std::sync::{Arc, Mutex, MutexGuard};

use crate::core::engine::acces_hardware::{
    self, CpuProfile, DrmDriver, GpuRenderBackend, HardwareCapabilities, KernelConfig,
    NativeHardwareBackend, RamRuntimeConfig,
};
use crate::core::engine::math::{Mat4, Vec3, Vec4};
use crate::core::engine::rendering::{
    lod::manager::LodManager,
    raytracing::{CpuRayTracer, RenderConfig, Scene},
    raytracing::acceleration::BvhNode,
    framebuffer::FrameBuffer,
    shader_dispatcher::{AdaptiveComputeDispatcher, TileComputeDescriptor},
};

use types::RenderPreset;

#[derive(Debug, Clone)]
struct CachedBvh {
    signature: u64,
    object_count: usize,
    triangle_count: usize,
    bvh: Arc<BvhNode>,
}

impl std::fmt::Debug for Renderer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Renderer")
            .field("width", &self.width)
            .field("height", &self.height)
            .field("lod_manager", &self.lod_manager)
            .field("hw_caps", &self.hw_caps)
            .field("cpu_profile", &self.cpu_profile)
            .field("ram_config", &self.ram_config)
            .field("gpu", &self.gpu)
            .finish()
    }
}

pub struct Renderer {
    pub(super) width: usize,
    pub(super) height: usize,
    pub(super) tracer: CpuRayTracer,
    pub(super) lod_manager: LodManager,
    pub hw_caps: HardwareCapabilities,
    pub(super) cpu_profile: CpuProfile,
    pub(super) ram_config: RamRuntimeConfig,
    pub(super) gpu: Option<GpuRenderBackend>,
    bvh_cache: Mutex<Option<CachedBvh>>,
    compute_dispatcher: Mutex<AdaptiveComputeDispatcher>,
}

impl Renderer {
        fn with_resolution_from_backend(width: usize, height: usize, native_backend: &NativeHardwareBackend) -> Self {
            let hw_caps = native_backend.hw_caps().clone();
            hw_caps.log_summary();

            let cpu_profile = native_backend.cpu_profile().clone();
            let ram_config = native_backend.ram_config();
            let ram_available = ram_config.available_bytes.unwrap_or(ram_config.total_bytes);
            crate::runtime_log!(
                "native-ram: page={} total={}MB available={}MB",
                ram_config.page_size,
                ram_config.total_bytes / (1024 * 1024),
                ram_available / (1024 * 1024),
            );
            let native_cpu = acces_hardware::native_cpu_call(&cpu_profile);
            let optimal_workgroup = acces_hardware::arch_optimal_workgroup();
            crate::runtime_log!(
                "native-cpu: arch={} logical_cores={} vec={}bit workgroup={}",
                native_cpu.architecture,
                native_cpu.logical_cores,
                native_cpu.vector_width_bits,
                optimal_workgroup,
            );
            let (matrix_probe, basis_probe, tone_probe) = Self::runtime_transform_probe(width, height);
            crate::runtime_log!(
                "native-math: mvp_probe={:.4} basis_probe={:.4} tone_probe={:.4}",
                matrix_probe,
                basis_probe,
                tone_probe,
            );
            cpu_profile.log_summary();

            let pixel_bytes = (width * height * 24) as u64;
            let max_bytes = hw_caps.max_framebuffer_bytes_for_input(width.saturating_mul(height));
            if pixel_bytes > max_bytes {
                crate::runtime_log!(
                    "renderer: WARNING {}×{} needs {}MB, only {}MB available — consider lower resolution",
                    width, height,
                    pixel_bytes / (1024 * 1024),
                    max_bytes / (1024 * 1024),
                );
            }

            let compute_dispatcher = AdaptiveComputeDispatcher::with_native_backend(native_backend);
            let mut gpu = native_backend.probe_gpu_backend();
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
                    crate::runtime_log!(
                        "renderer: GPU DRM backend active (driver={} family={} vram={}MB, CU={}, device={:04x})",
                        g.driver_name(),
                        driver_family,
                        g.vram_bytes() / (1024 * 1024),
                        g.compute_units(),
                        g.info().device_id,
                    );
                } else {
                    crate::runtime_log!(
                        "renderer: GPU DRM backend active (driver={} family={}, telemetry unavailable)",
                        g.driver_name(),
                        driver_family,
                    );
                }
                if let Some(dt_compatible) = g.dt_compatible.as_deref() {
                    crate::runtime_log!("renderer: GPU device-tree node={}", dt_compatible);
                }
                if let Some(ptr) = g.alloc_framebuffer(width, height) {
                    crate::runtime_log!(
                        "renderer: GPU framebuffer allocated ({}×{}, ptr={:p}, gem={})",
                        width,
                        height,
                        ptr,
                        g.has_gem_framebuffer(),
                    );
                } else {
                    crate::runtime_log!("renderer: GPU framebuffer alloc failed, GPU command path disabled");
                }
            } else {
                crate::runtime_log!("hardware: no DRM GPU available, using CPU-only rendering");
            }

            let native_gpu = acces_hardware::native_gpu_call(gpu.as_ref(), width.saturating_mul(height));
            crate::runtime_log!(
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
                ram_config,
                gpu,
                bvh_cache: Mutex::new(None),
                compute_dispatcher: Mutex::new(compute_dispatcher),
            }
        }

        pub fn with_resolution_using_backend(width: usize, height: usize, backend: &NativeHardwareBackend) -> Self {
            Self::with_resolution_from_backend(width, height, backend)
        }

    fn runtime_transform_probe(width: usize, height: usize) -> (f32, f32, f32) {
        let aspect = (width.max(1) as f32) / (height.max(1) as f32);
        let eye = Vec3::new(0.0, 1.5, 4.0);
        let center = Vec3::new(0.0, 0.0, 0.0);
        let up = Vec3::new(0.0, 1.0, 0.0);

        let view = Mat4::look_at(eye, center, up);
        let projection = Mat4::perspective(std::f32::consts::FRAC_PI_3, aspect, 0.1, 2500.0);
        let model = Mat4::translate(0.0, 0.0, 0.0)
            * Mat4::rotate(Vec3::new(0.0, 1.0, 0.0), 0.25)
            * Mat4::scale(1.0, 1.0, 1.0);
        let mvp = projection * view * model * Mat4::IDENTITY;

        let m = mvp.as_flat_array();
        let basis_forward = (center - eye).normalize();
        let basis_right = basis_forward.cross(up).normalize();
        let alignment = basis_forward.dot(up.normalize());

        let tone = Vec4::new(0.95, 0.93, 0.90, 1.0);
        let luma = tone.rgb().length() * tone.w;
        let matrix_probe = m[0].abs() + m[5].abs() + m[10].abs() + m[15].abs();
        let basis_probe = basis_right.length() + alignment.abs();

        (matrix_probe, basis_probe, luma)
    }

    fn persistent_bvh_cache_path(signature: u64, object_count: usize, triangle_count: usize) -> std::path::PathBuf {
        let mut path = std::env::temp_dir();
        path.push("enginerenderer-cache");
        path.push("bvh-cache");
        path.push(format!("{:016x}-{}-{}.bvh", signature, object_count, triangle_count));
        path
    }

    pub fn with_resolution(width: usize, height: usize) -> Self {
        let native_backend = acces_hardware::NativeHardwareBackend::detect();
        Self::with_resolution_from_backend(width, height, &native_backend)
    }

    pub fn default_cpu_hd() -> Self {
        Self::with_resolution(1920, 1080)
    }

    pub fn worker_threads(&self) -> usize {
        self.hw_caps
            .optimal_render_threads_for_input(self.width.saturating_mul(self.height))
    }

    pub fn from_preset(preset: RenderPreset) -> Self {
        match preset {
            RenderPreset::AnimationFast | RenderPreset::PreviewCpu => Self::default_cpu_hd(),
            RenderPreset::UltraHdCpu => Self::with_resolution(2560, 1440),
            RenderPreset::ProductionReference => Self::with_resolution(3840, 2160),
        }
    }

    pub fn config_for(&self, preset: RenderPreset) -> RenderConfig {
        let (cfg_w, cfg_h) = match preset {
            RenderPreset::AnimationFast => (self.width, self.height),
            RenderPreset::PreviewCpu => (self.width.min(1920), self.height.min(1080)),
            RenderPreset::UltraHdCpu => (self.width, self.height),
            RenderPreset::ProductionReference => (self.width, self.height),
        };
        let max_threads = self
            .hw_caps
            .optimal_render_threads_for_input(cfg_w.saturating_mul(cfg_h));
        let threads = self.worker_threads().min(max_threads).max(1);
        match preset {
            RenderPreset::AnimationFast => RenderConfig {
                width: self.width,
                height: self.height,
                base_samples_per_pixel: 1,
                max_bounces: 1,
                max_distance: 160.0,
                thread_count: threads,
                denoise_strength: 0.0,
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

    pub(super) fn cached_bvh_for_scene(&self, scene: &Scene) -> (Option<Arc<BvhNode>>, bool) {
        let signature = scene.geometry_signature();
        let object_count = scene.objects.len();
        let triangle_count = scene.triangles.len();
        let cache_path = Self::persistent_bvh_cache_path(signature, object_count, triangle_count);

        {
            let cache = Self::lock_unpoisoned(&self.bvh_cache);
            if let Some(cached) = cache.as_ref()
                && cached.signature == signature
                && cached.object_count == object_count
                && cached.triangle_count == triangle_count
            {
                return (Some(Arc::clone(&cached.bvh)), true);
            }
        }

        let t_load = acces_hardware::precise_timestamp_ns();
        if let Ok(loaded) = BvhNode::load_from_path(&cache_path, scene) {
            let loaded = Arc::new(loaded);
            let stats = loaded.stats();
            let load_ms = acces_hardware::elapsed_ms(t_load, acces_hardware::precise_timestamp_ns());
            crate::runtime_log!(
                "tracer: BVH disk-load {:.2}ms (nodes={} leaves={})",
                load_ms,
                stats.node_count,
                stats.leaf_count,
            );
            let mut cache = Self::lock_unpoisoned(&self.bvh_cache);
            *cache = Some(CachedBvh {
                signature,
                object_count,
                triangle_count,
                bvh: Arc::clone(&loaded),
            });
            return (Some(loaded), true);
        }

        let t_build = acces_hardware::precise_timestamp_ns();
        let built = BvhNode::build(scene).map(Arc::new);
        if let Some(ref bvh) = built {
            let stats = bvh.stats();
            let build_ms = acces_hardware::elapsed_ms(t_build, acces_hardware::precise_timestamp_ns());
            crate::runtime_log!(
                "tracer: BVH build {:.2}ms (nodes={} leaves={})",
                build_ms,
                stats.node_count,
                stats.leaf_count,
            );
            if let Err(error) = bvh.save_to_path(&cache_path) {
                crate::runtime_log!("tracer: BVH disk-save failed: {}", error);
            }
        }
        let mut cache = Self::lock_unpoisoned(&self.bvh_cache);
        *cache = built.as_ref().map(|bvh| CachedBvh {
            signature,
            object_count,
            triangle_count,
            bvh: Arc::clone(bvh),
        });
        (built, false)
    }

    pub(super) fn submit_compute_workload(&self, scene: &Scene, width: usize, height: usize) -> bool {
        let mut dispatcher = Self::lock_unpoisoned(&self.compute_dispatcher);
        if dispatcher.device_count() == 0 {
            return false;
        }

        let workgroup_size = acces_hardware::arch_optimal_workgroup().max(1);
        let kernel_x = (workgroup_size.min(8)) as u16;
        let kernel_y = workgroup_size.div_ceil(kernel_x as usize).max(1) as u16;
        let tile_size = (kernel_x as usize)
            .saturating_mul(kernel_y as usize)
            .max(16);
        let kernel = KernelConfig::new(kernel_x, kernel_y, 1)
            .with_shared_memory((workgroup_size * 32) as u32);
        let kernel_source = format!(
            "kernel trace_tile(u32 tile_id, u32 object_count, u32 triangle_count, u64 scene_signature) {{ u32 lanes = {}; u32 tiles_x = {}; u32 tiles_y = {}; }}",
            workgroup_size,
            width.div_ceil(tile_size),
            height.div_ceil(tile_size),
        );

        dispatcher
            .dispatch_tile_compute_with_kernel(TileComputeDescriptor {
                image_width: width,
                image_height: height,
                tile_size,
                config: kernel,
                kernel_name: "trace-tile",
                kernel_source: kernel_source.as_bytes(),
                scene_signature: scene.geometry_signature(),
                object_count: scene.objects.len(),
                triangle_count: scene.triangles.len(),
            })
            .is_ok()
    }
    fn lock_unpoisoned<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
        match mutex.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        }
    }

    pub(super) fn upload_framebuffer_to_gpu(&self, framebuffer: &FrameBuffer) -> bool {
        let Some(gpu) = self.gpu.as_ref() else {
            return false;
        };
        let mut rgb = vec![0u8; framebuffer.width * framebuffer.height * 3];
        for (pixel, out) in framebuffer.color.iter().zip(rgb.chunks_exact_mut(3)) {
            let corrected = pixel.clamp(0.0, 1.0).powf(1.0 / 2.2);
            out[0] = (corrected.x * 255.0).round() as u8;
            out[1] = (corrected.y * 255.0).round() as u8;
            out[2] = (corrected.z * 255.0).round() as u8;
        }
        gpu.write_framebuffer_rgb(&rgb)
    }

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
                        crate::runtime_log!("gpu: fence cs_id={} sync={:.2}ms", cs_id, elapsed);
                        return Some(elapsed);
                    }
                    Err(e) => {
                        crate::runtime_log!("gpu: fence submit failed ({}), fallback sync", e);
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
