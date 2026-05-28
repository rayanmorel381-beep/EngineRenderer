use crate::core::engine::rendering::framebuffer::FrameBuffer;
use crate::core::engine::rendering::raytracing::Vec3;

pub const IBL_FACE_SIZE: usize = 64;
pub const IBL_FACE_COUNT: usize = 6;
pub const IBL_MIP_LEVELS: usize = 8;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CubeFace {
    PosX = 0,
    NegX = 1,
    PosY = 2,
    NegY = 3,
    PosZ = 4,
    NegZ = 5,
}

#[derive(Debug, Clone)]
pub struct IblProbe {
    pub position: Vec3,
    pub radiance: Vec<Vec3>,
    pub irradiance: Vec<Vec3>,
    pub resolution: usize,
    pub mip_levels: usize,
}

impl IblProbe {
    pub fn new(position: Vec3, resolution: usize) -> Self {
        let total = IBL_FACE_COUNT * resolution * resolution;
        let irr_total = IBL_FACE_COUNT * 8 * 8;
        Self {
            position,
            radiance: vec![Vec3::ZERO; total],
            irradiance: vec![Vec3::ZERO; irr_total],
            resolution,
            mip_levels: IBL_MIP_LEVELS,
        }
    }

    pub fn sample(&self, dir: Vec3) -> Vec3 {
        let (face, u, v) = dir_to_face_uv(dir);
        let fi = face as usize;
        let px =
            (u * (self.resolution - 1) as f64).clamp(0.0, (self.resolution - 1) as f64) as usize;
        let py =
            (v * (self.resolution - 1) as f64).clamp(0.0, (self.resolution - 1) as f64) as usize;
        let idx = fi * self.resolution * self.resolution + py * self.resolution + px;
        if idx < self.radiance.len() {
            self.radiance[idx]
        } else {
            Vec3::ZERO
        }
    }

    pub fn sample_irradiance_diffuse(&self, normal: Vec3) -> Vec3 {
        let (face, u, v) = dir_to_face_uv(normal);
        let fi = face as usize;
        let px = (u * 7.0).clamp(0.0, 7.0) as usize;
        let py = (v * 7.0).clamp(0.0, 7.0) as usize;
        let idx = fi * 64 + py * 8 + px;
        if idx < self.irradiance.len() {
            self.irradiance[idx]
        } else {
            Vec3::ZERO
        }
    }

    pub fn sample_specular(&self, reflect_dir: Vec3, roughness: f64) -> Vec3 {
        let mip = (roughness * (self.mip_levels - 1) as f64) as usize;
        let scale = 1.0 / (1 << mip) as f64;
        let res = ((self.resolution as f64 * scale) as usize).max(1);
        let (face, u, v) = dir_to_face_uv(reflect_dir);
        let fi = face as usize;
        let px = (u * (res - 1) as f64).clamp(0.0, (res - 1) as f64) as usize;
        let py = (v * (res - 1) as f64).clamp(0.0, (res - 1) as f64) as usize;
        let base = fi * self.resolution * self.resolution;
        let idx = base + py * res + px;
        if idx < self.radiance.len() {
            self.radiance[idx]
        } else {
            Vec3::ZERO
        }
    }
}

