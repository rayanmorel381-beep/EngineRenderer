use crate::core::engine::rendering::framebuffer::FrameBuffer;
use crate::core::engine::rendering::raytracing::Vec3;

const DIPOLE_BANDS: usize = 3;

#[derive(Debug, Clone, Copy)]
pub struct SssProfile {
    pub scattering_radius: [f64; DIPOLE_BANDS],
    pub falloff: Vec3,
    pub albedo: Vec3,
    pub depth_scale: f64,
    pub strength: f64,
}

impl SssProfile {
    pub fn skin() -> Self {
        Self {
            scattering_radius: [1.0, 0.5, 0.3],
            falloff: Vec3::new(0.22, 0.44, 0.81),
            albedo: Vec3::new(0.88, 0.74, 0.63),
            depth_scale: 1.0,
            strength: 1.0,
        }
    }

    pub fn wax() -> Self {
        Self {
            scattering_radius: [2.5, 2.0, 1.8],
            falloff: Vec3::new(0.05, 0.08, 0.13),
            albedo: Vec3::new(0.95, 0.93, 0.85),
            depth_scale: 0.8,
            strength: 1.2,
        }
    }

    pub fn marble() -> Self {
        Self {
            scattering_radius: [0.8, 0.7, 0.6],
            falloff: Vec3::new(0.10, 0.12, 0.18),
            albedo: Vec3::new(0.90, 0.90, 0.90),
            depth_scale: 1.5,
            strength: 0.8,
        }
    }

    pub fn evaluate_dipole(&self, r_sq: f64, band: usize) -> f64 {
        let sigma_s_prime = 1.0 / self.scattering_radius[band].max(1e-6);
        let sigma_a_val = match band {
            0 => self.falloff.x,
            1 => self.falloff.y,
            _ => self.falloff.z,
        };
        let sigma_t_prime = sigma_s_prime + sigma_a_val;
        let alpha_prime = sigma_s_prime / sigma_t_prime;
        let d = 1.0 / (3.0 * sigma_t_prime);
        let sigma_tr = (3.0 * sigma_a_val * sigma_t_prime).sqrt();

        let r = r_sq.sqrt().max(1e-4);
        let r_real = (r * r + d * d).sqrt();
        let r_virt = (r * r + (d * (1.0 + 4.0 / 3.0)) * (d * (1.0 + 4.0 / 3.0))).sqrt();

        let f_real = ((-sigma_tr * r_real).exp()) / r_real;
        let f_virt = ((-sigma_tr * r_virt).exp()) / r_virt;

        let c_phi = 0.25 / std::f64::consts::PI;
        let c_e = 0.5 * (1.0 - 2.0 * c_phi / 3.0);

        alpha_prime
            * (c_phi * (1.0 / r_real + sigma_tr) * f_real / (r_real)
                - c_e * (1.0 / r_virt + sigma_tr) * f_virt / (r_virt))
    }
}

pub struct SssPass {
    pub profile: SssProfile,
    pub kernel_radius: usize,
    pub samples: u32,
}

impl SssPass {
    pub fn new(profile: SssProfile, kernel_radius: usize, samples: u32) -> Self {
        Self {
            profile,
            kernel_radius,
            samples,
        }
    }

    pub fn apply(&self, fb: &FrameBuffer, normal_fb: &[Vec3], depth_fb: &[f64]) -> FrameBuffer {
        let w = fb.width;
        let h = fb.height;
        let mut out = fb.clone();

        let r = self.kernel_radius as i64;

        for y in 0..h {
            for x in 0..w {
                let idx = y * w + x;
                let center_depth = depth_fb[idx];
                if center_depth >= 1.0 {
                    continue;
                }
                let center_normal = normal_fb[idx];

                let mut sum_r = 0.0_f64;
                let mut sum_g = 0.0_f64;
                let mut sum_b = 0.0_f64;
                let mut total_weight = Vec3::ZERO;

                for dy in -r..=r {
                    for dx in -r..=r {
                        let sx = x as i64 + dx;
                        let sy = y as i64 + dy;
                        if sx < 0 || sx >= w as i64 || sy < 0 || sy >= h as i64 {
                            continue;
                        }
                        let sidx = sy as usize * w + sx as usize;
                        let sample_depth = depth_fb[sidx];
                        let depth_diff =
                            ((center_depth - sample_depth) * self.profile.depth_scale).abs();
                        if depth_diff > 0.1 {
                            continue;
                        }

                        let normal_sim = center_normal.dot(normal_fb[sidx]).max(0.0);
                        if normal_sim < 0.3 {
                            continue;
                        }

                        let r_sq = (dx * dx + dy * dy) as f64;

                        let wr = self.profile.evaluate_dipole(r_sq, 0).max(0.0) * normal_sim;
                        let wg = self.profile.evaluate_dipole(r_sq, 1).max(0.0) * normal_sim;
                        let wb = self.profile.evaluate_dipole(r_sq, 2).max(0.0) * normal_sim;

                        sum_r += fb.color[sidx].x * wr;
                        sum_g += fb.color[sidx].y * wg;
                        sum_b += fb.color[sidx].z * wb;
                        total_weight += Vec3::new(wr, wg, wb);
                    }
                }

                if total_weight.x > 1e-6 {
                    let quality_scale = (self.samples as f64 / 16.0).clamp(0.5, 1.0);
                    let blurred = Vec3::new(
                        sum_r / total_weight.x,
                        if total_weight.y > 1e-6 {
                            sum_g / total_weight.y
                        } else {
                            fb.color[idx].y
                        },
                        if total_weight.z > 1e-6 {
                            sum_b / total_weight.z
                        } else {
                            fb.color[idx].z
                        },
                    );
                    let sss_contrib = Vec3::new(
                        blurred.x * self.profile.albedo.x,
                        blurred.y * self.profile.albedo.y,
                        blurred.z * self.profile.albedo.z,
                    );
                    let strength = self.profile.strength * quality_scale;
                    out.color[idx] = fb.color[idx] * (1.0 - strength) + sss_contrib * strength;
                }
            }
        }
        out
    }
}
