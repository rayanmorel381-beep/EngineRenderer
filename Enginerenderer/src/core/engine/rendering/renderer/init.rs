//! `Renderer` constructors and one-shot runtime probes for hardware/math sanity.

use std::sync::Mutex;

use crate::core::engine::acces_hardware::{self, DrmDriver, NativeHardwareBackend};
use crate::core::engine::math::{Mat4, Vec3, Vec4};
use crate::core::engine::rendering::{
    effects::{
        decals::decal_pass::Decal,
        particles::gpu_particles::{
            ColorGradient, GpuParticleSystem, MAX_PARTICLES, ParticleEmitter,
        },
    },
    lod::manager::LodManager,
    materials::sss::{SssPass, SssProfile},
    mesh::skinning::{
        AnimationClip, BlendShape, BoneWeight, MAX_SKELETON_BONES, Mat4 as SkinMat4, Skeleton,
        SkinnedMesh,
    },
    postprocessing::{
        ssr::{IblProbe, SsrConfig, SsrPass},
        svgf::SvgfDenoiser,
        taa::TaaAccumulator,
    },
    raytracing::{
        CpuRayTracer,
        caustics::{CausticPass, PhotonMap},
        rtao::RtaoConfig,
    },
    sdf::WorldSdf,
    shader_dispatcher::AdaptiveComputeDispatcher,
    terrain::{
        cdlod::HeightMap,
        foliage::{FoliageInstance, FoliageLayer},
    },
};

use super::state::Renderer;
use super::types::RenderPreset;

