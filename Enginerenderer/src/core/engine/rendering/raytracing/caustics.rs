use crate::core::engine::rendering::framebuffer::FrameBuffer;
use crate::core::engine::rendering::raytracing::acceleration::BvhNode;
use crate::core::engine::rendering::raytracing::primitives::EPSILON;
use crate::core::engine::rendering::raytracing::shading::{make_seed, random_scalar};
use crate::core::engine::rendering::raytracing::{Ray, Scene, Vec3};

#[derive(Debug, Clone, Copy)]
pub struct Photon {
    pub position: Vec3,
    pub direction: Vec3,
    pub power: Vec3,
    pub normal: Vec3,
    pub bounce: u32,
}

#[derive(Debug, Clone)]
pub struct PhotonMap {
    pub photons: Vec<Photon>,
    pub kd_indices: Vec<usize>,
    pub built: bool,
}

impl PhotonMap {
    pub fn new() -> Self {
        Self {
            photons: Vec::new(),
            kd_indices: Vec::new(),
            built: false,
        }
    }

    pub fn emit(
        &mut self,
        scene: &Scene,
        photon_count: u32,
        max_bounces: u32,
        bvh: Option<&BvhNode>,
        seed: u32,
    ) {
        self.photons.clear();
        self.built = false;
        let mut rng = seed;

        for light in &scene.area_lights {
            let photons_per_light = photon_count / scene.area_lights.len().max(1) as u32;
            let base_power = light.color * (light.intensity / photons_per_light as f64);

            for i in 0..photons_per_light {
                rng = make_seed(rng, i, seed);
                let u = random_scalar(rng);
                rng = rng.wrapping_mul(0x9E37_79B9);
                let v = random_scalar(rng);
                rng = rng.wrapping_add(0x6C62_272E);
                let origin = light.sample_point(u, v);
                let normal = light.u.cross(light.v).normalize();
                let dir = cosine_hemisphere(normal, rng);
                let mut power = base_power;
                let mut ray = Ray::new(origin + normal * EPSILON, dir);

                for bounce in 0..max_bounces {
                    match BvhNode::hit_scene(scene, &ray, EPSILON, f64::MAX, bvh) {
                        None => break,
                        Some(hit) => {
                            let russian = random_scalar(rng.wrapping_mul(0x1234_5678));
                            let albedo_lum = luminance(hit.material.albedo);
                            if russian > albedo_lum || albedo_lum < 0.01 {
                                break;
                            }
                            power = power * hit.material.albedo * (1.0 / albedo_lum.max(1e-6));
                            self.photons.push(Photon {
                                position: hit.point,
                                direction: ray.direction,
                                power,
                                normal: hit.normal,
                                bounce,
                            });
                            rng = rng.wrapping_mul(0x9E37_79B9).wrapping_add(bounce);
                            let new_dir = diffuse_bounce(hit.normal, rng);
                            ray = Ray::new(hit.point + hit.normal * EPSILON, new_dir);
                        }
                    }
                }
            }
        }

        self.build_kd_tree();
    }

    fn build_kd_tree(&mut self) {
        let n = self.photons.len();
        self.kd_indices = (0..n).collect();
        if n == 0 {
            self.built = true;
            return;
        }
        kd_sort(&mut self.kd_indices, &self.photons, 0, n, 0);
        self.built = true;
    }

    pub fn gather(&self, pos: Vec3, normal: Vec3, radius: f64) -> Vec3 {
        if !self.built || self.photons.is_empty() {
            return Vec3::ZERO;
        }
        let radius_sq = radius * radius;
        let mut result = Vec3::ZERO;
        let mut count = 0u32;
        kd_gather(
            &KdQuery {
                indices: &self.kd_indices,
                photons: &self.photons,
                pos,
                normal,
                radius_sq,
            },
            &mut result,
            &mut count,
            0,
        );
        if count > 0 {
            result * (1.0 / (std::f64::consts::PI * radius_sq))
        } else {
            Vec3::ZERO
        }
    }
}

impl Default for PhotonMap {
    fn default() -> Self {
        Self::new()
    }
}