fn dir_to_face_uv(dir: Vec3) -> (CubeFace, f64, f64) {
    let ax = dir.x.abs();
    let ay = dir.y.abs();
    let az = dir.z.abs();
    if ax >= ay && ax >= az {
        if dir.x > 0.0 {
            (
                CubeFace::PosX,
                (-dir.z / dir.x) * 0.5 + 0.5,
                (-dir.y / dir.x) * 0.5 + 0.5,
            )
        } else {
            (
                CubeFace::NegX,
                (dir.z / -dir.x) * 0.5 + 0.5,
                (-dir.y / -dir.x) * 0.5 + 0.5,
            )
        }
    } else if ay >= ax && ay >= az {
        if dir.y > 0.0 {
            (
                CubeFace::PosY,
                (dir.x / dir.y) * 0.5 + 0.5,
                (dir.z / dir.y) * 0.5 + 0.5,
            )
        } else {
            (
                CubeFace::NegY,
                (dir.x / -dir.y) * 0.5 + 0.5,
                (-dir.z / -dir.y) * 0.5 + 0.5,
            )
        }
    } else if dir.z > 0.0 {
        (
            CubeFace::PosZ,
            (dir.x / dir.z) * 0.5 + 0.5,
            (-dir.y / dir.z) * 0.5 + 0.5,
        )
    } else {
        (
            CubeFace::NegZ,
            (-dir.x / -dir.z) * 0.5 + 0.5,
            (-dir.y / -dir.z) * 0.5 + 0.5,
        )
    }
}

#[derive(Debug, Clone, Copy)]
pub struct SsrConfig {
    pub max_steps: u32,
    pub step_size: f64,
    pub thickness: f64,
    pub fallback_ibl: bool,
    pub fade_start: f64,
    pub fade_end: f64,
    pub roughness_cutoff: f64,
}

impl Default for SsrConfig {
    fn default() -> Self {
        Self {
            max_steps: 64,
            step_size: 0.05,
            thickness: 0.1,
            fallback_ibl: true,
            fade_start: 0.7,
            fade_end: 0.95,
            roughness_cutoff: 0.6,
        }
    }
}

fn screen_to_uv(fb: &FrameBuffer, pos: Vec3, proj: &[[f64; 4]; 4]) -> Option<(f64, f64, f64)> {
    let cx = proj[0][0] * pos.x + proj[1][0] * pos.y + proj[2][0] * pos.z + proj[3][0];
    let cy = proj[0][1] * pos.x + proj[1][1] * pos.y + proj[2][1] * pos.z + proj[3][1];
    let cz = proj[0][2] * pos.x + proj[1][2] * pos.y + proj[2][2] * pos.z + proj[3][2];
    let cw = proj[0][3] * pos.x + proj[1][3] * pos.y + proj[2][3] * pos.z + proj[3][3];
    if cw < 1e-4 {
        return None;
    }
    let ndx = cx / cw;
    let ndy = cy / cw;
    let ndz = cz / cw;
    if !(-1.0..=1.0).contains(&ndx) || !(-1.0..=1.0).contains(&ndy) {
        return None;
    }
    let ux = ndx * 0.5 + 0.5;
    let uy = 1.0 - (ndy * 0.5 + 0.5);
    let px = (ux * fb.width as f64) as usize;
    let py = (uy * fb.height as f64) as usize;
    if px >= fb.width || py >= fb.height {
        return None;
    }
    Some((ux, uy, ndz * 0.5 + 0.5))
}

fn sample_fb_bilinear(fb: &FrameBuffer, u: f64, v: f64) -> Vec3 {
    let fx = (u * fb.width as f64).clamp(0.0, fb.width as f64 - 1.001);
    let fy = (v * fb.height as f64).clamp(0.0, fb.height as f64 - 1.001);
    let x0 = fx as usize;
    let y0 = fy as usize;
    let x1 = (x0 + 1).min(fb.width - 1);
    let y1 = (y0 + 1).min(fb.height - 1);
    let tx = fx - x0 as f64;
    let ty = fy - y0 as f64;
    let w = fb.width;
    fb.color[y0 * w + x0] * ((1.0 - tx) * (1.0 - ty))
        + fb.color[y0 * w + x1] * (tx * (1.0 - ty))
        + fb.color[y1 * w + x0] * ((1.0 - tx) * ty)
        + fb.color[y1 * w + x1] * (tx * ty)
}