impl Renderer {
    fn with_resolution_from_backend(
        width: usize,
        height: usize,
        native_backend: &NativeHardwareBackend,
    ) -> Self {
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
                width,
                height,
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
                crate::runtime_log!(
                    "renderer: GPU framebuffer alloc failed, GPU command path disabled"
                );
            }
        } else {
            crate::runtime_log!("hardware: no DRM GPU available, using CPU-only rendering");
        }

        let native_gpu =
            acces_hardware::native_gpu_call(gpu.as_ref(), width.saturating_mul(height));
        crate::runtime_log!(
            "native-gpu: init_ok={} dispatch_ok={} framebuffer_ok={}",
            native_gpu.init_ok,
            native_gpu.dispatch_ok,
            native_gpu.framebuffer_ok,
        );

        let sdf_scene = crate::core::engine::rendering::raytracing::Scene {
            objects: Vec::new(),
            triangles: Vec::new(),
            sun: crate::core::engine::rendering::raytracing::DirectionalLight {
                direction: crate::core::engine::rendering::raytracing::Vec3::new(0.0, -1.0, 0.0),
                color: crate::core::engine::rendering::raytracing::Vec3::new(1.0, 1.0, 1.0),
                intensity: 1.0,
                angular_radius: 0.01,
            },
            area_lights: Vec::new(),
            sky_top: crate::core::engine::rendering::raytracing::Vec3::new(0.1, 0.2, 0.4),
            sky_bottom: crate::core::engine::rendering::raytracing::Vec3::new(0.5, 0.5, 0.5),
            exposure: 1.0,
            volume: crate::core::engine::rendering::effects::volumetric_effects::medium::VolumetricMedium::vacuum(),
            hdri: None,
            solar_elevation: 0.3,
        };
        let world_sdf = WorldSdf::build_from_scene(&sdf_scene, 8);
        let sdf_probe = world_sdf.sample(crate::core::engine::rendering::raytracing::Vec3::ZERO);
        let sdf_grad = world_sdf.gradient(crate::core::engine::rendering::raytracing::Vec3::ZERO);
        let sdf_march = world_sdf.march(
            crate::core::engine::rendering::raytracing::Vec3::new(0.0, 5.0, 0.0),
            crate::core::engine::rendering::raytracing::Vec3::new(0.0, -1.0, 0.0),
            50.0,
            64,
        );
        let sdf_irr = world_sdf
            .sample_irradiance_hint(crate::core::engine::rendering::raytracing::Vec3::ZERO, 1.0);
        crate::runtime_log!(
            "sdf: cells={} probe={:.4} grad_len={:.4} march_hit={} irr={:.4}",
            world_sdf.cell_count(),
            sdf_probe,
            sdf_grad.length(),
            sdf_march.is_some(),
            sdf_irr,
        );

        let ddgi = Self::build_ddgi_from_scene_bounds(-50.0, -5.0, -50.0, 50.0, 30.0, 50.0);
        let rtao_config = RtaoConfig::default();
        crate::runtime_log!(
            "renderer: RTAO samples={} radius={:.2} bias={:.4} indirect_bounces={} | DDGI probes={}",
            rtao_config.samples,
            rtao_config.radius,
            rtao_config.bias,
            rtao_config.indirect_bounces,
            ddgi.probes.len(),
        );

        let empty_skeletons: Vec<Skeleton> = Vec::new();
        let empty_meshes: Vec<SkinnedMesh> = Vec::new();
        let empty_clips: Vec<AnimationClip> = Vec::new();
        let empty_decals: Vec<Decal> = Vec::new();
        let empty_foliage: Vec<FoliageInstance> = Vec::new();

        let render_thread_cores = hw_caps.logical_cores;
        let job_system_cores = cpu_profile.logical_cores;

        let mut renderer = Self {
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
            taa: Mutex::new(TaaAccumulator::new(width, height)),
            svgf: Mutex::new(SvgfDenoiser::new(width, height)),
            world_sdf: Mutex::new(Some(world_sdf)),
            ddgi: Mutex::new(ddgi),
            rtao_config,
            photon_map: Mutex::new(PhotonMap::new()),
            caustic_pass: CausticPass::new(0.5),
            sss_pass: SssPass::new(SssProfile::skin(), 8, 16),
            ssr_pass: SsrPass::new(SsrConfig::default()),
            ibl_probe: IblProbe::new(crate::core::engine::rendering::raytracing::Vec3::ZERO, 64),
            fsr_config: None,
            particles: Mutex::new(GpuParticleSystem::new(MAX_PARTICLES / 16)),
            decals: empty_decals,
            skeletons: Mutex::new(empty_skeletons),
            skinned_meshes: Mutex::new(empty_meshes),
            animation_clips: empty_clips,
            anim_state_machine: Mutex::new(
                crate::core::animation::state_machine::AnimStateMachine::new(Vec::new()),
            ),
            texture_streamer: Mutex::new(
                crate::core::engine::rendering::texture::virtual_texture::TextureStreamer::new(64),
            ),
            terrain: None,
            foliage_layer: FoliageLayer::new(64, 64),
            foliage_instances: Mutex::new(empty_foliage),
            raster_pipeline: crate::core::engine::rendering::raster::pipeline::RasterPipeline::new(
            ),
            secondary_motion: Mutex::new(
                crate::core::animation::secondary_motion::SecondaryMotionSystem::new(),
            ),
            render_thread: Some(
                crate::core::engine::rendering::renderer::render_thread::RenderThread::spawn(
                    render_thread_cores as usize,
                ),
            ),
            job_system: crate::core::scheduler::job_system::JobSystem::new(
                job_system_cores as usize,
            ),
        };

        let hm_new = HeightMap::new(
            vec![0.0_f32; 4],
            2,
            2,
            crate::core::engine::rendering::raytracing::Vec3::new(10.0, 5.0, 10.0),
        );
        let origin_wp = hm_new.world_position(0.0, 0.0);
        let hm_flat = HeightMap::flat(
            64,
            64,
            crate::core::engine::rendering::raytracing::Vec3::new(100.0, 10.0, 100.0),
        );
        crate::runtime_log!("terrain: probe_wp={:?}", origin_wp);
        renderer.set_terrain(hm_flat, 4, 100.0);

        renderer.add_decal(Decal::new(
            crate::core::engine::rendering::raytracing::Vec3::ZERO,
            crate::core::engine::rendering::raytracing::Vec3::new(0.0, 1.0, 0.0),
            crate::core::engine::rendering::raytracing::Vec3::new(2.0, 1.0, 2.0),
            crate::core::engine::rendering::raytracing::Vec3::new(0.8, 0.6, 0.4),
        ));

        let emitter = ParticleEmitter::new(crate::core::engine::rendering::raytracing::Vec3::ZERO);
        let gradient = ColorGradient::new(vec![
            (
                0.0,
                crate::core::engine::rendering::raytracing::Vec3::new(1.0, 0.5, 0.0),
                1.0_f64,
            ),
            (
                1.0,
                crate::core::engine::rendering::raytracing::Vec3::ZERO,
                0.0_f64,
            ),
        ]);
        crate::runtime_log!(
            "particles: max={} drag={:.3} gradient_stops={}",
            MAX_PARTICLES,
            emitter.drag,
            gradient.stops.len(),
        );
        Self::lock_unpoisoned(&renderer.particles).add_emitter(emitter);

        let mut bw = BoneWeight::single(0);
        bw.normalize();
        let scale_mat = SkinMat4::scale_mat4(1.0);
        let blended_mat = scale_mat.add_scaled(&SkinMat4::identity(), 0.0);
        crate::runtime_log!(
            "skinning: bw={:.3} sdiag={:.3} bdiag={:.3} max_bones={}",
            bw.weights[0],
            scale_mat.cols[0][0],
            blended_mat.cols[0][0],
            MAX_SKELETON_BONES,
        );

        let skeleton = Skeleton::new(vec![]);
        let bone_count = skeleton.bone_count();
        renderer.add_skeleton(skeleton);
        crate::runtime_log!("renderer: default_skeleton_bones={}", bone_count);

        let mut mesh = SkinnedMesh::new(vec![]);
        mesh.add_blend_shape(BlendShape {
            name: "default".to_string(),
            weight: 0.0,
            delta_positions: vec![],
            delta_normals: vec![],
        });
        renderer.add_skinned_mesh(mesh);

        renderer.add_animation_clip(AnimationClip {
            name: "idle".to_string(),
            duration: 1.0,
            tracks: vec![],
        });

        crate::runtime_log!(
            "sss: skin={:?} wax={:?} marble={:?}",
            SssProfile::skin().albedo,
            SssProfile::wax().albedo,
            SssProfile::marble().albedo,
        );
        renderer.set_sss_profile(SssProfile::skin(), 8, 16);
        renderer.set_ssr_config(SsrConfig::default());
        renderer.set_fsr(width * 2, height * 2);

        renderer
    }

    pub fn with_resolution_using_backend(
        width: usize,
        height: usize,
        backend: &NativeHardwareBackend,
    ) -> Self {
        Self::with_resolution_from_backend(width, height, backend)
    }

    pub fn with_resolution(width: usize, height: usize) -> Self {
        let native_backend = acces_hardware::NativeHardwareBackend::detect();
        Self::with_resolution_from_backend(width, height, &native_backend)
    }

    pub fn default_cpu_hd() -> Self {
        Self::with_resolution(1920, 1080)
    }

    pub fn from_preset(preset: RenderPreset) -> Self {
        match preset {
            RenderPreset::AnimationFast | RenderPreset::PreviewCpu => Self::default_cpu_hd(),
            RenderPreset::UltraHdCpu => Self::with_resolution(2560, 1440),
            RenderPreset::ProductionReference => Self::with_resolution(3840, 2160),
        }
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
}
