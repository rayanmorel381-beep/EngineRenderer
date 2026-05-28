use crate::core::engine::rendering::lod::manager::LodManager;
use crate::core::engine::rendering::raytracing::shading::{TraceContext, make_seed, trace_ray};
use crate::core::engine::rendering::raytracing::{Ray, Scene, Vec3};

pub const DDGI_IRRADIANCE_TEXELS: usize = 8;
pub const DDGI_VISIBILITY_TEXELS: usize = 16;
pub const DDGI_RAYS_PER_PROBE: usize = 64;

#[derive(Debug, Clone, Copy)]
pub struct ProbeGrid {
    pub origin: Vec3,
    pub spacing: Vec3,
    pub counts: [u32; 3],
}

impl ProbeGrid {
    pub fn new(origin: Vec3, spacing: Vec3, counts: [u32; 3]) -> Self {
        Self {
            origin,
            spacing,
            counts,
        }
    }

    pub fn probe_count(&self) -> usize {
        (self.counts[0] * self.counts[1] * self.counts[2]) as usize
    }

    pub fn probe_position(&self, index: usize) -> Vec3 {
        let nx = self.counts[0] as usize;
        let ny = self.counts[1] as usize;
        let x = index % nx;
        let y = (index / nx) % ny;
        let z = index / (nx * ny);
        self.origin
            + Vec3::new(
                x as f64 * self.spacing.x,
                y as f64 * self.spacing.y,
                z as f64 * self.spacing.z,
            )
    }

    pub fn find_enclosing_probes(&self, pos: Vec3) -> [usize; 8] {
        let local = pos - self.origin;
        let fx = (local.x / self.spacing.x).max(0.0);
        let fy = (local.y / self.spacing.y).max(0.0);
        let fz = (local.z / self.spacing.z).max(0.0);
        let ix = (fx as usize).min(self.counts[0] as usize - 2);
        let iy = (fy as usize).min(self.counts[1] as usize - 2);
        let iz = (fz as usize).min(self.counts[2] as usize - 2);
        let nx = self.counts[0] as usize;
        let ny = self.counts[1] as usize;
        [
            ix + iy * nx + iz * nx * ny,
            (ix + 1) + iy * nx + iz * nx * ny,
            ix + (iy + 1) * nx + iz * nx * ny,
            (ix + 1) + (iy + 1) * nx + iz * nx * ny,
            ix + iy * nx + (iz + 1) * nx * ny,
            (ix + 1) + iy * nx + (iz + 1) * nx * ny,
            ix + (iy + 1) * nx + (iz + 1) * nx * ny,
            (ix + 1) + (iy + 1) * nx + (iz + 1) * nx * ny,
        ]
    }

    pub fn trilinear_weights(&self, pos: Vec3) -> [f64; 8] {
        let local = pos - self.origin;
        let tx = (local.x / self.spacing.x).fract().clamp(0.0, 1.0);
        let ty = (local.y / self.spacing.y).fract().clamp(0.0, 1.0);
        let tz = (local.z / self.spacing.z).fract().clamp(0.0, 1.0);
        [
            (1.0 - tx) * (1.0 - ty) * (1.0 - tz),
            tx * (1.0 - ty) * (1.0 - tz),
            (1.0 - tx) * ty * (1.0 - tz),
            tx * ty * (1.0 - tz),
            (1.0 - tx) * (1.0 - ty) * tz,
            tx * (1.0 - ty) * tz,
            (1.0 - tx) * ty * tz,
            tx * ty * tz,
        ]
    }
}

#[derive(Debug, Clone)]
pub struct IrradianceProbe {
    pub irradiance: [[Vec3; DDGI_IRRADIANCE_TEXELS]; DDGI_IRRADIANCE_TEXELS],
    pub mean_distance: [[f64; DDGI_VISIBILITY_TEXELS]; DDGI_VISIBILITY_TEXELS],
    pub mean_distance_sq: [[f64; DDGI_VISIBILITY_TEXELS]; DDGI_VISIBILITY_TEXELS],
}

impl Default for IrradianceProbe {
    fn default() -> Self {
        Self {
            irradiance: [[Vec3::ZERO; DDGI_IRRADIANCE_TEXELS]; DDGI_IRRADIANCE_TEXELS],
            mean_distance: [[0.0; DDGI_VISIBILITY_TEXELS]; DDGI_VISIBILITY_TEXELS],
            mean_distance_sq: [[0.0; DDGI_VISIBILITY_TEXELS]; DDGI_VISIBILITY_TEXELS],
        }
    }
}

