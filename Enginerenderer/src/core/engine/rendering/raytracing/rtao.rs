use crate::core::engine::rendering::framebuffer::FrameBuffer;
use crate::core::engine::rendering::raytracing::acceleration::BvhNode;
use crate::core::engine::rendering::raytracing::primitives::EPSILON;
use crate::core::engine::rendering::raytracing::shading::{make_seed, random_scalar};
use crate::core::engine::rendering::raytracing::{Ray, Scene, Vec3};

#[derive(Debug, Clone, Copy)]
pub struct RtaoConfig {
    pub samples: u32,
    pub radius: f64,
    pub bias: f64,
    pub power: f64,
    pub falloff: f64,
    pub indirect_bounces: u32,
}

impl Default for RtaoConfig {
    fn default() -> Self {
        Self {
            samples: 8,
            radius: 2.0,
            bias: 0.01,
            power: 1.5,
            falloff: 1.0,
            indirect_bounces: 1,
        }
    }
}

pub struct RtaoPass;

impl RtaoPass {
    pub fn compute(
        fb: &FrameBuffer,
        scene: &Scene,
        normal_fb: &[Vec3],
        world_pos_fb: &[Vec3],
        config: &RtaoConfig,
        bvh: Option<&BvhNode>,
        base_seed: u32,
    ) -> Vec<f64> {
        let w = fb.width;
        let h = fb.height;
        let mut occlusion = vec![0.0_f64; w * h];

        for y in 0..h {
            for x in 0..w {
                let idx = y * w + x;
                let world_pos = world_pos_fb[idx];
                let normal = normal_fb[idx];
                if normal.length_squared() < 0.1 {
                    continue;
                }
                let normal = normal.normalize();

                let mut ao = 0.0_f64;
                let mut valid_samples = 0u32;

                for s in 0..config.samples {
                    let seed = make_seed(
                        make_seed(x as u32, y as u32, base_seed),
                        s,
                        base_seed.wrapping_add(0xCAFE_BABE),
                    );
                    let dir = cosine_weighted_hemisphere(normal, seed);
                    let origin = world_pos + normal * config.bias;
                    let ray = Ray::new(origin, dir);

                    let occluded =
                        BvhNode::hit_scene(scene, &ray, EPSILON, config.radius, bvh).is_some();
                    if occluded {
                        let hit_t = config.radius * 0.5;
                        let falloff =
                            1.0 - (hit_t / config.radius).clamp(0.0, 1.0) * config.falloff;
                        ao += falloff;
                    }
                    valid_samples += 1;
                }

                if valid_samples > 0 {
                    let raw = ao / valid_samples as f64;
                    occlusion[idx] = raw.powf(config.power);
                }
            }
        }

        occlusion
    }

    pub fn apply(fb: &mut FrameBuffer, occlusion: &[f64]) {
        for (idx, pixel) in fb.color.iter_mut().enumerate() {
            let ao = 1.0 - occlusion[idx].clamp(0.0, 1.0);
            *pixel = *pixel * ao;
        }
    }

    pub fn compute_multibounce(
        fb: &FrameBuffer,
        scene: &Scene,
        normal_fb: &[Vec3],
        world_pos_fb: &[Vec3],
        config: &RtaoConfig,
        bvh: Option<&BvhNode>,
        base_seed: u32,
    ) -> Vec<f64> {
        let bounces = config.indirect_bounces.max(1);
        let mut accumulated =
            Self::compute(fb, scene, normal_fb, world_pos_fb, config, bvh, base_seed);

        for bounce in 1..bounces {
            let bounce_seed = base_seed.wrapping_add(bounce.wrapping_mul(0xDEAD_BEEF));
            let w = fb.width;
            let h = fb.height;

            for y in 0..h {
                for x in 0..w {
                    let idx = y * w + x;
                    let world_pos = world_pos_fb[idx];
                    let normal = normal_fb[idx];
                    if normal.length_squared() < 0.1 {
                        continue;
                    }
                    let normal = normal.normalize();

                    let mut indirect_ao = 0.0_f64;
                    let mut valid_samples = 0u32;

                    for s in 0..config.samples {
                        let seed = make_seed(
                            make_seed(x as u32, y as u32, bounce_seed),
                            s,
                            bounce_seed.wrapping_add(0xBEEF_CAFE),
                        );
                        let dir = cosine_weighted_hemisphere(normal, seed);
                        let bounce_origin = world_pos + normal * config.bias;
                        let bounce_ray = Ray::new(bounce_origin, dir);

                        if let Some(hit) =
                            BvhNode::hit_scene(scene, &bounce_ray, EPSILON, config.radius, bvh)
                        {
                            let secondary_origin = bounce_origin + dir * hit.distance;
                            let secondary_normal = hit.normal;
                            for s2 in 0..config.samples / 2 {
                                let seed2 = make_seed(seed, s2, bounce_seed);
                                let dir2 = cosine_weighted_hemisphere(secondary_normal, seed2);
                                let ray2 = Ray::new(
                                    secondary_origin + secondary_normal * config.bias,
                                    dir2,
                                );
                                if BvhNode::hit_scene(scene, &ray2, EPSILON, config.radius, bvh)
                                    .is_some()
                                {
                                    indirect_ao += 0.5 / (bounce as f64 + 1.0);
                                }
                            }
                        }
                        valid_samples += 1;
                    }

                    if valid_samples > 0 {
                        let raw = indirect_ao / valid_samples as f64;
                        accumulated[idx] = (accumulated[idx] + raw * 0.3).min(1.0);
                    }
                }
            }
        }

        accumulated
    }
}

fn cosine_weighted_hemisphere(normal: Vec3, seed: u32) -> Vec3 {
    use std::f64::consts::TAU;
    let u1 = random_scalar(seed);
    let u2 = random_scalar(seed.wrapping_mul(0x9E37_79B9).wrapping_add(1));
    let r = u1.sqrt();
    let phi = TAU * u2;
    let local = Vec3::new(r * phi.cos(), (1.0 - u1).sqrt().max(0.0), r * phi.sin());
    let up = if normal.y.abs() < 0.99 {
        Vec3::new(0.0, 1.0, 0.0)
    } else {
        Vec3::new(1.0, 0.0, 0.0)
    };
    let right = normal.cross(up).normalize();
    let fwd = right.cross(normal).normalize();
    (right * local.x + normal * local.y + fwd * local.z).normalize()
}
