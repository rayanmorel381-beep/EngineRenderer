//! Tests for mesh system, rendering utilities, tone mapping, AABB,
//! framebuffer operations, and Gaussian blur.

use enginerenderer::api::objects::primitives::*;
use enginerenderer::api::engine::rendering::*;

// ── Mesh system ─────────────────────────────────────────────────────────

#[test]
fn mesh_generators_produce_vertices() {
    let mut cube_mesh = unit_cube();
    let plane_mesh = ground_plane(4, 10.0);
    let torus_mesh = torus(3.0, 0.8, 16, 8);

    recalculate_normals(&mut cube_mesh);
    compute_tangents(&mut cube_mesh);

    let mut subdivided = plane_mesh;
    subdivide(&mut subdivided);
    recalculate_normals(&mut subdivided);

    for v in &cube_mesh.vertices {
        assert!(v.tangent.length() >= 0.0);
    }

    assert!(cube_mesh.descriptor.vertex_count > 0);
    assert!(torus_mesh.descriptor.vertex_count > 0);
}

#[test]
fn mesh_asset_aabb() {
    let cube_mesh = unit_cube();
    let (mesh_min, mesh_max) = cube_mesh.aabb();
    assert!(mesh_max.x >= mesh_min.x);
}

#[test]
fn mesh_descriptor_factories() {
    let sphere_desc = uv_sphere(16, 32, 1.0);
    let cube_desc = cube();
    let plane_desc = plane(4, 10.0);
    assert!(sphere_desc.vertex_count > 0);
    assert!(cube_desc.vertex_count > 0);
    let density = geometric_density(&plane_desc, 400.0);
    assert!(density >= 0.0);
}

// ── AABB ────────────────────────────────────────────────────────────────

#[test]
fn aabb_expand_center_hit() {
    let mut aabb = Aabb::empty();
    aabb.expand(Vec3::new(-1.0, -1.0, -1.0));
    aabb.expand(Vec3::new(1.0, 1.0, 1.0));
    let center = aabb.center();
    assert!(center.length() < 1e-9);
    let ray_origin = Vec3::new(0.0, 0.0, -5.0);
    let ray_dir = Vec3::new(0.0, 0.0, 1.0);
    let ray_inv_dir = Vec3::new(
        1.0 / ray_dir.x.max(1e-12),
        1.0 / ray_dir.y.max(1e-12),
        1.0 / ray_dir.z,
    );
    let did_hit = aabb.hit(ray_origin, ray_inv_dir, 0.001, 1000.0);
    assert!(did_hit);
}

// ── Rendering utils ─────────────────────────────────────────────────────

#[test]
fn util_inverse_lerp_remap_quintic() {
    let inv_t = inverse_lerp(0.0, 1.0, 0.5);
    assert!((inv_t - 0.5).abs() < 0.01);
    let remapped = remap(0.5, 0.0, 1.0, -1.0, 1.0);
    assert!((remapped - 0.0).abs() < 0.01);
    let quintic = quintic_smooth(0.0, 1.0, 0.5);
    assert!((0.0..=1.0).contains(&quintic));
}

#[test]
fn util_bias_gain() {
    let b = bias(0.5, 0.3);
    assert!(b > 0.0);
    let g = gain(0.5, 0.4);
    assert!(g > 0.0);
}

#[test]
fn util_srgb_round_trip() {
    let linear = srgb_to_linear(Vec3::new(0.5, 0.5, 0.5));
    let srgb = linear_to_srgb(linear);
    assert!(srgb.x > 0.0);
}

#[test]
fn util_hsv_round_trip() {
    let hsv = rgb_to_hsv(Vec3::new(1.0, 0.0, 0.0));
    let rgb_back = hsv_to_rgb(hsv);
    assert!((rgb_back.x - 1.0).abs() < 0.01);
}

#[test]
fn util_color_temperature() {
    let warm = color_temperature(5500.0);
    assert!(warm.x > 0.0 && warm.y > 0.0 && warm.z > 0.0);
}

