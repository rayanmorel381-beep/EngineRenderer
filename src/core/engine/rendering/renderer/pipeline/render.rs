
use std::{
    error::Error,
    path::{Path, PathBuf},
};

use crate::core::engine::acces_hardware::{
    precise_timestamp_ns, elapsed_ms as hw_elapsed_ms, HwInstant,
};

use crate::core::engine::rendering::{
    culling::helpers::sphere_occludes,
    culling::scene_culler::SceneCuller,
    effects::shadow_map::cascade::ShadowCascade,
    environment::scattering::rayleigh_scatter,
    framebuffer::FrameBuffer,
    postprocessing::depth_of_field::DepthOfField,
    postprocessing::processor::{PostProcessor, reinhard_tonemap},
    preprocessing::scene_preprocessor::ScenePreprocessor,
    raytracing::{Camera, Scene},
    raytracing::acceleration::BvhNode,
    utils::{ev100_from_luminance, exposure_from_ev100},
};
use crate::core::scheduler::adaptive::TileScheduler;
use super::super::scene_builder::build_realistic_scene;

use super::super::types::{RenderPreset, RenderReport};
use super::super::Renderer;

#[derive(Debug, Clone, Copy)]
pub struct AnimationFramePressure {
    pub preset: RenderPreset,
    pub sample_pressure_scale: f64,
}

impl Renderer {
    pub fn render_to_file<P: AsRef<Path>>(
        &self,
        output_path: P,
        preset: RenderPreset,
    ) -> Result<RenderReport, Box<dyn Error>> {
        let aspect_ratio = self.width as f64 / self.height.max(1) as f64;
        let (scene, camera) = build_realistic_scene(aspect_ratio);
        self.render_scene_to_file(&scene, &camera, output_path, preset)
    }

    pub fn render_scene_to_file<P: AsRef<Path>>(
        &self,
        scene: &Scene,
        camera: &Camera,
        output_path: P,
        preset: RenderPreset,
    ) -> Result<RenderReport, Box<dyn Error>> {
        self.render_scene_to_file_with_pressure(scene, camera, output_path, preset, 1.0)
    }