impl IrradianceProbe {
    pub fn sample_irradiance(&self, direction: Vec3) -> Vec3 {
        let (u, v) = oct_encode(direction.normalize());
        let tx = ((u * 0.5 + 0.5) * (DDGI_IRRADIANCE_TEXELS - 1) as f64)
            .clamp(0.0, (DDGI_IRRADIANCE_TEXELS - 1) as f64);
        let ty = ((v * 0.5 + 0.5) * (DDGI_IRRADIANCE_TEXELS - 1) as f64)
            .clamp(0.0, (DDGI_IRRADIANCE_TEXELS - 1) as f64);
        let base = bilinear_irradiance(&self.irradiance, tx, ty);
        let vx = (tx * (DDGI_VISIBILITY_TEXELS - 1) as f64
            / (DDGI_IRRADIANCE_TEXELS - 1).max(1) as f64)
            .clamp(0.0, (DDGI_VISIBILITY_TEXELS - 1) as f64) as usize;
        let vy = (ty * (DDGI_VISIBILITY_TEXELS - 1) as f64
            / (DDGI_IRRADIANCE_TEXELS - 1).max(1) as f64)
            .clamp(0.0, (DDGI_VISIBILITY_TEXELS - 1) as f64) as usize;
        let mean_d = self.mean_distance[vy][vx];
        let mean_d_sq = self.mean_distance_sq[vy][vx];
        let variance = (mean_d_sq - mean_d * mean_d).max(0.0);
        let visibility = if variance < 1e-6 {
            1.0
        } else {
            (variance / (variance + mean_d * mean_d * 0.02)).clamp(0.1, 1.0)
        };
        base * visibility
    }