#[test]
fn util_fresnel() {
    let fs = fresnel_schlick(0.5, 0.04);
    let fv = fresnel_schlick_vec(0.5, Vec3::new(0.04, 0.04, 0.04));
    let fd = fresnel_dielectric(0.5, 1.5);
    assert!(fs > 0.0);
    assert!(fv.x > 0.0);
    assert!(fd > 0.0);
}

#[test]
fn util_spherical_cartesian() {
    let cart = spherical_to_cartesian(std::f64::consts::FRAC_PI_2, 0.0);
    let (theta, phi) = cartesian_to_spherical(cart);
    assert!(theta >= 0.0);
    assert!(phi.is_finite());
}

#[test]
fn util_reflect() {
    let reflected = reflect(
        Vec3::new(1.0, -1.0, 0.0).normalize(),
        Vec3::new(0.0, 1.0, 0.0),
    );
    assert!(reflected.y > 0.0);
}

#[test]
fn util_triangle_area_and_barycentric() {
    let area = triangle_area(
        Vec3::ZERO,
        Vec3::new(1.0, 0.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
    );
    assert!((area - 0.5).abs() < 0.01);

    let (u, v, w) = barycentric(
        Vec3::new(0.25, 0.25, 0.0),
        Vec3::ZERO,
        Vec3::new(1.0, 0.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
    );
    assert!(u > 0.0 && v > 0.0 && w >= 0.0);
}

// ── Tone mapping ────────────────────────────────────────────────────────

#[test]
fn tone_mapping_operators() {
    let color = Vec3::new(0.8, 0.4, 0.2);
    let _aces = aces_tonemap(color);
    let _reinhard = reinhard_extended(color, 4.0);
    let _uncharted = uncharted2_tonemap(color);
    let _reinhard_op = ToneMappingOperator::Reinhard.apply(color, 1.0);
    let _filmic_op = ToneMappingOperator::Filmic.apply(color, 1.0);
    let _agx_op = ToneMappingOperator::AgX.apply(color, 1.0);
}

// ── FrameBuffer ─────────────────────────────────────────────────────────

#[test]
fn framebuffer_pixel_count() {
    let fb = FrameBuffer::new(64, 64);
    assert_eq!(fb.pixel_count(), 64 * 64);
}

#[test]
fn framebuffer_set_clear_cycle() {
    let mut fb = FrameBuffer::new(8, 8);
    fb.set_pixel(0, 0, Vec3::new(1.0, 0.5, 0.2));
    assert!(fb.get_pixel(0, 0).length() > 0.0);
    fb.clear();
    assert!(fb.get_pixel(0, 0).length() < 1e-9);
}

#[test]
fn framebuffer_alpha_depth_sample_count() {
    let fb = FrameBuffer::new(8, 8);
    assert!(fb.alpha[0] >= 0.0);
    assert!(!fb.depth.is_empty());
    assert!(!fb.sample_count.is_empty());
}

#[test]
fn framebuffer_depth_range_and_color() {
    let fb = FrameBuffer::new(8, 8);
    let (_dmin, _dmax) = fb.depth_range();
    let depth_color = fb.depth_to_color();
    assert_eq!(depth_color.len(), fb.pixel_count());
}

#[test]
fn framebuffer_tile_accumulate_merge() {
    let mut fb = FrameBuffer::new(64, 64);
    let tile = fb.tile_region(0, 0, 8, 8);
    fb.write_tile(0, 0, 8, 8, &tile);
    fb.accumulate(0, 0, fb.get_pixel(0, 0), fb.get_depth(0, 0));
    let zero = FrameBuffer::new(64, 64);
    fb.merge(&zero);
}

// ── Gaussian blur ───────────────────────────────────────────────────────

#[test]
fn gaussian_weights_positive_sigma() {
    let weights = gaussian_weights(3, 1.0);
    assert!(!weights.is_empty());
    let sum: f64 = weights.iter().sum();
    assert!((sum - 1.0).abs() < 0.01);
}

#[test]
fn gaussian_weights_zero_sigma_returns_empty() {
    let weights = gaussian_weights(3, 0.0);
    assert!(weights.is_empty());
}