    pub fn render_scene_to_file_with_pressure<P: AsRef<Path>>(
        &self,
        scene: &Scene,
        camera: &Camera,
        output_path: P,
        preset: RenderPreset,
        sample_pressure_scale: f64,
    ) -> Result<RenderReport, Box<dyn Error>> {
        let mut config = self.config_for(preset);
        let is_preview = matches!(preset, RenderPreset::PreviewCpu);
        let input_scene_objects = scene.objects.len() + scene.triangles.len();

        // ── Scene preprocessing ─────────────────────────────────────────
        let preprocessed = ScenePreprocessor::analyze(scene, camera);
        let adaptive_budget_ms = match preset {
            RenderPreset::AnimationFast | RenderPreset::PreviewCpu => 16.0,
            RenderPreset::UltraHdCpu => 120.0,
            RenderPreset::ProductionReference => 260.0,
        };
        let quality = crate::core::engine::rendering::preprocessing::scene_preprocessor::AdaptiveQualitySettings::from_analysis(
            &preprocessed.analysis,
            adaptive_budget_ms,
        );
        let minimum_spp = match preset {
            RenderPreset::AnimationFast => 1,
            RenderPreset::PreviewCpu => 2,
            RenderPreset::UltraHdCpu => 8,
            RenderPreset::ProductionReference => 16,
        };
        config.base_samples_per_pixel = ((config.base_samples_per_pixel as f64 * quality.sample_multiplier)
            .round() as u32)
            .max(minimum_spp);
        apply_runtime_sampling_pressure(&mut config, preset, sample_pressure_scale);
        config.max_bounces = config.max_bounces.min(quality.bounce_limit.max(2));
        let cam_near = preprocessed.camera_info.near_plane;
        let cam_far = preprocessed.camera_info.far_plane;
        let scene_radius = preprocessed.analysis.scene_radius;

        // ── Frustum + culling ───────────────────────────────────────────
        let frustum = self.build_frustum(camera, &config, cam_near, cam_far);

        let contribution_threshold = if is_preview {
            (quality.ao_quality * 0.18).clamp(0.08, 0.20)
        } else {
            (quality.ao_quality * 0.08).clamp(0.02, 0.08)
        };
        let culler = SceneCuller::new(config.max_distance)
            .with_screen_params(60.0_f64.to_radians(), config.height as f64)
            .with_contribution_threshold(contribution_threshold)
            .with_backface_culling(is_preview);

        let (distance_culled, cull_stats) = culler.cull_scene_with_stats(scene, camera);
        let mut render_scene = culler.cull_with_frustum(&distance_culled, &frustum);
        crate::runtime_log!(
            "culled {:.0}% spheres, {:.0}% triangles",
            cull_stats.sphere_ratio() * 100.0,
            cull_stats.triangle_ratio() * 100.0,
        );

        // ── Shadow cascade ──────────────────────────────────────────────
        let total_cascade_bias = self.apply_shadow_cascade(
            &mut render_scene, camera, cam_near, cam_far, &config,
        );

        // ── Atmospheric scattering ──────────────────────────────────────
        let rayleigh_blue = rayleigh_scatter(440.0);
        let rayleigh_red = rayleigh_scatter(680.0);
        let scatter_ratio = rayleigh_blue / rayleigh_red.max(1.0);
        render_scene.exposure *= 1.0 + (scatter_ratio - 8.0).abs() * 0.001;

        // ── Cloud density probe ─────────────────────────────────────────
        self.apply_cloud_layer(&mut render_scene, camera);

        // ── Shadow-aware sun occlusion (PCF + contact) ──────────────────
        self.apply_shadow_sampling(&mut render_scene, camera);

        // ── Sphere occlusion test ───────────────────────────────────────
        if render_scene.objects.len() >= 2 {
            let occluded = sphere_occludes(
                &render_scene.objects[0],
                render_scene.objects[1].center,
                render_scene.objects[1].radius,
                camera.origin,
            );
            if occluded {
                render_scene.exposure *= 1.02;
            }
        }

        if is_preview {
            render_scene.volume = render_scene.volume.with_density_multiplier(0.35);
        }

        let pixel_work = config.width * config.height * config.base_samples_per_pixel as usize;
        let max_threads = self.hw_caps.optimal_render_threads_for_input(pixel_work);
        let pixel_threads = pixel_work.div_ceil(4_000).clamp(1, max_threads);
        let complexity_threads = input_scene_objects.div_ceil(20_000).clamp(1, max_threads);
        config.thread_count = pixel_threads.max(complexity_threads);
        crate::runtime_log!(
            "adaptive-threads: {} objects, {}x{} @{}spp → {} threads (max={})",
            input_scene_objects, config.width, config.height,
            config.base_samples_per_pixel, config.thread_count, max_threads,
        );
        let compute_submitted = self.submit_compute_workload(&render_scene, config.width, config.height);

        // ── Ray trace (with precise per-phase timing) ───────────────────
        let t_frame = precise_timestamp_ns();
        let start = HwInstant::now();

        let t_trace = precise_timestamp_ns();
        let (cached_bvh, cache_hit) = self.cached_bvh_for_scene(&render_scene);
        crate::runtime_log!("tracer: BVH cache {}", if cache_hit { "hit" } else { "miss" });
        let (image, bvh_stats) = self
            .tracer
            .render_with_bvh(&render_scene, camera, &config, &self.lod_manager, cached_bvh.as_deref());
        let trace_ms = hw_elapsed_ms(t_trace, precise_timestamp_ns());

        if let Some(sync_ms) = self.gpu_fence_and_sync() {
            crate::runtime_log!("gpu-sync: {:.2}ms", sync_ms);
        }

        // ── FrameBuffer ─────────────────────────────────────────────────
        let t_post = precise_timestamp_ns();
        let mut framebuffer = FrameBuffer::from(image);

        // Depth fog only when the scene has meaningful depth variation
        let (depth_min, depth_max) = framebuffer.depth_range();
        if (depth_max - depth_min) > 1.0 {
            self.apply_depth_fog(&mut framebuffer);
        }

        // ── Post-processing (adaptive to scene complexity) ──────────────
        let pixel_count = config.width * config.height;
        let scene_complexity = render_scene.objects.len() + render_scene.triangles.len();

        // Bloom: threshold/radius/intensity scale with scene luminance
        let avg_luma = framebuffer.average_luminance();
        let bloom_threshold = (1.0 + quality.shadow_quality * 0.2).max(avg_luma * 2.0);
        let bloom_radius = if pixel_count > 2_000_000 { 3 } else { 2 };
        let post = PostProcessor::cinematic()
            .with_bloom_threshold(bloom_threshold)
            .with_bloom_radius(bloom_radius)
            .with_bloom_intensity(if matches!(preset, RenderPreset::ProductionReference) { 0.16 } else { 0.10 })
            .with_grain(0.0)
            .with_aberration(0.0)
            .with_sharpen(if is_preview { 0.0 } else { 0.14 })
            .with_exposure(1.0);
        post.apply(&mut framebuffer);

        // ── God rays (single pass, strength depends on sun visibility) ──
        let sun_dir = (-render_scene.sun.direction).normalize();
        let sun_dot = camera.direction.normalize().dot(sun_dir);
        if sun_dot > 0.0 {
            let sun_screen_x = 0.5 + sun_dot * 0.3;
            let intensity = sun_dot.clamp(0.0, 1.0);
            self.apply_god_rays(&mut framebuffer, sun_screen_x, intensity);
        }

        // ── Depth-of-field (skip if aperture produces no visible blur) ──
        if matches!(preset, RenderPreset::ProductionReference) {
            let focus_dist = scene_radius.max(10.0);
            let dof = DepthOfField::new(focus_dist, 0.0035, 1.0);
            let max_coc = dof.max_coc_for_range(depth_min, depth_max);
            if max_coc >= 1.25 {
                dof.apply(&mut framebuffer);
            }
        }

        // ── Tone mapping & color grading ────────────────────────────────
        self.apply_tone_mapping_and_grading(&mut framebuffer);

        // ── Final bloom pass only if scene has HDR highlights ───────────
        let brightest = framebuffer.brightest_pixel();
        if brightest.length() > 1.2 {
            let bloom_only = PostProcessor::cinematic().with_bloom_threshold(1.2);
            bloom_only.apply_bloom_only(&mut framebuffer);
        }
        let uploaded_to_gpu = self.upload_framebuffer_to_gpu(&framebuffer);
        let post_ms = hw_elapsed_ms(t_post, precise_timestamp_ns());

        if std::env::var_os("ENGINE_RENDER_INLINE_PROBE").is_some() {
            let inline_report = self.render(scene, camera, preset);
            crate::runtime_log!(
                "inline-probe: {}x{} {:.1}ms",
                inline_report.width,
                inline_report.height,
                inline_report.duration_ms,
            );
        }

        // ── Diagnostics ─────────────────────────────────────────────────
        let scene_bounds_min = preprocessed.analysis.scene_bounds_min;
        let scene_bounds_max = preprocessed.analysis.scene_bounds_max;
        let frust_hw = preprocessed.camera_info.frustum_half_width;
        let frust_hh = preprocessed.camera_info.frustum_half_height;
        let point_inside = frustum.contains_point(camera.origin + camera.direction.normalize() * 5.0);
        let aabb_vis = frustum.contains_aabb(scene_bounds_min, scene_bounds_max);
        crate::runtime_log!(
            "frustum: point_inside={} aabb={:?} frust_hw={:.2} frust_hh={:.2}",
            point_inside, aabb_vis, frust_hw, frust_hh
        );

        let ev100 = ev100_from_luminance(avg_luma.max(0.001));
        let exposure = exposure_from_ev100(ev100);
        crate::runtime_log!(
            "exposure: ev100={:.2} exposure={:.4} sorted_objects={} sorted_triangles={} sample_mult={:.2} bounce_lim={} vol_qual={:.2} dominant_light=({:.2},{:.2},{:.2}) avg_obj_r={:.2} total_cascade_bias={:.4}",
            ev100, exposure,
            preprocessed.sorted_object_indices.len(),
            preprocessed.sorted_triangle_indices.len(),
            quality.sample_multiplier, quality.bounce_limit, quality.volumetric_quality,
            preprocessed.analysis.dominant_light_direction.x,
            preprocessed.analysis.dominant_light_direction.y,
            preprocessed.analysis.dominant_light_direction.z,
            preprocessed.analysis.average_object_radius, total_cascade_bias
        );

        // ── Per-phase timing summary ────────────────────────────────────
        let total_frame_ms = hw_elapsed_ms(t_frame, precise_timestamp_ns());
        crate::runtime_log!(
            "pipeline: total={:.1}ms trace={:.1} post={:.1} complexity={} | {} simd={} gpu_fb={}",
            total_frame_ms, trace_ms, post_ms,
            scene_complexity,
            self.gpu_info_tag(),
            self.simd_tag(),
            uploaded_to_gpu,
        );
        crate::runtime_log!("compute: submitted={}", compute_submitted);

        // ── Analysis ────────────────────────────────────────────────────
        let average_luminance = framebuffer.average_luminance();
        let (min_luminance, max_luminance) = framebuffer.luminance_range();
        let brightest_pixel = framebuffer.brightest_pixel();
        crate::runtime_log!(
            "luminance: avg={:.4} range=[{:.4},{:.4}]",
            average_luminance, min_luminance, max_luminance,
        );

        let image = framebuffer.into_image();
        image.save(output_path.as_ref())?;

        let end = HwInstant::now();

        Ok(RenderReport {
            width: image.width,
            height: image.height,
            rendered_pixels: image.width * image.height,
            duration_ms: end.duration_since_ms(&start),
            output_path: output_path.as_ref().to_path_buf(),
            object_count: render_scene.objects.len(),
            triangle_count: render_scene.triangles.len(),
            average_luminance,
            min_luminance,
            max_luminance,
            brightest_pixel,
            estimated_samples_per_pixel: config.base_samples_per_pixel as usize,
            bvh: bvh_stats,
        })
    }

