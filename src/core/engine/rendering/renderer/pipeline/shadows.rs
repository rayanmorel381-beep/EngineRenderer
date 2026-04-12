//! Shadow cascade, cloud layers, and shadow sampling helpers.

use crate::core::engine::rendering::{
    culling::frustum::Frustum,
    effects::shadow_map::cascade::ShadowCascade,
    effects::shadow_map::sampling::{contact_shadow, pcf_shadow},
    environment::clouds::CloudLayer,
    raytracing::{Camera, RenderConfig, Scene, Vec3},
};

use super::super::Renderer;

impl Renderer {
    pub(in crate::core::engine::rendering::renderer) fn build_frustum(
        &self,
        camera: &Camera,
        config: &RenderConfig,
        near: f64,
        far: f64,
    ) -> Frustum {
        let fov_rad = 60.0_f64.to_radians();
        let aspect = config.width as f64 / config.height as f64;
        Frustum::from_camera(camera, fov_rad, aspect, near, far)
    }

    pub(in crate::core::engine::rendering::renderer) fn apply_shadow_cascade(
        &self,
        render_scene: &mut Scene,
        camera: &Camera,
        cam_near: f64,
        cam_far: f64,
        config: &RenderConfig,
    ) -> f64 {
        let shadow_cascade = ShadowCascade::build_with_camera(
            render_scene,
            camera,
            cam_near,
            cam_far.min(config.max_distance),
            4,
        );

        let mut total_cascade_bias = 0.0;
        for (i, cascade_cfg) in shadow_cascade.cascades.iter().enumerate() {
            total_cascade_bias += cascade_cfg.bias + cascade_cfg.normal_bias;
            eprintln!(
                "cascade[{}]: near={:.2} far={:.2} res={} bias={:.4}",
                i, cascade_cfg.split_near, cascade_cfg.split_far,
                cascade_cfg.resolution, cascade_cfg.bias
            );
            if i < shadow_cascade.light_matrices.len() {
                let lm = &shadow_cascade.light_matrices[i];
                let projected = lm.project(camera.origin);
                let in_bounds = lm.is_in_bounds(projected);
                eprintln!(
                    "  light_matrix[{}]: origin=({:.2},{:.2},{:.2}) half={:.2} in_bounds={} r=({:.2},{:.2},{:.2}) u=({:.2},{:.2},{:.2}) f=({:.2},{:.2},{:.2}) near={:.2} far={:.2}",
                    i, lm.origin.x, lm.origin.y, lm.origin.z, lm.half_extent, in_bounds,
                    lm.right.x, lm.right.y, lm.right.z,
                    lm.up.x, lm.up.y, lm.up.z,
                    lm.forward.x, lm.forward.y, lm.forward.z,
                    lm.near_z, lm.far_z
                );
            }
        }

        let cascade_idx = shadow_cascade.cascade_index_for_depth(cam_far * 0.3);
        let blend = shadow_cascade.cascade_blend_factor(cam_far * 0.3, cascade_idx);

        render_scene.sun.intensity *= 1.0
            - shadow_cascade.occlusion_estimate * shadow_cascade.shadow_strength * 0.26;
        render_scene.exposure *= 1.0 + shadow_cascade.occlusion_estimate * 0.06;
        render_scene.exposure *= 1.0 + blend * shadow_cascade.cascade_blend_width * 0.001;

        total_cascade_bias
    }

    pub(in crate::core::engine::rendering::renderer) fn apply_cloud_layer(
        &self,
        render_scene: &mut Scene,
        camera: &Camera,
    ) {
        let cloud = CloudLayer::cirrus();
        let cumulus = CloudLayer::cumulus();
        let cloud_density = cloud.sample_density(camera.origin + Vec3::new(0.0, cloud.altitude, 0.0));
        let cumulus_density = cumulus.sample_density(camera.origin + Vec3::new(0.0, cumulus.altitude, 0.0));
        let cloud_tint = cloud.cloud_color(
            camera.origin + Vec3::new(0.0, cloud.altitude, 0.0),
            -render_scene.sun.direction.normalize(),
            render_scene.sun.color * render_scene.sun.intensity,
            render_scene.sky_top * 0.15,
        );
        render_scene.sky_top += cloud_tint * (cloud_density + cumulus_density * 0.5) * 0.08;
    }

    pub(in crate::core::engine::rendering::renderer) fn apply_shadow_sampling(
        &self,
        render_scene: &mut Scene,
        camera: &Camera,
    ) {
        let sun_dir = (-render_scene.sun.direction).normalize();
        let sample_point = camera.origin + camera.direction.normalize() * 10.0;
        let pcf_vis = pcf_shadow(
            render_scene,
            sample_point,
            Vec3::new(0.0, 1.0, 0.0),
            sun_dir,
            0.001,
            0.002,
            1,
        );
        let contact_vis = contact_shadow(
            render_scene,
            sample_point,
            Vec3::new(0.0, 1.0, 0.0),
            sun_dir,
            5.0,
            8,
        );
        render_scene.sun.intensity *= 0.95 + pcf_vis * 0.05;
        render_scene.sun.intensity *= 0.97 + contact_vis * 0.03;
    }
}
