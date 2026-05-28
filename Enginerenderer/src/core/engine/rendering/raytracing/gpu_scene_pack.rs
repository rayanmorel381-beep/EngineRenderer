//! Packs a CPU [`Scene`] / [`Camera`] / render config into raw byte blobs
//! ready to be uploaded into SSBOs by the GPU path tracer.
//!
//! Layout follows GLSL `std430` rules.

use super::camera::Camera;
use super::math::Vec3;
use super::primitives::{Material, Sphere, Triangle};
use super::scene::{AreaLight, DirectionalLight, Scene};
use crate::core::engine::rendering::texture::procedural_texture::TextureKind;

const F32_PER_SPHERE: usize = 28;
const F32_PER_TRIANGLE: usize = 40;
const F32_PER_AREA_LIGHT: usize = 16;

/// CPU-side image / sampling configuration consumed by the GPU pipeline.
#[derive(Debug, Clone, Copy)]
pub struct GpuFrameConfig {
    pub width: u32,
    pub height: u32,
    pub samples: u32,
    pub max_bounces: u32,
    pub seed: u32,
    pub exposure: f32,
}

/// Encodes the per-frame uniform-like SSBO.
pub fn pack_frame(camera: &Camera, scene: &Scene, cfg: GpuFrameConfig) -> Vec<u8> {
    let mut floats: Vec<f32> = Vec::with_capacity(40);
    let mut uints: Vec<u32> = Vec::with_capacity(8);

    let (origin, lower_left, horizontal, vertical) = camera.viewport_basis();
    push_vec4(&mut floats, origin, 0.0);
    push_vec4(&mut floats, lower_left, 0.0);
    push_vec4(&mut floats, horizontal, 0.0);
    push_vec4(&mut floats, vertical, 0.0);

    let DirectionalLight {
        direction,
        color,
        intensity,
        ..
    } = scene.sun;
    push_vec4(&mut floats, direction.normalize_or_zero(), intensity as f32);
    push_vec4(&mut floats, color, 0.0);
    let sun_dir_n = direction.normalize_or_zero();
    let (sky_top, sky_bottom) = if let Some(env) = scene.hdri.as_ref() {
        (
            env.hdri_probe(Vec3::new(0.0, 1.0, 0.0), sun_dir_n),
            env.hdri_probe(Vec3::new(0.0, -1.0, 0.0), sun_dir_n),
        )
    } else {
        (scene.sky_top, scene.sky_bottom)
    };
    push_vec4(&mut floats, sky_top, 0.0);
    push_vec4(&mut floats, sky_bottom, 0.0);

    let (mie_g, cloud_coverage, cloud_density, cloud_altitude) = if scene.hdri.is_some() {
        (0.76, 0.45, 0.55, 0.62)
    } else {
        (0.0, 0.0, 0.0, 0.0)
    };
    floats.push(mie_g);
    floats.push(cloud_coverage);
    floats.push(cloud_density);
    floats.push(cloud_altitude);

    floats.push(cfg.exposure.max(1e-3));
    floats.push(0.0);
    floats.push(0.0);
    floats.push(0.0);

    let sphere_count = scene.objects.len() as u32;
    let triangle_count = scene.triangles.len() as u32;
    let area_light_count = scene.area_lights.len() as u32;
    uints.push(cfg.width);
    uints.push(cfg.height);
    uints.push(sphere_count);
    uints.push(triangle_count);
    uints.push(cfg.samples.max(1));
    uints.push(cfg.seed);
    uints.push(cfg.max_bounces);
    uints.push(area_light_count);

    let mut bytes = Vec::with_capacity(floats.len() * 4 + uints.len() * 4);
    for f in &floats {
        bytes.extend_from_slice(&f.to_le_bytes());
    }
    for u in &uints {
        bytes.extend_from_slice(&u.to_le_bytes());
    }
    bytes
}

/// Encodes every sphere primitive into a single std430 SSBO blob.
pub fn pack_spheres(spheres: &[Sphere]) -> Vec<u8> {
    if spheres.is_empty() {
        return placeholder_sphere_blob();
    }
    let mut floats = Vec::with_capacity(spheres.len() * F32_PER_SPHERE);
    for s in spheres {
        push_vec4(&mut floats, s.center, s.radius as f32);
        push_material_full(&mut floats, &s.material);
    }
    floats_to_bytes(&floats)
}