    pub fn render_animation_frame<P: AsRef<Path>>(
        &self,
        scene: &Scene,
        camera: &Camera,
        bvh: Option<&BvhNode>,
        scheduler: &TileScheduler,
        output_path: P,
        preset: RenderPreset,
    ) -> Result<RenderReport, Box<dyn Error>> {
        self.render_animation_frame_with_pressure(
            scene,
            camera,
            bvh,
            scheduler,
            output_path,
            AnimationFramePressure {
                preset,
                sample_pressure_scale: 1.0,
            },
        )
    }

    pub fn render_animation_frame_with_pressure<P: AsRef<Path>>(
        &self,
        scene: &Scene,
        camera: &Camera,
        bvh: Option<&BvhNode>,
        scheduler: &TileScheduler,
        output_path: P,
        pressure: AnimationFramePressure,
    ) -> Result<RenderReport, Box<dyn Error>> {
        let preset = pressure.preset;
        let sample_pressure_scale = pressure.sample_pressure_scale;
        let mut config = self.config_for(preset);
        let is_preview = matches!(preset, RenderPreset::PreviewCpu | RenderPreset::AnimationFast);

        let preprocessed = ScenePreprocessor::analyze(scene, camera);
        let adaptive_budget_ms = match preset {
            RenderPreset::AnimationFast | RenderPreset::PreviewCpu => 16.0,
            RenderPreset::UltraHdCpu => 120.0,
            RenderPreset::ProductionReference => 260.0,
        };
        let quality = crate::core::engine::rendering::preprocessing::scene_preprocessor::AdaptiveQualitySettings::from_analysis(
            &preprocessed.analysis,
            adaptive_budget_ms,
        );
        let minimum_spp = match preset {
            RenderPreset::AnimationFast => 1,
            RenderPreset::PreviewCpu => 2,
            RenderPreset::UltraHdCpu => 8,
            RenderPreset::ProductionReference => 16,
        };
        config.base_samples_per_pixel = ((config.base_samples_per_pixel as f64 * quality.sample_multiplier)
            .round() as u32)
            .max(minimum_spp);
        apply_runtime_sampling_pressure(&mut config, preset, sample_pressure_scale);
        config.max_bounces = config.max_bounces.min(quality.bounce_limit.max(2));
        let cam_near = preprocessed.camera_info.near_plane;
        let cam_far  = preprocessed.camera_info.far_plane;
        let scene_radius = preprocessed.analysis.scene_radius;

        let frustum = self.build_frustum(camera, &config, cam_near, cam_far);

        let contribution_threshold = if is_preview {
            (quality.ao_quality * 0.18).clamp(0.08, 0.20)
        } else {
            (quality.ao_quality * 0.08).clamp(0.02, 0.08)
        };
        let culler = SceneCuller::new(config.max_distance)
            .with_screen_params(60.0_f64.to_radians(), config.height as f64)
            .with_contribution_threshold(contribution_threshold)
            .with_backface_culling(is_preview);

        let (distance_culled, cull_stats) = culler.cull_scene_with_stats(scene, camera);
        let mut render_scene = culler.cull_with_frustum(&distance_culled, &frustum);
        let cull_energy = 1.0 - (cull_stats.sphere_ratio() * 0.02 + cull_stats.triangle_ratio() * 0.01);
        render_scene.exposure *= cull_energy.clamp(0.95, 1.0);

        let total_cascade_bias = self.apply_shadow_cascade(
            &mut render_scene, camera, cam_near, cam_far, &config,
        );

        let rayleigh_blue = rayleigh_scatter(440.0);
        let rayleigh_red  = rayleigh_scatter(680.0);
        let scatter_ratio = rayleigh_blue / rayleigh_red.max(1.0);
        render_scene.exposure *= 1.0 + (scatter_ratio - 8.0).abs() * 0.001;

        self.apply_cloud_layer(&mut render_scene, camera);
        self.apply_shadow_sampling(&mut render_scene, camera);

        if render_scene.objects.len() >= 2 {
            let occluded = sphere_occludes(
                &render_scene.objects[0],
                render_scene.objects[1].center,
                render_scene.objects[1].radius,
                camera.origin,
            );
            if occluded {
                render_scene.exposure *= 1.02;
            }
        }

        if is_preview {
            render_scene.volume = render_scene.volume.with_density_multiplier(0.35);
        }

        let compute_submitted = self.submit_compute_workload(&render_scene, config.width, config.height);

        let t_frame = precise_timestamp_ns();
        let start = HwInstant::now();

        let t_trace = precise_timestamp_ns();
        let reuse_input_bvh = bvh.is_some()
            && render_scene.objects.len() == scene.objects.len()
            && render_scene.triangles.len() == scene.triangles.len();
        let (cached_bvh, cache_hit) = if reuse_input_bvh {
            (None, false)
        } else {
            self.cached_bvh_for_scene(&render_scene)
        };
        if !reuse_input_bvh {
            crate::runtime_log!("tracer: BVH cache {}", if cache_hit { "hit" } else { "miss" });
        }
        let selected_bvh = if reuse_input_bvh { bvh } else { cached_bvh.as_deref() };
        let (image, bvh_stats) = self
            .tracer
            .render_with_scheduler(
                &render_scene,
                camera,
                &config,
                &self.lod_manager,
                selected_bvh,
                scheduler,
            );
        let trace_ms = hw_elapsed_ms(t_trace, precise_timestamp_ns());

        let t_post = precise_timestamp_ns();
        let mut framebuffer = FrameBuffer::from(image);

        let (depth_min, depth_max) = framebuffer.depth_range();
        if (depth_max - depth_min) > 1.0 {
            self.apply_depth_fog(&mut framebuffer);
        }

        let avg_luma = framebuffer.average_luminance();
        let bloom_threshold = (1.0 + quality.shadow_quality * 0.2).max(avg_luma * 2.0);
        let pixel_count = config.width * config.height;
        let bloom_radius = if pixel_count > 2_000_000 { 3 } else { 2 };
        let post = PostProcessor::cinematic()
            .with_bloom_threshold(bloom_threshold)
            .with_bloom_radius(bloom_radius)
            .with_bloom_intensity(if matches!(preset, RenderPreset::ProductionReference) { 0.16 } else { 0.10 })
            .with_grain(0.0)
            .with_aberration(0.0)
            .with_sharpen(if is_preview { 0.0 } else { 0.14 })
            .with_exposure(1.0);
        post.apply(&mut framebuffer);

        let sun_dir = (-render_scene.sun.direction).normalize();
        let sun_dot = camera.direction.normalize().dot(sun_dir);
        if sun_dot > 0.0 {
            let sun_screen_x = 0.5 + sun_dot * 0.3;
            let intensity = sun_dot.clamp(0.0, 1.0);
            self.apply_god_rays(&mut framebuffer, sun_screen_x, intensity);
        }

        if matches!(preset, RenderPreset::ProductionReference) {
            let focus_dist = scene_radius.max(10.0);
            let dof = DepthOfField::new(focus_dist, 0.0035, 1.0);
            let max_coc = dof.max_coc_for_range(depth_min, depth_max);
            if max_coc >= 1.25 {
                dof.apply(&mut framebuffer);
            }
        }

        self.apply_tone_mapping_and_grading(&mut framebuffer);

        let brightest = framebuffer.brightest_pixel();
        if brightest.length() > 1.2 {
            let bloom_only = PostProcessor::cinematic().with_bloom_threshold(1.2);
            bloom_only.apply_bloom_only(&mut framebuffer);
        }
        let uploaded_to_gpu = self.upload_framebuffer_to_gpu(&framebuffer);
        let post_ms = hw_elapsed_ms(t_post, precise_timestamp_ns());

        let total_frame_ms = hw_elapsed_ms(t_frame, precise_timestamp_ns());
        crate::runtime_log!(
            "anim-frame: total={:.1}ms trace={:.1} post={:.1} threads={} cascade_bias={:.4} gpu_fb={}",
            total_frame_ms, trace_ms, post_ms, scheduler.worker_count(), total_cascade_bias, uploaded_to_gpu,
        );
        crate::runtime_log!("compute: submitted={}", compute_submitted);

        let average_luminance = framebuffer.average_luminance();
        let (min_luminance, max_luminance) = framebuffer.luminance_range();
        let brightest_pixel = framebuffer.brightest_pixel();

        let image = framebuffer.into_image();
        image.save(output_path.as_ref())?;

        let end = HwInstant::now();

        Ok(RenderReport {
            width: image.width,
            height: image.height,
            rendered_pixels: image.width * image.height,
            duration_ms: end.duration_since_ms(&start),
            output_path: output_path.as_ref().to_path_buf(),
            object_count: render_scene.objects.len(),
            triangle_count: render_scene.triangles.len(),
            average_luminance,
            min_luminance,
            max_luminance,
            brightest_pixel,
            estimated_samples_per_pixel: config.base_samples_per_pixel as usize,
            bvh: bvh_stats,
        })
    }

