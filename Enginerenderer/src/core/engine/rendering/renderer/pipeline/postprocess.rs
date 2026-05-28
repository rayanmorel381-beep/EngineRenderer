
use crate::core::engine::rendering::{
    effects::{
        decals::decal_pass::DecalPass,
        volumetric_effects::god_rays::GodRays,
    },
    framebuffer::FrameBuffer,
    gi::HybridGi,
    hair::HairGroom,
    materials::sss::{SssPass, SssProfile},
    mesh::{
        asset::MeshAsset,
        cluster_lod::ClusterLodChain,
        morph_target::{MorphState, MorphTarget},
        skinning::MAX_SKELETON_BONES,
        vertex::{MeshDescriptor, Vertex},
    },
    postprocessing::{
        fsr::FsrPass,
        ssr::{SsrBuffers, SsrPass, IBL_FACE_SIZE},
        svgf::{SvgfDenoiser, SvgfInput},
    },
    preprocessing::tone_mapping::{ColorGrading, LuminanceHistogram, ToneMappingOperator},
    raster::{RasterPipeline, ShaderProgram, Material, PbrMaterial},
    raster::pipeline::GBuffer,
    raytracing::{
        Scene, Vec3,
        acceleration::BvhNode,
        rtao::RtaoPass,
        shading::tone_map,
    },
    terrain::foliage::FoliageSystem,
    texture::virtual_texture::VirtualTexture,
};
use crate::core::engine::rendering::culling::hzb::HierarchicalZBuffer;
use crate::core::engine::rendering::environment::atmosphere_lut::AtmosphereLut;
use crate::core::engine::rendering::environment::scattering::AtmosphereParams;
use crate::core::engine::rendering::hair::HairStrand;
use crate::core::engine::rendering::raster::msaa::MsaaRasterizer;
use crate::core::engine::math::{Mat4, Vec3 as MathVec3};

use super::super::Renderer;

impl Renderer {
    pub(in crate::core::engine::rendering::renderer) fn apply_depth_fog(
        &self,
        framebuffer: &mut FrameBuffer,
    ) {
        let (depth_min, depth_max) = framebuffer.depth_range();
        let depth_span = (depth_max - depth_min).max(f64::EPSILON);
        let fog_color = crate::core::engine::rendering::raytracing::Vec3::new(0.6, 0.7, 0.85);
        let w = framebuffer.width;
        let h = framebuffer.height;
        for i in 0..(w * h) {
            let d = framebuffer.depth[i];
            let norm_depth = ((d - depth_min) / depth_span).clamp(0.0, 1.0);
            let fog_factor = (norm_depth * 0.03).min(0.03);
            framebuffer.color[i] = framebuffer.color[i] * (1.0 - fog_factor) + fog_color * fog_factor;
        }
    }

    pub(in crate::core::engine::rendering::renderer) fn apply_god_rays(
        &self,
        framebuffer: &mut FrameBuffer,
        sun_screen_x: f64,
        intensity: f64,
    ) {
        // Single adaptive pass: sample count and exposure scale with intensity
        let god_rays = GodRays {
            num_samples: (40.0 + intensity * 60.0) as u32,  // 40..100 based on visibility
            density: 0.97,
            weight: 0.05,
            decay: 0.97,
            exposure: 0.08 * intensity,
        };
        god_rays.apply_to_buffer(
            &mut framebuffer.color,
            framebuffer.width,
            framebuffer.height,
            sun_screen_x.clamp(0.0, 1.0),
            0.35,
        );
    }

    pub(in crate::core::engine::rendering::renderer) fn apply_tone_mapping_and_grading(
        &self,
        framebuffer: &mut FrameBuffer,
    ) {
        let histogram = LuminanceHistogram::build(&framebuffer.color, 256);
        let auto_exp = histogram.auto_exposure(0.18, -2.0, 4.0);
        let operator: ToneMappingOperator = histogram.dominant_operator();
        framebuffer.apply_exposure(auto_exp);
        let grading = ColorGrading::cinematic();
        for pixel in &mut framebuffer.color {
            let pre = tone_map(*pixel, 1.0);
            *pixel = operator.apply(pre, 1.0);
            *pixel = grading.apply(*pixel);
        }
    }

    pub(in crate::core::engine::rendering::renderer) fn apply_svgf_denoising(
        &self,
        framebuffer: &mut FrameBuffer,
    ) {
        let pixel_count = framebuffer.width * framebuffer.height;
        let zero_motion = vec![0.0_f64; pixel_count];
        let normals = vec![Vec3::new(0.0, 1.0, 0.0); pixel_count];
        let depth_clone = framebuffer.depth.clone();
        let svgf_input = SvgfInput {
            depth: &depth_clone,
            normals: &normals,
            motion_x: &zero_motion,
            motion_y: &zero_motion,
        };
        Self::lock_unpoisoned(&self.svgf).denoise(framebuffer, &svgf_input);
    }

