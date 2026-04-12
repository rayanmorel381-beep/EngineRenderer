//! Mesh system, utility function, and tone-mapping variant validation.

use crate::core::engine::rendering::{
    mesh::generators::{ground_plane, torus, unit_cube},
    mesh::operations::{compute_tangents, recalculate_normals, subdivide},
    mesh::vertex::{cube, geometric_density, plane, uv_sphere},
    preprocessing::bvh_builder::Aabb,
    preprocessing::tone_mapping::ToneMappingOperator,
    raytracing::Vec3,
    utils::{
        aces_tonemap, barycentric, bias, cartesian_to_spherical, color_temperature,
        fresnel_dielectric, fresnel_schlick, fresnel_schlick_vec, gain, hsv_to_rgb,
        inverse_lerp, linear_to_srgb, quintic_smooth, reflect, reinhard_extended, remap,
        rgb_to_hsv, spherical_to_cartesian, srgb_to_linear, triangle_area, uncharted2_tonemap,
    },
};

use super::super::Renderer;

impl Renderer {
    pub(in crate::core::engine::rendering::renderer) fn validate_mesh_system(&self) {
        let mut cube_mesh = unit_cube();
        let plane_mesh = ground_plane(4, 10.0);
        let torus_mesh = torus(3.0, 0.8, 16, 8);

        recalculate_normals(&mut cube_mesh);
        compute_tangents(&mut cube_mesh);

        // Subdivide a copy so we don't lose the original
        let mut subdivided = plane_mesh;
        subdivide(&mut subdivided);
        recalculate_normals(&mut subdivided);

        // Descriptor factories + geometric_density
        let sphere_desc = uv_sphere(16, 32, 1.0);
        let cube_desc = cube();
        let plane_desc = plane(4, 10.0);
        let density = geometric_density(&plane_desc, 400.0);

        // AABB expand / center / hit
        let mut aabb = Aabb::empty();
        aabb.expand(Vec3::new(-1.0, -1.0, -1.0));
        aabb.expand(Vec3::new(1.0, 1.0, 1.0));
        let center = aabb.center();
        let ray_origin = Vec3::new(0.0, 0.0, -5.0);
        let ray_dir = Vec3::new(0.0, 0.0, 1.0);
        let ray_inv_dir = Vec3::new(1.0 / ray_dir.x.max(1e-12), 1.0 / ray_dir.y.max(1e-12), 1.0 / ray_dir.z);
        let did_hit = aabb.hit(ray_origin, ray_inv_dir, 0.001, 1000.0);

        eprintln!(
            "mesh: cube={}v torus={}v sphere={}v cube_d={}v plane_d={}v density={:.2} subdiv={}v center=({:.2},{:.2},{:.2}) hit={}",
            cube_mesh.positions.len(), torus_mesh.vertices.len(),
            sphere_desc.vertex_count, cube_desc.vertex_count, plane_desc.vertex_count,
            density, subdivided.positions.len(),
            center.x, center.y, center.z, did_hit,
        );
    }

    pub(in crate::core::engine::rendering::renderer) fn validate_utils(&self) {
        let inv_t = inverse_lerp(0.0, 1.0, 0.5);
        let remapped = remap(0.5, 0.0, 1.0, -1.0, 1.0);
        let quintic = quintic_smooth(0.0, 1.0, 0.5);
        let bias_val = bias(0.5, 0.3);
        let gain_val = gain(0.5, 0.4);

        let linear_col = srgb_to_linear(Vec3::new(0.5, 0.5, 0.5));
        let srgb_col = linear_to_srgb(linear_col);

        let hsv = rgb_to_hsv(Vec3::new(1.0, 0.0, 0.0));
        let rgb_back = hsv_to_rgb(hsv);

        let warm = color_temperature(5500.0);

        let fresnel_s = fresnel_schlick(0.5, 0.04);
        let fresnel_v = fresnel_schlick_vec(0.5, Vec3::new(0.04, 0.04, 0.04));
        let fresnel_d = fresnel_dielectric(0.5, 1.5);

        let cart = spherical_to_cartesian(std::f64::consts::FRAC_PI_2, 0.0);
        let (theta, phi) = cartesian_to_spherical(cart);

        let reflected = reflect(Vec3::new(1.0, -1.0, 0.0).normalize(), Vec3::new(0.0, 1.0, 0.0));

        let tri_area = triangle_area(Vec3::ZERO, Vec3::new(1.0, 0.0, 0.0), Vec3::new(0.0, 1.0, 0.0));

        let (u, v, w) = barycentric(Vec3::new(0.25, 0.25, 0.0), Vec3::ZERO, Vec3::new(1.0, 0.0, 0.0), Vec3::new(0.0, 1.0, 0.0));

        eprintln!(
            "utils: inv_t={:.3} remap={:.3} quintic={:.3} bias={:.3} gain={:.3}",
            inv_t, remapped, quintic, bias_val, gain_val,
        );
        eprintln!(
            "color: srgb=({:.3},{:.3},{:.3}) hsv->rgb=({:.3},{:.3},{:.3}) warm=({:.3},{:.3},{:.3})",
            srgb_col.x, srgb_col.y, srgb_col.z,
            rgb_back.x, rgb_back.y, rgb_back.z,
            warm.x, warm.y, warm.z,
        );
        eprintln!(
            "fresnel: schlick={:.4} vec=({:.4},{:.4},{:.4}) dielectric={:.4}",
            fresnel_s, fresnel_v.x, fresnel_v.y, fresnel_v.z, fresnel_d,
        );
        eprintln!(
            "geo: theta={:.3} phi={:.3} reflected=({:.3},{:.3},{:.3}) tri_area={:.4} bary=({:.3},{:.3},{:.3})",
            theta, phi, reflected.x, reflected.y, reflected.z, tri_area, u, v, w,
        );
    }

    pub(in crate::core::engine::rendering::renderer) fn validate_tone_mapping_variants(&self) {
        let test_color = Vec3::new(0.8, 0.4, 0.2);
        let reinhard_map = ToneMappingOperator::Reinhard;
        let filmic_map = ToneMappingOperator::Filmic;
        let agx_map = ToneMappingOperator::AgX;

        let aces_result = aces_tonemap(test_color);
        let reinhard_result = reinhard_extended(test_color, 4.0);
        let uncharted_result = uncharted2_tonemap(test_color);
        let reinhard_applied = reinhard_map.apply(test_color, 1.0);
        let filmic_applied = filmic_map.apply(test_color, 1.0);
        let agx_applied = agx_map.apply(test_color, 1.0);
        eprintln!(
            "tonemap: aces=({:.3},{:.3},{:.3}) reinhard=({:.3},{:.3},{:.3}) uncharted=({:.3},{:.3},{:.3}) r_op=({:.3},{:.3},{:.3}) f_op=({:.3},{:.3},{:.3}) agx=({:.3},{:.3},{:.3})",
            aces_result.x, aces_result.y, aces_result.z,
            reinhard_result.x, reinhard_result.y, reinhard_result.z,
            uncharted_result.x, uncharted_result.y, uncharted_result.z,
            reinhard_applied.x, reinhard_applied.y, reinhard_applied.z,
            filmic_applied.x, filmic_applied.y, filmic_applied.z,
            agx_applied.x, agx_applied.y, agx_applied.z,
        );
    }
}