/// Encodes every triangle primitive into a single std430 SSBO blob.
pub fn pack_triangles(triangles: &[Triangle]) -> Vec<u8> {
    if triangles.is_empty() {
        return placeholder_triangle_blob();
    }
    let mut floats = Vec::with_capacity(triangles.len() * F32_PER_TRIANGLE);
    for t in triangles {
        push_vec4(&mut floats, t.a, 0.0);
        push_vec4(&mut floats, t.b, 0.0);
        push_vec4(&mut floats, t.c, 0.0);
        push_material_full(&mut floats, &t.material);
    }
    floats_to_bytes(&floats)
}

/// Encodes every rectangular area light into a single std430 SSBO blob.
pub fn pack_area_lights(lights: &[AreaLight]) -> Vec<u8> {
    if lights.is_empty() {
        return placeholder_area_light_blob();
    }
    let mut floats = Vec::with_capacity(lights.len() * F32_PER_AREA_LIGHT);
    for l in lights {
        push_vec4(&mut floats, l.position, 0.0);
        push_vec4(&mut floats, l.u, 0.0);
        push_vec4(&mut floats, l.v, 0.0);
        push_vec4(&mut floats, l.color, l.intensity as f32);
    }
    floats_to_bytes(&floats)
}

fn placeholder_sphere_blob() -> Vec<u8> {
    let mut floats = vec![0f32; F32_PER_SPHERE];
    floats[3] = -1.0;
    floats_to_bytes(&floats)
}

fn placeholder_triangle_blob() -> Vec<u8> {
    floats_to_bytes(&[0f32; F32_PER_TRIANGLE])
}

fn placeholder_area_light_blob() -> Vec<u8> {
    floats_to_bytes(&[0f32; F32_PER_AREA_LIGHT])
}

fn push_vec4(buf: &mut Vec<f32>, v: Vec3, w: f32) {
    buf.push(v.x as f32);
    buf.push(v.y as f32);
    buf.push(v.z as f32);
    buf.push(w);
}

fn push_material_full(buf: &mut Vec<f32>, mat: &Material) {
    push_vec4(buf, mat.albedo, mat.roughness as f32);
    push_vec4(buf, mat.emission, mat.metallic as f32);
    buf.push(mat.transmission as f32);
    buf.push(mat.ior as f32);
    buf.push(0.0);
    buf.push(0.0);

    if mat.texture_weight > 1e-3 {
        let tex = mat.surface_texture();
        let kind_u: u32 = match tex.kind {
            TextureKind::BrushedMetal => 0,
            TextureKind::RockyMineral => 1,
            TextureKind::FrozenCrystal => 2,
            TextureKind::OceanicBands => 3,
        };
        push_vec4(buf, tex.base_color, tex.scale as f32);
        push_vec4(buf, tex.accent_color, tex.detail_boost as f32);
        buf.push(f32::from_bits(kind_u));
        buf.push(mat.texture_weight as f32);
        buf.push(mat.uv_scale.max(0.05) as f32);
        buf.push(0.0);
    } else {
        push_vec4(buf, Vec3::ZERO, 1.0);
        push_vec4(buf, Vec3::ZERO, 0.0);
        buf.push(f32::from_bits(0));
        buf.push(0.0);
        buf.push(1.0);
        buf.push(0.0);
    }
}

fn floats_to_bytes(floats: &[f32]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(floats.len() * 4);
    for f in floats {
        bytes.extend_from_slice(&f.to_le_bytes());
    }
    bytes
}

trait Vec3NormExt {
    fn normalize_or_zero(self) -> Vec3;
}

impl Vec3NormExt for Vec3 {
    fn normalize_or_zero(self) -> Vec3 {
        let len_sq = self.x * self.x + self.y * self.y + self.z * self.z;
        if len_sq < 1e-30 {
            Vec3::ZERO
        } else {
            self.normalize()
        }
    }
}