    pub fn render_animation_frame_to_buffer(
        &self,
        scene: &Scene,
        camera: &Camera,
        bvh: Option<&BvhNode>,
        scheduler: &TileScheduler,
        preset: RenderPreset,
    ) -> Result<(Vec<crate::core::engine::rendering::raytracing::Vec3>, RenderReport), Box<dyn Error>> {
        self.render_animation_frame_to_buffer_impl(scene, camera, bvh, scheduler, preset, 1.0)
    }

    pub fn render_animation_frame_to_buffer_with_pressure(
        &self,
        scene: &Scene,
        camera: &Camera,
        bvh: Option<&BvhNode>,
        scheduler: &TileScheduler,
        preset: RenderPreset,
        sample_pressure_scale: f64,
    ) -> Result<(Vec<crate::core::engine::rendering::raytracing::Vec3>, RenderReport), Box<dyn Error>> {
        if (sample_pressure_scale - 1.0).abs() <= 0.000_001 {
            return self.render_animation_frame_to_buffer(scene, camera, bvh, scheduler, preset);
        }

        self.render_animation_frame_to_buffer_impl(
            scene,
            camera,
            bvh,
            scheduler,
            preset,
            sample_pressure_scale,
        )
    }

    fn render_animation_frame_to_buffer_impl(
        &self,
        scene: &Scene,
        camera: &Camera,
        bvh: Option<&BvhNode>,
        scheduler: &TileScheduler,
        preset: RenderPreset,
        sample_pressure_scale: f64,
    ) -> Result<(Vec<crate::core::engine::rendering::raytracing::Vec3>, RenderReport), Box<dyn Error>> {
        let mut config = self.config_for(preset);
        let is_preview = matches!(preset, RenderPreset::PreviewCpu | RenderPreset::AnimationFast);
        let start = HwInstant::now();

        apply_runtime_sampling_pressure(&mut config, preset, sample_pressure_scale);

        if matches!(preset, RenderPreset::AnimationFast) {
            let compute_submitted = self.submit_compute_workload(scene, config.width, config.height);
            let cached_bvh = if bvh.is_none() {
                let (cached_bvh, cache_hit) = self.cached_bvh_for_scene(scene);
                crate::runtime_log!("tracer: BVH cache {}", if cache_hit { "hit" } else { "miss" });
                cached_bvh
            } else {
                None
            };
            let (image, bvh_stats) = self
                .tracer
                .render_with_scheduler(scene, camera, &config, &self.lod_manager, bvh.or(cached_bvh.as_deref()), scheduler);
            let framebuffer = FrameBuffer::from(image);
            let _ = self.upload_framebuffer_to_gpu(&framebuffer);
            let average_luminance = framebuffer.average_luminance();
            let (min_luminance, max_luminance) = framebuffer.luminance_range();
            let brightest_pixel = framebuffer.brightest_pixel();
            let pixels = framebuffer.color.clone();
            let w = framebuffer.width;
            let h = framebuffer.height;

            let report = RenderReport {
                width: w,
                height: h,
                rendered_pixels: w * h,
                duration_ms: start.elapsed_ms(),
                output_path: PathBuf::new(),
                object_count: scene.objects.len(),
                triangle_count: scene.triangles.len(),
                average_luminance,
                min_luminance,
                max_luminance,
                brightest_pixel,
                estimated_samples_per_pixel: config.base_samples_per_pixel as usize,
                bvh: bvh_stats,
            };

            crate::runtime_log!("compute: submitted={}", compute_submitted);

            return Ok((pixels, report));
        }

        let preprocessed = ScenePreprocessor::analyze(scene, camera);
        let adaptive_budget_ms = match preset {
            RenderPreset::AnimationFast | RenderPreset::PreviewCpu => 16.0,
            RenderPreset::UltraHdCpu => 120.0,
            RenderPreset::ProductionReference => 260.0,
        };
        let quality = crate::core::engine::rendering::preprocessing::scene_preprocessor::AdaptiveQualitySettings::from_analysis(
            &preprocessed.analysis,
            adaptive_budget_ms,
        );
        let minimum_spp = match preset {
            RenderPreset::AnimationFast => 1,
            RenderPreset::PreviewCpu => 2,
            RenderPreset::UltraHdCpu => 8,
            RenderPreset::ProductionReference => 16,
        };
        config.base_samples_per_pixel = ((config.base_samples_per_pixel as f64 * quality.sample_multiplier)
            .round() as u32)
            .max(minimum_spp);
        apply_runtime_sampling_pressure(&mut config, preset, sample_pressure_scale);
        config.max_bounces = config.max_bounces.min(quality.bounce_limit.max(2));
        let cam_near = preprocessed.camera_info.near_plane;
        let cam_far  = preprocessed.camera_info.far_plane;
        let scene_radius = preprocessed.analysis.scene_radius;

        let frustum = self.build_frustum(camera, &config, cam_near, cam_far);

        let contribution_threshold = if is_preview {
            (quality.ao_quality * 0.18).clamp(0.08, 0.20)
        } else {
            (quality.ao_quality * 0.08).clamp(0.02, 0.08)
        };
        let culler = SceneCuller::new(config.max_distance)
            .with_screen_params(60.0_f64.to_radians(), config.height as f64)
            .with_contribution_threshold(contribution_threshold)
            .with_backface_culling(is_preview);

        let (distance_culled, cull_stats) = culler.cull_scene_with_stats(scene, camera);
        let mut render_scene = culler.cull_with_frustum(&distance_culled, &frustum);
        let cull_energy = 1.0 - (cull_stats.sphere_ratio() * 0.02 + cull_stats.triangle_ratio() * 0.01);
        render_scene.exposure *= cull_energy.clamp(0.95, 1.0);

        let total_cascade_bias = self.apply_shadow_cascade(
            &mut render_scene, camera, cam_near, cam_far, &config,
        );

        let rayleigh_blue = rayleigh_scatter(440.0);
        let rayleigh_red  = rayleigh_scatter(680.0);
        let scatter_ratio = rayleigh_blue / rayleigh_red.max(1.0);
        render_scene.exposure *= 1.0 + (scatter_ratio - 8.0).abs() * 0.001;

        self.apply_cloud_layer(&mut render_scene, camera);
        self.apply_shadow_sampling(&mut render_scene, camera);

        if render_scene.objects.len() >= 2 {
            let occluded = sphere_occludes(
                &render_scene.objects[0],
                render_scene.objects[1].center,
                render_scene.objects[1].radius,
                camera.origin,
            );
            if occluded {
                render_scene.exposure *= 1.02;
            }
        }

        if is_preview {
            render_scene.volume = render_scene.volume.with_density_multiplier(0.35);
        }

        let compute_submitted = self.submit_compute_workload(&render_scene, config.width, config.height);

        let t_frame = precise_timestamp_ns();

        let t_trace = precise_timestamp_ns();
        let (cached_bvh, cache_hit) = self.cached_bvh_for_scene(&render_scene);
        crate::runtime_log!("tracer: BVH cache {}", if cache_hit { "hit" } else { "miss" });
        let (image, bvh_stats) = self
            .tracer
            .render_with_scheduler(
                &render_scene,
                camera,
                &config,
                &self.lod_manager,
                cached_bvh.as_deref(),
                scheduler,
            );
        let trace_ms = hw_elapsed_ms(t_trace, precise_timestamp_ns());

        let t_post = precise_timestamp_ns();
        let mut framebuffer = FrameBuffer::from(image);

        let (depth_min, depth_max) = framebuffer.depth_range();
        if (depth_max - depth_min) > 1.0 {
            self.apply_depth_fog(&mut framebuffer);
        }

        let avg_luma = framebuffer.average_luminance();
        let bloom_threshold = (1.0 + quality.shadow_quality * 0.2).max(avg_luma * 2.0);
        let pixel_count = config.width * config.height;
        let bloom_radius = if pixel_count > 2_000_000 { 3 } else { 2 };
        let post = PostProcessor::cinematic()
            .with_bloom_threshold(bloom_threshold)
            .with_bloom_radius(bloom_radius)
            .with_bloom_intensity(if matches!(preset, RenderPreset::ProductionReference) { 0.16 } else { 0.10 })
            .with_grain(0.0)
            .with_aberration(0.0)
            .with_sharpen(if is_preview { 0.0 } else { 0.14 })
            .with_exposure(1.0);
        post.apply(&mut framebuffer);

        let sun_dir = (-render_scene.sun.direction).normalize();
        let sun_dot = camera.direction.normalize().dot(sun_dir);
        if sun_dot > 0.0 {
            let sun_screen_x = 0.5 + sun_dot * 0.3;
            let intensity = sun_dot.clamp(0.0, 1.0);
            self.apply_god_rays(&mut framebuffer, sun_screen_x, intensity);
        }

        if matches!(preset, RenderPreset::ProductionReference) {
            let focus_dist = scene_radius.max(10.0);
            let dof = DepthOfField::new(focus_dist, 0.0035, 1.0);
            let max_coc = dof.max_coc_for_range(depth_min, depth_max);
            if max_coc >= 1.25 {
                dof.apply(&mut framebuffer);
            }
        }

        self.apply_tone_mapping_and_grading(&mut framebuffer);

        let brightest = framebuffer.brightest_pixel();
        if brightest.length() > 1.2 {
            let bloom_only = PostProcessor::cinematic().with_bloom_threshold(1.2);
            bloom_only.apply_bloom_only(&mut framebuffer);
        }
        let uploaded_to_gpu = self.upload_framebuffer_to_gpu(&framebuffer);
        let post_ms = hw_elapsed_ms(t_post, precise_timestamp_ns());

        let total_frame_ms = hw_elapsed_ms(t_frame, precise_timestamp_ns());
        crate::runtime_log!(
            "anim-frame-buf: total={:.1}ms trace={:.1} post={:.1} threads={} cascade_bias={:.4} gpu_fb={}",
            total_frame_ms, trace_ms, post_ms, scheduler.worker_count(), total_cascade_bias, uploaded_to_gpu,
        );
        crate::runtime_log!("compute: submitted={}", compute_submitted);

        let average_luminance = framebuffer.average_luminance();
        let (min_luminance, max_luminance) = framebuffer.luminance_range();
        let brightest_pixel = framebuffer.brightest_pixel();

        let pixels = framebuffer.color.clone();
        let w = framebuffer.width;
        let h = framebuffer.height;

        let end = HwInstant::now();

        let report = RenderReport {
            width: w,
            height: h,
            rendered_pixels: w * h,
            duration_ms: end.duration_since_ms(&start),
            output_path: PathBuf::new(),
            object_count: render_scene.objects.len(),
            triangle_count: render_scene.triangles.len(),
            average_luminance,
            min_luminance,
            max_luminance,
            brightest_pixel,
            estimated_samples_per_pixel: config.base_samples_per_pixel as usize,
            bvh: bvh_stats,
        };

        Ok((pixels, report))
    }