fn kd_sort(indices: &mut [usize], photons: &[Photon], start: usize, end: usize, depth: usize) {
    if end - start <= 1 {
        return;
    }
    let axis = depth % 3;
    let mid = (start + end) / 2;
    let range = &mut indices[start..end];
    range.sort_unstable_by(|&a, &b| {
        let pa = axis_val(photons[a].position, axis);
        let pb = axis_val(photons[b].position, axis);
        pa.partial_cmp(&pb).unwrap_or(std::cmp::Ordering::Equal)
    });
    kd_sort(indices, photons, start, mid, depth + 1);
    kd_sort(indices, photons, mid + 1, end, depth + 1);
}

struct KdQuery<'a> {
    indices: &'a [usize],
    photons: &'a [Photon],
    pos: Vec3,
    normal: Vec3,
    radius_sq: f64,
}

fn kd_gather(q: &KdQuery<'_>, result: &mut Vec3, count: &mut u32, depth: usize) {
    if q.indices.is_empty() {
        return;
    }
    let mid = q.indices.len() / 2;
    let photon = &q.photons[q.indices[mid]];
    let diff = photon.position - q.pos;
    let dist_sq = diff.dot(diff);
    if dist_sq < q.radius_sq {
        let cos_theta = q.normal.dot(-photon.direction).max(0.0);
        let surface_align = q.normal.dot(photon.normal).max(0.0);
        if cos_theta > 0.0 && surface_align > 0.01 {
            let bounce_falloff = 1.0 / (1 + photon.bounce) as f64;
            let gaussian = (-dist_sq / (2.0 * q.radius_sq)).exp();
            *result += photon.power * (cos_theta * surface_align * gaussian * bounce_falloff);
            *count += 1;
        }
    }
    let axis = depth % 3;
    let split = axis_val(photon.position, axis);
    let query = axis_val(q.pos, axis);
    let (near, far) = if query < split {
        (&q.indices[..mid], &q.indices[mid + 1..])
    } else {
        (&q.indices[mid + 1..], &q.indices[..mid])
    };
    kd_gather(
        &KdQuery {
            indices: near,
            photons: q.photons,
            pos: q.pos,
            normal: q.normal,
            radius_sq: q.radius_sq,
        },
        result,
        count,
        depth + 1,
    );
    let axial_dist = (query - split) * (query - split);
    if axial_dist < q.radius_sq {
        kd_gather(
            &KdQuery {
                indices: far,
                photons: q.photons,
                pos: q.pos,
                normal: q.normal,
                radius_sq: q.radius_sq,
            },
            result,
            count,
            depth + 1,
        );
    }
}

fn axis_val(v: Vec3, axis: usize) -> f64 {
    match axis {
        0 => v.x,
        1 => v.y,
        _ => v.z,
    }
}

fn luminance(c: Vec3) -> f64 {
    c.x * 0.2126 + c.y * 0.7152 + c.z * 0.0722
}

fn cosine_hemisphere(normal: Vec3, seed: u32) -> Vec3 {
    use std::f64::consts::TAU;
    let u1 = random_scalar(seed);
    let u2 = random_scalar(seed.wrapping_mul(0x9E37_79B9));
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

fn diffuse_bounce(normal: Vec3, seed: u32) -> Vec3 {
    cosine_hemisphere(normal, seed)
}

pub struct CausticPass {
    pub gather_radius: f64,
    pub visualize_indirect: bool,
}

impl CausticPass {
    pub fn new(gather_radius: f64) -> Self {
        Self {
            gather_radius,
            visualize_indirect: false,
        }
    }

    pub fn render(
        &self,
        fb: &mut FrameBuffer,
        photon_map: &PhotonMap,
        world_pos_fb: &[Vec3],
        normal_fb: &[Vec3],
    ) {
        let w = fb.width;
        let h = fb.height;
        for y in 0..h {
            for x in 0..w {
                let idx = y * w + x;
                let pos = world_pos_fb[idx];
                let normal = normal_fb[idx];
                if normal.length_squared() < 0.01 {
                    continue;
                }
                let caustic = photon_map.gather(pos, normal.normalize(), self.gather_radius);
                let scale = if self.visualize_indirect { 3.0 } else { 1.0 };
                fb.color[idx] += caustic * scale;
            }
        }
    }
}
