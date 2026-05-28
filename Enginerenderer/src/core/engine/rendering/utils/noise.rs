//! Integer hashing, value noise (2D/3D) and fractal Brownian motion.

use crate::core::engine::rendering::raytracing::Vec3;

use super::interpolation::lerp;

pub fn hash_u32(mut x: u32) -> u32 {
    x = x.wrapping_mul(0x9E3779B9);
    x ^= x >> 16;
    x = x.wrapping_mul(0x85EBCA6B);
    x ^= x >> 13;
    x = x.wrapping_mul(0xC2B2AE35);
    x ^= x >> 16;
    x
}

pub fn hash_to_float(seed: u32) -> f64 {
    (hash_u32(seed) & 0x00FF_FFFF) as f64 / 16_777_215.0
}

pub fn hash_2d(x: i32, y: i32) -> f64 {
    hash_to_float((x as u32).wrapping_mul(1597334673) ^ (y as u32).wrapping_mul(3812015801))
}

pub fn value_noise_2d(x: f64, y: f64) -> f64 {
    let ix = x.floor() as i32;
    let iy = y.floor() as i32;
    let fx = x - x.floor();
    let fy = y - y.floor();
    let ux = fx * fx * (3.0 - 2.0 * fx);
    let uy = fy * fy * (3.0 - 2.0 * fy);

    let c00 = hash_2d(ix, iy);
    let c10 = hash_2d(ix + 1, iy);
    let c01 = hash_2d(ix, iy + 1);
    let c11 = hash_2d(ix + 1, iy + 1);

    lerp(lerp(c00, c10, ux), lerp(c01, c11, ux), uy)
}

pub fn value_noise_3d(point: Vec3) -> f64 {
    let base = value_noise_2d(point.x, point.z);
    let layer = value_noise_2d(point.x + point.y * 0.317, point.z + point.y * 0.719);
    (base + layer) * 0.5
}

pub fn fbm_3d(point: Vec3, octaves: u32, lacunarity: f64, persistence: f64) -> f64 {
    let mut value = 0.0;
    let mut amplitude = 1.0;
    let mut frequency = 1.0;
    let mut max_amplitude = 0.0;

    for _ in 0..octaves {
        value += value_noise_3d(point * frequency) * amplitude;
        max_amplitude += amplitude;
        amplitude *= persistence;
        frequency *= lacunarity;
    }

    if max_amplitude > 0.0 {
        value / max_amplitude
    } else {
        0.0
    }
}