    pub fn render(&self, scene: &Scene, camera: &Camera, preset: RenderPreset) -> RenderReport {
        self.render_with_pressure(scene, camera, preset, 1.0)
    }

    pub fn render_with_pressure(&self, scene: &Scene, camera: &Camera, preset: RenderPreset, sample_pressure_scale: f64) -> RenderReport {
        let mut config = self.config_for(preset);
        let is_preview = matches!(preset, RenderPreset::PreviewCpu);
        let input_scene_objects = scene.objects.len() + scene.triangles.len();

        let preprocessed = ScenePreprocessor::analyze(scene, camera);
        let adaptive_budget_ms = match preset {
            RenderPreset::AnimationFast | RenderPreset::PreviewCpu => 16.0,
            RenderPreset::UltraHdCpu => 120.0,
            RenderPreset::ProductionReference => 260.0,
        };
        let quality = crate::core::engine::rendering::preprocessing::scene_preprocessor::AdaptiveQualitySettings::from_analysis(&preprocessed.analysis, adaptive_budget_ms);
        let minimum_spp = match preset {
            RenderPreset::AnimationFast => 1,
            RenderPreset::PreviewCpu => 2,
            RenderPreset::UltraHdCpu => 8,
            RenderPreset::ProductionReference => 16,
        };
        config.base_samples_per_pixel = ((config.base_samples_per_pixel as f64 * quality.sample_multiplier)
            .round() as u32)
            .max(minimum_spp);
        apply_runtime_sampling_pressure(&mut config, preset, sample_pressure_scale);
        config.max_bounces = config.max_bounces.min(quality.bounce_limit.max(2));
        let cam_near = preprocessed.camera_info.near_plane;
        let cam_far = preprocessed.camera_info.far_plane;

        let frustum = self.build_frustum(camera, &config, cam_near, cam_far);

        let contribution_threshold = if is_preview {
            (quality.ao_quality * 0.18).clamp(0.08, 0.20)
        } else {
            (quality.ao_quality * 0.08).clamp(0.02, 0.08)
        };
        let culler = SceneCuller::new(config.max_distance)
            .with_screen_params(60.0_f64.to_radians(), config.height as f64)
            .with_contribution_threshold(contribution_threshold)
            .with_backface_culling(is_preview);

        let (distance_culled, stats) = culler.cull_scene_with_stats(scene, camera);
        let mut render_scene = culler.cull_with_frustum(&distance_culled, &frustum);
        crate::runtime_log!(
            "culled {:.0}% spheres, {:.0}% triangles",
            stats.sphere_ratio() * 100.0,
            stats.triangle_ratio() * 100.0,
        );

        let shadow_cascade = ShadowCascade::build_with_camera(
            &render_scene,
            camera,
            cam_near,
            cam_far.min(config.max_distance),
            4,
        );
        render_scene.sun.intensity *= 1.0
            - shadow_cascade.occlusion_estimate * shadow_cascade.shadow_strength * 0.26;
        render_scene.exposure *= 1.0 + shadow_cascade.occlusion_estimate * 0.06;

        let pixel_work = config.width * config.height * config.base_samples_per_pixel as usize;
        let max_threads = self.hw_caps.optimal_render_threads_for_input(pixel_work);
        let pixel_threads = pixel_work.div_ceil(4_000).clamp(1, max_threads);
        let complexity_threads = input_scene_objects.div_ceil(20_000).clamp(1, max_threads);
        config.thread_count = pixel_threads.max(complexity_threads);
        crate::runtime_log!(
            "adaptive-threads: {} objects, {}x{} @{}spp → {} threads (max={})",
            input_scene_objects, config.width, config.height,
            config.base_samples_per_pixel, config.thread_count, max_threads,
        );
        let compute_submitted = self.submit_compute_workload(&render_scene, config.width, config.height);

        let start = HwInstant::now();
        let t_trace = precise_timestamp_ns();
        let (cached_bvh, cache_hit) = self.cached_bvh_for_scene(&render_scene);
        crate::runtime_log!("tracer: BVH cache {}", if cache_hit { "hit" } else { "miss" });
        let (image, bvh_stats) = self
            .tracer
            .render_with_bvh(&render_scene, camera, &config, &self.lod_manager, cached_bvh.as_deref());
        let trace_ms = hw_elapsed_ms(t_trace, precise_timestamp_ns());

        let t_post = precise_timestamp_ns();
        let mut framebuffer = FrameBuffer::from(image);

        let post = PostProcessor::cinematic()
            .with_bloom_threshold(1.0 + quality.shadow_quality * 0.2)
            .with_exposure(1.0);
        post.apply(&mut framebuffer);

        self.apply_tone_mapping_and_grading(&mut framebuffer);

        // Soft highlight compression for the inline render path
        // (the full pipeline uses apply_bloom_only instead).
        let brightest_pre = framebuffer.brightest_pixel();
        if brightest_pre.length() > 1.2 {
            for pixel in &mut framebuffer.color {
                *pixel = reinhard_tonemap(*pixel);
            }
        }
        let uploaded_to_gpu = self.upload_framebuffer_to_gpu(&framebuffer);
        let post_ms = hw_elapsed_ms(t_post, precise_timestamp_ns());
        crate::runtime_log!(
            "render: trace={:.1}ms post={:.1}ms | {} simd={} gpu_fb={}",
            trace_ms, post_ms, self.gpu_info_tag(), self.simd_tag(), uploaded_to_gpu,
        );
        crate::runtime_log!("compute: submitted={}", compute_submitted);

        let average_luminance = framebuffer.average_luminance();
        let (min_luminance, max_luminance) = framebuffer.luminance_range();
        let brightest_pixel = framebuffer.brightest_pixel();
        let image = framebuffer.into_image();

        RenderReport {
            width: image.width,
            height: image.height,
            rendered_pixels: image.width * image.height,
            duration_ms: start.elapsed_ms(),
            output_path: PathBuf::new(),
            object_count: render_scene.objects.len(),
            triangle_count: render_scene.triangles.len(),
            average_luminance,
            min_luminance,
            max_luminance,
            brightest_pixel,
            estimated_samples_per_pixel: config.base_samples_per_pixel as usize,
            bvh: bvh_stats,
        }
    }

}

fn apply_runtime_sampling_pressure(
    config: &mut crate::core::engine::rendering::raytracing::RenderConfig,
    preset: RenderPreset,
    sample_pressure_scale: f64,
) {
    let minimum_spp = match preset {
        RenderPreset::AnimationFast => 1,
        RenderPreset::PreviewCpu => 2,
        RenderPreset::UltraHdCpu => 8,
        RenderPreset::ProductionReference => 16,
    };

    let scaled = ((config.base_samples_per_pixel as f64) * sample_pressure_scale.clamp(0.50, 1.25)).round() as u32;
    config.base_samples_per_pixel = scaled.max(minimum_spp);
}