    pub fn update_from_rays(&mut self, rays: &[ProbeRaySample], hysteresis: f64) {
        let mut acc = [[Vec3::ZERO; DDGI_IRRADIANCE_TEXELS]; DDGI_IRRADIANCE_TEXELS];
        let mut weights = [[0.0_f64; DDGI_IRRADIANCE_TEXELS]; DDGI_IRRADIANCE_TEXELS];
        let mut acc_dist = [[0.0_f64; DDGI_VISIBILITY_TEXELS]; DDGI_VISIBILITY_TEXELS];
        let mut acc_dist_sq = [[0.0_f64; DDGI_VISIBILITY_TEXELS]; DDGI_VISIBILITY_TEXELS];
        let mut vis_counts = [[0.0_f64; DDGI_VISIBILITY_TEXELS]; DDGI_VISIBILITY_TEXELS];

        for ray in rays {
            let (u, v) = oct_encode(ray.direction);
            let tx = ((u * 0.5 + 0.5) * (DDGI_IRRADIANCE_TEXELS - 1) as f64)
                .clamp(0.0, (DDGI_IRRADIANCE_TEXELS - 1) as f64);
            let ty = ((v * 0.5 + 0.5) * (DDGI_IRRADIANCE_TEXELS - 1) as f64)
                .clamp(0.0, (DDGI_IRRADIANCE_TEXELS - 1) as f64);
            let ix = tx as usize;
            let iy = ty as usize;
            let w = ray.direction.dot(ray.direction).max(0.0001);
            acc[iy][ix] += ray.radiance * w;
            weights[iy][ix] += w;
            let vx = (tx * (DDGI_VISIBILITY_TEXELS - 1) as f64
                / (DDGI_IRRADIANCE_TEXELS - 1) as f64) as usize;
            let vy = (ty * (DDGI_VISIBILITY_TEXELS - 1) as f64
                / (DDGI_IRRADIANCE_TEXELS - 1) as f64) as usize;
            let vx = vx.min(DDGI_VISIBILITY_TEXELS - 1);
            let vy = vy.min(DDGI_VISIBILITY_TEXELS - 1);
            acc_dist[vy][vx] += ray.hit_distance;
            acc_dist_sq[vy][vx] += ray.hit_distance * ray.hit_distance;
            vis_counts[vy][vx] += 1.0;
        }

        for y in 0..DDGI_IRRADIANCE_TEXELS {
            for x in 0..DDGI_IRRADIANCE_TEXELS {
                if weights[y][x] > 1e-6 {
                    let new_val = acc[y][x] * (1.0 / weights[y][x]);
                    self.irradiance[y][x] =
                        self.irradiance[y][x] * hysteresis + new_val * (1.0 - hysteresis);
                }
            }
        }

        for y in 0..DDGI_VISIBILITY_TEXELS {
            for x in 0..DDGI_VISIBILITY_TEXELS {
                if vis_counts[y][x] > 0.0 {
                    let mean = acc_dist[y][x] / vis_counts[y][x];
                    let mean_sq = acc_dist_sq[y][x] / vis_counts[y][x];
                    self.mean_distance[y][x] =
                        self.mean_distance[y][x] * hysteresis + mean * (1.0 - hysteresis);
                    self.mean_distance_sq[y][x] =
                        self.mean_distance_sq[y][x] * hysteresis + mean_sq * (1.0 - hysteresis);
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ProbeRaySample {
    pub direction: Vec3,
    pub radiance: Vec3,
    pub hit_distance: f64,
}

pub struct DdgiVolume {
    pub grid: ProbeGrid,
    pub probes: Vec<IrradianceProbe>,
    pub hysteresis: f64,
    pub normal_bias: f64,
    pub view_bias: f64,
    pub frame: u64,
}

impl DdgiVolume {
    pub fn new(grid: ProbeGrid) -> Self {
        let n = grid.probe_count();
        Self {
            probes: (0..n).map(|_| IrradianceProbe::default()).collect(),
            grid,
            hysteresis: 0.97,
            normal_bias: 0.25,
            view_bias: 0.01,
            frame: 0,
        }
    }

    pub fn update(
        &mut self,
        scene: &Scene,
        lod: &LodManager,
        max_bounces: u32,
        sdf: Option<&crate::core::engine::rendering::sdf::world_sdf::WorldSdf>,
    ) {
        let probe_count = self.grid.probe_count();
        for probe_idx in 0..probe_count {
            let probe_pos = self.grid.probe_position(probe_idx);
            let mut ray_samples = Vec::with_capacity(DDGI_RAYS_PER_PROBE);

            for ray_idx in 0..DDGI_RAYS_PER_PROBE {
                let seed = make_seed(probe_idx as u32, ray_idx as u32, self.frame as u32);
                let dir = fibonacci_sphere(ray_idx, DDGI_RAYS_PER_PROBE, seed);
                let ray = Ray::new(probe_pos + dir * 0.001, dir);
                let ctx = TraceContext {
                    scene,
                    lod_manager: lod,
                    global_bounce_limit: max_bounces,
                    seed,
                    bvh: None,
                    sdf,
                };
                let radiance = trace_ray(ray, 0, ctx);
                ray_samples.push(ProbeRaySample {
                    direction: dir,
                    radiance,
                    hit_distance: 0.0,
                });
            }

            self.probes[probe_idx].update_from_rays(&ray_samples, self.hysteresis);
        }
        self.frame += 1;
    }

    pub fn sample_irradiance(
        &self,
        pos: Vec3,
        normal: Vec3,
        view_dir: Vec3,
        sdf: Option<&crate::core::engine::rendering::sdf::world_sdf::WorldSdf>,
    ) -> Vec3 {
        let biased_pos = pos + normal * self.normal_bias + view_dir * self.view_bias;
        let indices = self.grid.find_enclosing_probes(biased_pos);
        let weights = self.grid.trilinear_weights(biased_pos);

        let mut irradiance = Vec3::ZERO;
        let mut total_weight = 0.0_f64;

        for (i, &probe_idx) in indices.iter().enumerate() {
            if probe_idx >= self.probes.len() {
                continue;
            }
            let probe_pos = self.grid.probe_position(probe_idx);
            let to_probe = (probe_pos - pos).normalize();
            let crush = to_probe.dot(normal).max(0.0001);
            let sdf_vis = if let Some(sdf_ref) = sdf {
                let probe_dist = (probe_pos - pos).length().max(1e-6);
                let midpoint = pos + to_probe * probe_dist * 0.5;
                (sdf_ref.sample(midpoint) * 4.0 / probe_dist.max(0.25)).clamp(0.0, 1.0)
            } else {
                1.0
            };
            let w = weights[i] * crush * sdf_vis;
            irradiance += self.probes[probe_idx].sample_irradiance(normal) * w;
            total_weight += w;
        }

        if total_weight > 1e-6 {
            irradiance * (1.0 / total_weight)
        } else {
            Vec3::ZERO
        }
    }
}

fn fibonacci_sphere(index: usize, total: usize, seed: u32) -> Vec3 {
    use std::f64::consts::TAU;
    let golden_ratio = (1.0 + 5.0_f64.sqrt()) * 0.5;
    let theta = (1.0 - 2.0 * (index as f64 + 0.5) / total as f64).acos();
    let phi = TAU * (index as f64 / golden_ratio);
    let jitter = (seed.wrapping_mul(0x9E37_79B9) >> 8) as f64 / 16_777_216.0 - 0.5;
    let (sin_t, cos_t) = theta.sin_cos();
    let (sin_p, cos_p) = (phi + jitter * 0.1).sin_cos();
    Vec3::new(sin_t * cos_p, cos_t, sin_t * sin_p)
}

fn oct_encode(n: Vec3) -> (f64, f64) {
    let l1 = n.x.abs() + n.y.abs() + n.z.abs();
    let mut x = n.x / l1;
    let mut y = n.z / l1;
    if n.y < 0.0 {
        let ox = x;
        x = (1.0 - y.abs()) * ox.signum();
        y = (1.0 - ox.abs()) * y.signum();
    }
    (x, y)
}

fn bilinear_irradiance(
    tex: &[[Vec3; DDGI_IRRADIANCE_TEXELS]; DDGI_IRRADIANCE_TEXELS],
    tx: f64,
    ty: f64,
) -> Vec3 {
    let x0 = (tx as usize).min(DDGI_IRRADIANCE_TEXELS - 2);
    let y0 = (ty as usize).min(DDGI_IRRADIANCE_TEXELS - 2);
    let fx = tx - x0 as f64;
    let fy = ty - y0 as f64;
    tex[y0][x0] * ((1.0 - fx) * (1.0 - fy))
        + tex[y0][x0 + 1] * (fx * (1.0 - fy))
        + tex[y0 + 1][x0] * ((1.0 - fx) * fy)
        + tex[y0 + 1][x0 + 1] * (fx * fy)
}