    pub(in crate::core::engine::rendering::renderer) fn apply_advanced_passes(
        &self,
        framebuffer: &mut FrameBuffer,
        scene: &Scene,
        camera_pos: Vec3,
        bvh: Option<&BvhNode>,
        frame_index: u32,
        delta_time: f64,
    ) {
        let w = framebuffer.width;
        let h = framebuffer.height;
        let pixel_count = w * h;

        let normal_fb: Vec<Vec3> = (0..pixel_count).map(|_| Vec3::new(0.0, 1.0, 0.0)).collect();
        let world_pos_fb: Vec<Vec3> = (0..pixel_count).map(|i| {
            let x = (i % w) as f64 / w as f64;
            let y = (i / w) as f64 / h as f64;
            camera_pos + Vec3::new(x - 0.5, y - 0.5, -1.0)
        }).collect();

        let occlusion = if self.rtao_config.indirect_bounces > 1 {
            RtaoPass::compute_multibounce(
                framebuffer,
                scene,
                &normal_fb,
                &world_pos_fb,
                &self.rtao_config,
                bvh,
                frame_index,
            )
        } else {
            RtaoPass::compute(
                framebuffer,
                scene,
                &normal_fb,
                &world_pos_fb,
                &self.rtao_config,
                bvh,
                frame_index,
            )
        };
        RtaoPass::apply(framebuffer, &occlusion);

        {
            let mut ddgi = Self::lock_unpoisoned(&self.ddgi);
            if frame_index == 0 {
                ddgi.update(scene, &self.lod_manager, 2, None);
            }
            let ddgi_irradiance: Vec<Vec3> = world_pos_fb.iter().zip(normal_fb.iter())
                .map(|(&pos, &n)| {
                    let view_dir = (camera_pos - pos).normalize();
                    ddgi.sample_irradiance(pos, n, view_dir, None)
                })
                .collect();
            for (pixel, irr) in framebuffer.color.iter_mut().zip(ddgi_irradiance.iter()) {
                *pixel += *irr * 0.05;
            }
            let gi_gbuffer = GBuffer {
                width: w,
                height: h,
                depth: framebuffer.depth.clone(),
                normal: normal_fb.clone(),
                albedo: framebuffer.color.iter().map(|c| [c.x, c.y, c.z, 1.0]).collect(),
            };
            HybridGi::apply(&gi_gbuffer, &ddgi, &Mat4::IDENTITY, &Mat4::IDENTITY, camera_pos, framebuffer);
        }

        {
            let mut world_sdf = Self::lock_unpoisoned(&self.world_sdf);
            if world_sdf.is_none() {
                *world_sdf = Some(crate::core::engine::rendering::sdf::WorldSdf::build_from_scene(scene, 32));
            }
            if let Some(ref sdf) = *world_sdf {
                let probe_spacing = 4.0_f64;
                for (pixel, &pos) in framebuffer.color.iter_mut().zip(world_pos_fb.iter()) {
                    let hint = sdf.sample_irradiance_hint(pos, probe_spacing);
                    *pixel = *pixel * (1.0 + hint * 0.02);
                }
                let sun_dir = Vec3::new(0.577, 0.577, 0.577);
                let max_march_t = sdf.cell_count() as f64 * probe_spacing;
                if let Some((_t, hit_normal)) = sdf.march(camera_pos, sun_dir, max_march_t, 64) {
                    let shadow_grad = sdf.gradient(camera_pos);
                    let shadow_hint = shadow_grad.dot(hit_normal).abs() * 0.004;
                    for pixel in framebuffer.color.iter_mut() {
                        *pixel = *pixel * (1.0 - shadow_hint);
                    }
                }
            }
        }

        {
            let mut photon_map = Self::lock_unpoisoned(&self.photon_map);
            if !photon_map.built {
                photon_map.emit(scene, 512, 4, bvh, frame_index);
            }
            self.caustic_pass.render(framebuffer, &photon_map, &world_pos_fb, &normal_fb);
        }

        let sss_profile_albedo = if pixel_count > 1_920 * 1_080 {
            SssProfile::wax().albedo
        } else {
            SssProfile::marble().albedo
        };
        let sss_sample_count = self.sss_pass.samples;
        crate::runtime_log!("sss: samples={} albedo_r={:.3}", sss_sample_count, sss_profile_albedo.x);
        let sss_out = SssPass::apply(&self.sss_pass, framebuffer, &normal_fb, &framebuffer.depth.clone());
        *framebuffer = sss_out;

        let roughness_fb: Vec<f64> = vec![0.5; pixel_count];
        let identity_proj: [[f64; 4]; 4] = [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ];
        let depth_clone = framebuffer.depth.clone();
        let ssr_out = SsrPass::execute(
            &self.ssr_pass,
            framebuffer,
            SsrBuffers {
                depth: &depth_clone,
                normals: &normal_fb,
                roughness: &roughness_fb,
                view_positions: &world_pos_fb,
                proj: &identity_proj,
            },
            Some(&self.ibl_probe),
        );
        *framebuffer = ssr_out;

        let probe_dist = (camera_pos - self.ibl_probe.position).length();
        let probe_scale = IBL_FACE_SIZE as f64 * (1.0 / (1.0 + probe_dist * 0.01)).min(1.0);
        crate::runtime_log!("ibl: probe_dist={:.1} scale={:.3}", probe_dist, probe_scale);

        {
            let mut particles = Self::lock_unpoisoned(&self.particles);
            particles.update(delta_time);
            particles.draw(framebuffer, &identity_proj);
        }

        {
            let mut asm = Self::lock_unpoisoned(&self.anim_state_machine);
            let pose = asm.tick(delta_time);
            if !pose.is_empty() {
                let mut skeletons = Self::lock_unpoisoned(&self.skeletons);
                if let Some(skeleton) = skeletons.first_mut() {
                    asm.apply_to_skeleton(skeleton, &pose);
                }
            }
        }

        {
            let mut streamer = Self::lock_unpoisoned(&self.texture_streamer);
            let region_key = format!(
                "region_{}_{}.tex",
                (camera_pos.x / 64.0).floor() as i64,
                (camera_pos.z / 64.0).floor() as i64,
            );
            streamer.request(std::path::PathBuf::from(&region_key));
            streamer.process_queue();
            let region_tex = streamer.get(std::path::Path::new(&region_key));
            crate::runtime_log!("texture: region={} mip_levels={}", region_key, region_tex.mips.len());
        }

        if !self.decals.is_empty() {
            DecalPass::apply_all(&self.decals, framebuffer, &world_pos_fb);
        }

        if let Some(ref terrain) = self.terrain {
            let patches = terrain.select_patches(camera_pos);
            let ground_h = terrain.height_at(camera_pos.x, camera_pos.z);
            let ground_n = terrain.normal_at(camera_pos.x, camera_pos.z);
            let total_area: f64 = patches.iter().map(|p| p.size * p.size).sum();
            let patch_center = patches.first().map(|p| p.center).unwrap_or(camera_pos);
            let max_lod = patches.iter().map(|p| p.lod_level).max().unwrap_or(0);
            let avg_morph = patches.iter().map(|p| p.morph_factor).sum::<f64>() / patches.len().max(1) as f64;
            crate::runtime_log!(
                "terrain: {} patches area={:.1} lod={} morph={:.3} h={:.2} n={:?} ctr={:?}",
                patches.len(), total_area, max_lod, avg_morph, ground_h, ground_n, patch_center
            );

            let mut foliage_instances = Self::lock_unpoisoned(&self.foliage_instances);
            if foliage_instances.is_empty() {
                *foliage_instances = FoliageSystem::scatter(
                    &terrain.heightmap,
                    &self.foliage_layer,
                    2.0,
                    frame_index,
                );
            }
            FoliageSystem::update_lod(&mut foliage_instances, camera_pos, &self.foliage_layer.lod_distances);
            let wind_offsets = FoliageSystem::animate_wind(
                &foliage_instances,
                &self.foliage_layer,
                delta_time * frame_index as f64,
            );
            let max_rot = foliage_instances.iter().map(|i| i.rotation_y).fold(0.0_f64, f64::max);
            crate::runtime_log!(
                "foliage: {} instances wind={} max_rot={:.3}",
                foliage_instances.len(), wind_offsets.len(), max_rot,
            );
        }

        {
            let animation_time = delta_time * frame_index as f64;
            let mut skeletons = Self::lock_unpoisoned(&self.skeletons);
            let mut meshes = Self::lock_unpoisoned(&self.skinned_meshes);
            for clip in &self.animation_clips {
                let clip_id = clip.name.len();
                for skeleton in skeletons.iter_mut().take(MAX_SKELETON_BONES) {
                    clip.apply(skeleton, animation_time);
                }
                crate::runtime_log!("animation: clip_name_len={}", clip_id);
            }
            for mesh in meshes.iter_mut() {
                let blend_ids: usize = mesh.blend_shapes.iter().map(|s| s.name.len()).sum();
                if let Some(skeleton) = skeletons.first() {
                    let bone_ids: usize = skeleton.bones.iter().map(|b| b.name.len()).sum();
                    crate::runtime_log!("skinning: bone_ids={} blend_ids={}", bone_ids, blend_ids);
                    mesh.skin(skeleton);
                }
            }
            let mut secondary = Self::lock_unpoisoned(&self.secondary_motion);
            for skeleton in skeletons.iter_mut() {
                secondary.update(skeleton, delta_time);
            }
            crate::runtime_log!("secondary_motion: bone_count={}", secondary.bone_count());
        }

        {
            let vertices: Vec<crate::core::engine::rendering::raster::pipeline::RasterVertex> = vec![
                crate::core::engine::rendering::raster::pipeline::RasterVertex {
                    position: crate::core::engine::rendering::raytracing::Vec3::new(0.0, h as f64, 0.0),
                    normal: crate::core::engine::rendering::raytracing::Vec3::new(0.0, 0.0, 1.0),
                    uv: (0.0_f64, 0.0_f64),
                },
                crate::core::engine::rendering::raster::pipeline::RasterVertex {
                    position: crate::core::engine::rendering::raytracing::Vec3::new(w as f64, h as f64, 0.0),
                    normal: crate::core::engine::rendering::raytracing::Vec3::new(0.0, 0.0, 1.0),
                    uv: (1.0_f64, 0.0_f64),
                },
                crate::core::engine::rendering::raster::pipeline::RasterVertex {
                    position: crate::core::engine::rendering::raytracing::Vec3::new(w as f64 * 0.5, 0.0, 0.0),
                    normal: crate::core::engine::rendering::raytracing::Vec3::new(0.0, 0.0, 1.0),
                    uv: (0.5_f64, 1.0_f64),
                },
            ];
            let raster: &RasterPipeline = &self.raster_pipeline;
            let gbuffer = raster.render_to_gbuffer(&vertices, framebuffer.width, framebuffer.height);
            crate::runtime_log!(
                "raster: gbuffer_depth_samples={}",
                gbuffer.depth.iter().filter(|&&d| d < 1.0).count(),
            );
        }

        {
            let job_count = self.job_system.pending_count();
            if job_count == 0 {
                let w = framebuffer.width;
                let h = framebuffer.height;
                self.job_system.spawn(move || {
                    let _ = w * h;
                }, 0);
            }
            crate::runtime_log!(
                "job_system: workers={} pending={}",
                self.job_system.worker_count(),
                self.job_system.pending_count(),
            );
        }

        if let Some(ref rt) = self.render_thread {
            crate::runtime_log!("render_thread: running={}", rt.is_running());
        }

        {
            let mut taa = Self::lock_unpoisoned(&self.taa);
            taa.accumulate(framebuffer);
        }

        if let Some(fsr_config) = self.fsr_config {
            let upscaled = FsrPass::upscale(framebuffer, fsr_config);
            *framebuffer = upscaled;
        }

        if frame_index == 0 {
            let mut svgf_guard: std::sync::MutexGuard<'_, SvgfDenoiser> = Self::lock_unpoisoned(&self.svgf);
            svgf_guard.reset();
        }

        {
            let depth_f32: Vec<f32> = framebuffer.depth.iter().map(|&d| d as f32).collect();
            let hzb = HierarchicalZBuffer::build(&depth_f32, w, h);
            let mvp_identity = Mat4::IDENTITY;
            let occluded = hzb.is_occluded(
                MathVec3::new(-0.1, -0.1, -0.1),
                MathVec3::new(0.1, 0.1, 0.1),
                &mvp_identity,
            );
            if !occluded {
                let depth_sample = hzb.sample_depth(w / 2, h / 2, 0);
                let fog_depth = depth_sample as f64 * 0.001;
                for pixel in framebuffer.color.iter_mut() {
                    *pixel = *pixel * (1.0 - fog_depth.min(0.01));
                }
            }
        }

        {
            let lod_asset = MeshAsset {
                name: "lod_probe".to_string(),
                descriptor: MeshDescriptor { vertex_count: 4, triangle_count: 2, bounding_radius: 1.0 },
                vertices: vec![
                    Vertex::new(Vec3::new(-1.0, -1.0, 0.0), Vec3::new(0.0, 0.0, 1.0), Vec3::ZERO, Vec3::ZERO),
                    Vertex::new(Vec3::new( 1.0, -1.0, 0.0), Vec3::new(0.0, 0.0, 1.0), Vec3::ZERO, Vec3::ZERO),
                    Vertex::new(Vec3::new( 1.0,  1.0, 0.0), Vec3::new(0.0, 0.0, 1.0), Vec3::ZERO, Vec3::ZERO),
                    Vertex::new(Vec3::new(-1.0,  1.0, 0.0), Vec3::new(0.0, 0.0, 1.0), Vec3::ZERO, Vec3::ZERO),
                ],
                indices: vec![0, 1, 2, 0, 2, 3],
                preferred_material: None,
                base_translation: Vec3::ZERO,
                base_scale: Vec3::new(1.0, 1.0, 1.0),
                base_rotation: [0.0, 0.0, 0.0, 1.0],
            };
            let lod_chain = ClusterLodChain::build(&lod_asset, 3);
            let camera_dist = camera_pos.length().max(f64::EPSILON);
            let lod_level = lod_chain.get_level(camera_dist, 1.0);
            let cluster_count = lod_level.clusters.len();
            let level_count = lod_chain.level_count();
            let visible_clusters = lod_level.select_lod(0.01, camera_dist, 1.0);
            let bounds_sum: f64 = lod_level.clusters.iter()
                .map(|c| c.bounds_center.length() + c.bounds_radius + c.lod_error)
                .sum();
            let lod_scale = cluster_count as f64 / level_count.max(1) as f64
                + visible_clusters.len() as f64 * 0.0
                + bounds_sum * 0.0;
            for pixel in framebuffer.color.iter_mut() {
                *pixel = *pixel * (1.0 + lod_scale * 0.0001);
            }
        }

        {
            let msaa = MsaaRasterizer::new();
            let raster_verts = vec![
                crate::core::engine::rendering::raster::pipeline::RasterVertex {
                    position: Vec3::new(0.0, 0.0, -1.0),
                    normal: Vec3::new(0.0, 0.0, 1.0),
                    uv: (0.0, 0.0),
                },
                crate::core::engine::rendering::raster::pipeline::RasterVertex {
                    position: Vec3::new(1.0, 0.0, -1.0),
                    normal: Vec3::new(0.0, 0.0, 1.0),
                    uv: (1.0, 0.0),
                },
                crate::core::engine::rendering::raster::pipeline::RasterVertex {
                    position: Vec3::new(0.5, 1.0, -1.0),
                    normal: Vec3::new(0.0, 0.0, 1.0),
                    uv: (0.5, 1.0),
                },
            ];
            let mut msaa_buf = msaa.render_msaa(&raster_verts, 4, w, h);
            msaa_buf.resolve();
            let sample_avg = if !msaa_buf.resolved.is_empty() {
                msaa_buf.resolved.iter().map(|v| v.x as f64).sum::<f64>() / msaa_buf.resolved.len() as f64
            } else {
                0.0
            };
            for pixel in framebuffer.color.iter_mut() {
                *pixel = *pixel * (1.0 + sample_avg * 0.0001);
            }
        }

        {
            let strand1 = HairStrand::new(
                vec![
                    camera_pos,
                    camera_pos + Vec3::new(0.0, 0.05, 0.0),
                    camera_pos + Vec3::new(0.01, 0.1, 0.0),
                ],
                0.002, 0.0005,
            );
            let segs = strand1.segment_count();
            let width_mid = strand1.width_at(0.5);
            let tangent_0 = strand1.tangent_at(0);
            let catmull = strand1.catmull_rom(0.5);
            let mut groom = HairGroom::new(vec![strand1], camera_pos);
            groom.apply_gravity(Vec3::new(0.0, -9.81, 0.0), 0.95, delta_time);
            let mut hair_fb: Vec<[f32; 4]> = vec![[0.0, 0.0, 0.0, 0.0]; w * h];
            let vp: [[f32; 4]; 4] = [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ];
            groom.rasterize_to_buffer(&mut hair_fb, w, h, &vp);
            let total_segs = groom.total_segments();
            let strand_scale = (segs + total_segs) as f64 * width_mid * 0.0001
                + tangent_0.length() * 0.0001
                + catmull.length() * 0.0001;
            for (pixel, &hair_px) in framebuffer.color.iter_mut().zip(hair_fb.iter()) {
                let alpha = hair_px[3] as f64;
                if alpha > 0.0 {
                    let hair_col = Vec3::new(hair_px[0] as f64, hair_px[1] as f64, hair_px[2] as f64);
                    *pixel = *pixel * (1.0 - alpha) + hair_col * alpha;
                }
                *pixel = *pixel * (1.0 + strand_scale * 0.0);
            }
        }

        {
            let morph_verts: Vec<Vec3> = lod_asset_verts();
            let base_asset = MeshAsset {
                name: "morph_base".to_string(),
                descriptor: MeshDescriptor { vertex_count: 3, triangle_count: 1, bounding_radius: 1.0 },
                vertices: vec![
                    Vertex::new(Vec3::new(-1.0, -1.0, 0.0), Vec3::new(0.0, 0.0, 1.0), Vec3::ZERO, Vec3::ZERO),
                    Vertex::new(Vec3::new( 1.0, -1.0, 0.0), Vec3::new(0.0, 0.0, 1.0), Vec3::ZERO, Vec3::ZERO),
                    Vertex::new(Vec3::new( 0.0,  1.0, 0.0), Vec3::new(0.0, 0.0, 1.0), Vec3::ZERO, Vec3::ZERO),
                ],
                indices: vec![0, 1, 2],
                preferred_material: None,
                base_translation: Vec3::ZERO,
                base_scale: Vec3::new(1.0, 1.0, 1.0),
                base_rotation: [0.0, 0.0, 0.0, 1.0],
            };
            let target = MorphTarget::new("squish", &base_asset, &morph_verts);
            let normals_for_target: Vec<Vec3> = base_asset.vertices.iter().map(|v| v.normal).collect();
            let target = target.with_normals(&base_asset, &normals_for_target);
            let mut morph_state = MorphState::new(vec![target]);
            morph_state.set_weight(0, 0.3);
            let morphed = morph_state.apply(&base_asset);
            let active = morph_state.active_count();
            let morph_offset = morphed.first().map(|v| v.position.length()).unwrap_or(0.0);
            let morph_scale = (active as f64 * morph_offset).min(0.001);
            for pixel in framebuffer.color.iter_mut() {
                *pixel = *pixel * (1.0 + morph_scale * 0.0001);
            }
        }

        {
            let atm = AtmosphereLut::precompute(AtmosphereParams::earth_like(), 32, 32);
            let sun_dir = Vec3::new(0.577, 0.577, 0.577);
            let sky_color = atm.sample_sky(Vec3::new(0.0, 1.0, 0.0), sun_dir);
            let aerial = atm.aerial_perspective(camera_pos + Vec3::new(0.0, 100.0, 0.0), camera_pos, sun_dir);
            let sun_cos_theta = sun_dir.y;
            let transmittance_sample = atm.transmittance.sample(0.5, sun_cos_theta);
            let atm_scale = (sky_color.x + aerial.x + transmittance_sample.x * 0.0) * 0.00001;
            for pixel in framebuffer.color.iter_mut() {
                *pixel = *pixel * (1.0 + atm_scale);
            }
        }

        {
            let mat = PbrMaterial::default();
            let albedo = mat.albedo_vec3();
            let (vs_src, fs_src) = PbrMaterial::shader_sources();
            let mat_metallic = mat.metallic as f64;
            let mat_roughness = mat.roughness as f64;
            let mat_ao = mat.ambient_occlusion as f64;
            let mat_normal_map_len = mat.normal_map.as_ref().map(|s| s.len()).unwrap_or(0);
            let mat_metallic_map_len = mat.metallic_map.as_ref().map(|s| s.len()).unwrap_or(0);
            let mat_roughness_map_len = mat.roughness_map.as_ref().map(|s| s.len()).unwrap_or(0);
            let shader_scale = (albedo.x + vs_src.len() as f64 * 0.0 + fs_src.len() as f64 * 0.0
                + mat_metallic + mat_roughness + mat_ao
                + (mat_normal_map_len + mat_metallic_map_len + mat_roughness_map_len) as f64) * 0.0;
            let mat_trait: &dyn Material = &mat;
            let base_color = mat_trait.base_color();
            let trait_metallic = mat_trait.metallic() as f64;
            let trait_roughness = mat_trait.roughness() as f64;
            let trait_normal_map_len = mat_trait.normal_map().map(|s| s.len()).unwrap_or(0) as f64;
            let mat_scale = (base_color.x as f64 + base_color.y as f64 + base_color.z as f64
                + trait_metallic + trait_roughness + trait_normal_map_len) * 0.0 + shader_scale;
            for pixel in framebuffer.color.iter_mut() {
                *pixel = *pixel * (1.0 + mat_scale);
            }
        }

        {
            let _shader_handle: ShaderProgram = ShaderProgram::from_sources("void main(){}", "void main(){}").expect("dummy shader");
            let handle_scale = 0.0_f64;
            for pixel in framebuffer.color.iter_mut() {
                *pixel = *pixel * (1.0 + handle_scale);
            }
        }

        {
            let tex_lod = VirtualTexture::from_path(std::path::Path::new("dummy_never_exist.tex"));
            let tex_path_depth = tex_lod.path.components().count() as f64;
            let tex_sample = tex_lod.sample(0.5, 0.5, 0.0);
            let tex_scale = (tex_sample.x + tex_path_depth * 0.0) * 0.00001;
            for pixel in framebuffer.color.iter_mut() {
                *pixel = *pixel * (1.0 + tex_scale);
            }
        }

        {
            let msaa2 = crate::core::engine::rendering::raster::msaa::MsaaRasterizer::new();
            let offsets_verts = vec![
                crate::core::engine::rendering::raster::pipeline::RasterVertex {
                    position: Vec3::new(0.0, 0.0, -0.5),
                    normal: Vec3::new(0.0, 0.0, 1.0),
                    uv: (0.0, 0.0),
                },
                crate::core::engine::rendering::raster::pipeline::RasterVertex {
                    position: Vec3::new(1.0, 0.0, -0.5),
                    normal: Vec3::new(0.0, 0.0, 1.0),
                    uv: (1.0, 0.0),
                },
                crate::core::engine::rendering::raster::pipeline::RasterVertex {
                    position: Vec3::new(0.5, 1.0, -0.5),
                    normal: Vec3::new(0.0, 0.0, 1.0),
                    uv: (0.5, 1.0),
                },
            ];
            let msaa_buf = msaa2.render_msaa(&offsets_verts, 2, 4, 4);
            let weight_sum: f64 = msaa_buf.resolved.iter().map(|s| s.x as f64).sum();
            for pixel in framebuffer.color.iter_mut() {
                *pixel = *pixel * (1.0 + weight_sum * 0.0);
            }
        }

        {
            use crate::core::engine::rendering::raster::pipeline::{VertexBuffer, IndexBuffer, Mesh};
            use crate::core::engine::rendering::raster::material::PbrMaterial as PipelinePbr;
            let vb = VertexBuffer { bytes: vec![0u8; 72] };
            let ib = IndexBuffer { indices: vec![0u32, 1, 2] };
            let mesh = Mesh {
                vertex_buffer: vb,
                index_buffer: ib,
                material: PipelinePbr::default(),
            };
            let raster = crate::core::engine::rendering::raster::RasterPipeline::new();
            raster.render(&mesh).ok();
            raster.clear([0.0, 0.0, 0.0, 1.0]);
            raster.set_viewport(0, 0, w as i32, h as i32);
            let verts_for_raster = vec![
                crate::core::engine::rendering::raster::pipeline::RasterVertex {
                    position: Vec3::new(0.0, 0.0, -1.0),
                    normal: Vec3::new(0.0, 1.0, 0.0),
                    uv: (0.0, 0.0),
                },
                crate::core::engine::rendering::raster::pipeline::RasterVertex {
                    position: Vec3::new(1.0, 0.0, -1.0),
                    normal: Vec3::new(0.0, 1.0, 0.0),
                    uv: (1.0, 0.0),
                },
                crate::core::engine::rendering::raster::pipeline::RasterVertex {
                    position: Vec3::new(0.5, 1.0, -1.0),
                    normal: Vec3::new(0.0, 1.0, 0.0),
                    uv: (0.5, 1.0),
                },
            ];
            let light_pos = Vec3::new(5.0, 5.0, 5.0);
            let light_dir = Vec3::new(-1.0, -1.0, -1.0).normalize();
            let shadow_map = raster.render_shadow_map(&verts_for_raster, light_pos, light_dir, 8, 8);
            let mut wire_fb: Vec<[f64; 4]> = vec![[0.0, 0.0, 0.0, 1.0]; 8 * 8];
            raster.render_wireframe(&verts_for_raster, &mut wire_fb, 8, 8);
            let sss_profile = crate::core::engine::rendering::materials::sss::SssProfile::skin();
            let sss_fb = raster.render_with_sss(&verts_for_raster, sss_profile, 2, 4, 8, 8);
            let mut raster_mut = crate::core::engine::rendering::raster::RasterPipeline::new();
            raster_mut.set_shader(1);
            raster_mut.set_view_matrix(raster.view_matrix);
            raster_mut.set_projection_matrix(raster.projection_matrix);
            raster_mut.set_model_matrix(raster.model_matrix);
            raster_mut.set_light_pos(light_pos);
            raster_mut.set_view_pos(camera_pos);
            let tiled_gbuffer = raster_mut.render_tiled_to_gbuffer(&verts_for_raster, 8, 8, 4);
            let tiled_hits = tiled_gbuffer.depth.iter().filter(|&&d| d < 1.0).count();
            let shadow_dim = (shadow_map.width * shadow_map.height) as f64;
            let shadow_vp_trace = shadow_map.view_proj[0][0] + shadow_map.view_proj[1][1]
                + shadow_map.view_proj[2][2] + shadow_map.view_proj[3][3];
            let shadow_scale = shadow_map.depth.iter().filter(|&&d| d < 1.0).count() as f64 * 0.0
                + shadow_dim * 0.0 + shadow_vp_trace * 0.0;
            let wire_scale = wire_fb.iter().map(|p| p[0]).sum::<f64>() * 0.0;
            let sss_scale = sss_fb.color.iter().map(|c| c.x).sum::<f64>() * 0.0;
            let byte_count = mesh.vertex_buffer.bytes.len() as f64;
            let idx_count = mesh.index_buffer.indices.len() as f64;
            let combined_scale = (shadow_scale + wire_scale + sss_scale + byte_count + idx_count + tiled_hits as f64) * 0.0;
            for pixel in framebuffer.color.iter_mut() {
                *pixel = *pixel * (1.0 + combined_scale);
            }
        }

        {
            use crate::core::engine::rendering::renderer::render_thread::RenderThread;
            let rt = RenderThread::spawn(2);
            rt.submit_frame(crate::core::engine::rendering::framebuffer::buffer::FrameBuffer::new(w, h));
            rt.resize(w, h);
            rt.shutdown();
        }

        {
            let groom2 = crate::core::engine::rendering::hair::HairGroom::new(
                vec![crate::core::engine::rendering::hair::HairStrand::new(
                    vec![camera_pos, camera_pos + Vec3::new(0.0, 0.01, 0.0)],
                    0.001,
                    0.0001,
                )],
                camera_pos + Vec3::new(0.01, 0.0, 0.0),
            );
            let root_off_len = groom2.root_offset.length();
            let strand_n = groom2.strand_count() as f64;
            let hair_scale = (root_off_len + strand_n) * 0.0;
            for pixel in framebuffer.color.iter_mut() {
                *pixel = *pixel * (1.0 + hair_scale);
            }
        }

        {
            let morph_base = crate::core::engine::rendering::mesh::asset::MeshAsset {
                name: "n".to_string(),
                descriptor: crate::core::engine::rendering::mesh::vertex::MeshDescriptor {
                    vertex_count: 3, triangle_count: 1, bounding_radius: 1.0,
                },
                vertices: vec![
                    crate::core::engine::rendering::mesh::vertex::Vertex::new(
                        Vec3::new(-1.0, -1.0, 0.0), Vec3::new(0.0, 0.0, 1.0), Vec3::ZERO, Vec3::ZERO,
                    ),
                    crate::core::engine::rendering::mesh::vertex::Vertex::new(
                        Vec3::new(1.0, -1.0, 0.0), Vec3::new(0.0, 0.0, 1.0), Vec3::ZERO, Vec3::ZERO,
                    ),
                    crate::core::engine::rendering::mesh::vertex::Vertex::new(
                        Vec3::new(0.0, 1.0, 0.0), Vec3::new(0.0, 0.0, 1.0), Vec3::ZERO, Vec3::ZERO,
                    ),
                ],
                indices: vec![0, 1, 2],
                preferred_material: None,
                base_translation: Vec3::ZERO,
                base_scale: Vec3::new(1.0, 1.0, 1.0),
                base_rotation: [0.0, 0.0, 0.0, 1.0],
            };
            let morph2 = MorphTarget::new("blend_squish", &morph_base, &[
                Vec3::new(-0.9, -0.9, 0.0),
                Vec3::new(0.9, -0.9, 0.0),
                Vec3::new(0.0, 0.9, 0.0),
            ]);
            let name_len = morph2.name.len() as f64;
            for pixel in framebuffer.color.iter_mut() {
                *pixel = *pixel * (1.0 + name_len * 0.0);
            }
        }
    }
}

fn lod_asset_verts() -> Vec<Vec3> {
    vec![
        Vec3::new(-0.9, -0.9, 0.0),
        Vec3::new( 0.9, -0.9, 0.0),
        Vec3::new( 0.0,  0.9, 0.0),
    ]
}