fn trace_screen_reflection(
    view_pos: Vec3,
    reflect_dir: Vec3,
    fb: &FrameBuffer,
    depth_fb: &[f64],
    proj: &[[f64; 4]; 4],
    config: &SsrConfig,
) -> Option<(Vec3, f64)> {
    let mut ray_pos = view_pos;
    let step = reflect_dir * config.step_size;
    let w = fb.width;
    let h = fb.height;

    for _ in 0..config.max_steps {
        ray_pos += step;
        if let Some((u, v, ray_depth)) = screen_to_uv(fb, ray_pos, proj) {
            let px = (u * w as f64) as usize;
            let py = (v * h as f64) as usize;
            if px >= w || py >= h {
                continue;
            }
            let scene_depth = depth_fb[py * w + px];
            let delta = ray_depth - scene_depth;
            if delta > 0.0 && delta < config.thickness {
                let screen_dist = (u - 0.5).abs().max((v - 0.5).abs());
                let fade = 1.0
                    - ((screen_dist - config.fade_start) / (config.fade_end - config.fade_start))
                        .clamp(0.0, 1.0);
                let color = sample_fb_bilinear(fb, u, v);
                return Some((color, fade));
            }
        }
    }
    None
}

pub struct SsrBuffers<'a> {
    pub depth: &'a [f64],
    pub normals: &'a [Vec3],
    pub roughness: &'a [f64],
    pub view_positions: &'a [Vec3],
    pub proj: &'a [[f64; 4]; 4],
}

pub struct SsrPass {
    pub config: SsrConfig,
}

impl SsrPass {
    pub fn new(config: SsrConfig) -> Self {
        Self { config }
    }

    pub fn execute(
        &self,
        fb: &FrameBuffer,
        buffers: SsrBuffers<'_>,
        probe: Option<&IblProbe>,
    ) -> FrameBuffer {
        let mut out = fb.clone();
        let w = fb.width;
        let h = fb.height;

        for y in 0..h {
            for x in 0..w {
                let idx = y * w + x;
                let roughness = buffers.roughness[idx];
                if roughness > self.config.roughness_cutoff {
                    continue;
                }

                let view_pos = buffers.view_positions[idx];
                let normal = buffers.normals[idx].normalize();
                let view_dir = (-view_pos).normalize();
                let reflect_dir = reflect_ray(view_dir, normal);

                let reflection = trace_screen_reflection(
                    view_pos,
                    reflect_dir,
                    fb,
                    buffers.depth,
                    buffers.proj,
                    &self.config,
                );

                let ibl_fallback =
                    probe.map_or(Vec3::ZERO, |p| p.sample_specular(reflect_dir, roughness));
                let ibl_diffuse = probe.map_or(Vec3::ZERO, |p| p.sample_irradiance_diffuse(normal));
                let ibl_ambient = probe.map_or(Vec3::ZERO, |p| p.sample(normal));

                let blended = match reflection {
                    Some((ssr_color, fade)) => {
                        if self.config.fallback_ibl {
                            ssr_color * fade
                                + ibl_fallback * (1.0 - fade)
                                + ibl_diffuse * roughness * 0.1
                        } else {
                            ssr_color * fade + ibl_ambient * roughness * 0.05
                        }
                    }
                    None => {
                        if self.config.fallback_ibl {
                            ibl_fallback + ibl_diffuse * roughness * 0.1
                        } else {
                            ibl_ambient * roughness * 0.05
                        }
                    }
                };

                let fresnel = fresnel_schlick(normal.dot(view_dir).clamp(0.0, 1.0), 0.04);
                out.color[idx] += blended * fresnel * (1.0 - roughness);
            }
        }
        out
    }
}

fn reflect_ray(view: Vec3, normal: Vec3) -> Vec3 {
    let d = -view;
    (d - normal * 2.0 * d.dot(normal)).normalize()
}

fn fresnel_schlick(cos_theta: f64, f0: f64) -> f64 {
    f0 + (1.0 - f0) * (1.0 - cos_theta).powi(5)
}
