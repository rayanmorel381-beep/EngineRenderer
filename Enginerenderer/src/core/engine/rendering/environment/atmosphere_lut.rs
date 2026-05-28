use crate::core::engine::rendering::raytracing::Vec3;
use super::scattering::AtmosphereParams;

#[derive(Debug, Clone)]
pub struct TransmittanceLut {
    pub data: Vec<Vec3>,
    pub width: usize,
    pub height: usize,
}

impl TransmittanceLut {
    pub fn precompute(params: &AtmosphereParams, width: usize, height: usize) -> Self {
        let mut data = vec![Vec3::ZERO; width * height];
        for j in 0..height {
            for i in 0..width {
                let altitude = params.atmosphere_height * (j as f64 / height.saturating_sub(1).max(1) as f64);
                let cos_theta = (i as f64 / width.saturating_sub(1).max(1) as f64) * 2.0 - 1.0;
                let t = integrate_transmittance(params, altitude, cos_theta, 64);
                data[j * width + i] = t;
            }
        }
        Self { data, width, height }
    }

    pub fn sample(&self, altitude: f64, cos_theta: f64) -> Vec3 {
        let u = ((cos_theta + 1.0) * 0.5).clamp(0.0, 1.0);
        let v = (altitude / 1.0).clamp(0.0, 1.0);
        let x = (u * (self.width - 1) as f64) as usize;
        let y = (v * (self.height - 1) as f64) as usize;
        self.data[y * self.width + x]
    }
}

#[derive(Debug, Clone)]
pub struct MultiScatterLut {
    pub data: Vec<Vec3>,
    pub r_size: usize,
    pub mu_s_size: usize,
}

impl MultiScatterLut {
    pub fn precompute(params: &AtmosphereParams, r_size: usize, mu_s_size: usize, transmittance: &TransmittanceLut) -> Self {
        let mut data = vec![Vec3::ZERO; r_size * mu_s_size];
        let dirs = 32;
        for j in 0..r_size {
            for i in 0..mu_s_size {
                let altitude = params.atmosphere_height * (j as f64 / r_size.saturating_sub(1).max(1) as f64);
                let cos_sun = (i as f64 / mu_s_size.saturating_sub(1).max(1) as f64) * 2.0 - 1.0;
                let sun_dir = Vec3::new((1.0 - cos_sun * cos_sun).sqrt(), cos_sun, 0.0);
                let mut ms = Vec3::ZERO;
                for d in 0..dirs {
                    let theta = std::f64::consts::PI * (d as f64 + 0.5) / dirs as f64;
                    let (sin_t, cos_t) = theta.sin_cos();
                    let view = Vec3::new(sin_t, cos_t, 0.0);
                    let primary = params.compute_sky_color(view, sun_dir, 8);
                    let transmit = transmittance.sample(altitude, cos_t);
                    let secondary_scatter = Vec3::new(
                        primary.x * transmit.x,
                        primary.y * transmit.y,
                        primary.z * transmit.z,
                    );
                    ms += secondary_scatter * (1.0 / dirs as f64);
                }
                data[j * mu_s_size + i] = ms;
            }
        }
        Self { data, r_size, mu_s_size }
    }

    pub fn sample(&self, altitude_frac: f64, cos_sun: f64) -> Vec3 {
        let u = ((cos_sun + 1.0) * 0.5).clamp(0.0, 1.0);
        let v = altitude_frac.clamp(0.0, 1.0);
        let x = (u * (self.mu_s_size - 1) as f64) as usize;
        let y = (v * (self.r_size - 1) as f64) as usize;
        self.data[y * self.mu_s_size + x]
    }
}

pub struct AtmosphereLut {
    pub params: AtmosphereParams,
    pub transmittance: TransmittanceLut,
    pub multi_scatter: MultiScatterLut,
}

impl AtmosphereLut {
    pub fn precompute(params: AtmosphereParams, r_steps: usize, mu_steps: usize) -> Self {
        let transmittance = TransmittanceLut::precompute(&params, mu_steps, r_steps);
        let multi_scatter = MultiScatterLut::precompute(&params, r_steps, mu_steps, &transmittance);
        Self { params, transmittance, multi_scatter }
    }

    pub fn sample_sky(&self, view_dir: Vec3, sun_dir: Vec3) -> Vec3 {
        let cos_theta = view_dir.dot(sun_dir);
        let altitude_frac = view_dir.y.max(0.0);
        let primary = self.params.compute_sky_color(view_dir, sun_dir, 16);
        let ms = self.multi_scatter.sample(altitude_frac, cos_theta);
        Vec3::new(
            primary.x + ms.x,
            primary.y + ms.y,
            primary.z + ms.z,
        )
    }

    pub fn aerial_perspective(&self, world_pos: Vec3, camera_pos: Vec3, sun_dir: Vec3) -> Vec3 {
        let dist = (world_pos - camera_pos).length();
        let view_dir = if dist > f64::EPSILON {
            (world_pos - camera_pos) * (1.0 / dist)
        } else {
            Vec3::new(0.0, 0.0, 1.0)
        };
        let cos_theta = view_dir.dot(sun_dir);
        let altitude_frac = view_dir.y.max(0.0);
        let depth_factor = 1.0 - (-dist * 0.001).exp();
        let ms = self.multi_scatter.sample(altitude_frac, cos_theta);
        ms * depth_factor
    }
}

fn integrate_transmittance(params: &AtmosphereParams, altitude: f64, cos_theta: f64, steps: usize) -> Vec3 {
    let r = params.planet_radius + altitude;
    let ray_len = ray_atmosphere_intersect(r, cos_theta, params.planet_radius + params.atmosphere_height);
    if ray_len <= 0.0 { return Vec3::new(1.0, 1.0, 1.0); }

    let dt = ray_len / steps as f64;
    let mut optical_depth_r = 0.0_f64;
    let mut optical_depth_m = 0.0_f64;

    for i in 0..steps {
        let t = (i as f64 + 0.5) * dt;
        let pos_r = (r * r + t * t + 2.0 * r * cos_theta * t).sqrt();
        let h = pos_r - params.planet_radius;
        optical_depth_r += (-h / params.rayleigh_scale_height).exp() * dt;
        optical_depth_m += (-h / params.mie_scale_height).exp() * dt;
    }

    let tau_r = params.rayleigh_coeff * optical_depth_r;
    let tau_m = Vec3::splat(params.mie_coeff * optical_depth_m * 1.1);
    Vec3::new(
        (-(tau_r.x + tau_m.x)).exp(),
        (-(tau_r.y + tau_m.y)).exp(),
        (-(tau_r.z + tau_m.z)).exp(),
    )
}

fn ray_atmosphere_intersect(r: f64, cos_theta: f64, r_atm: f64) -> f64 {
    let discriminant = r_atm * r_atm - r * r * (1.0 - cos_theta * cos_theta);
    if discriminant < 0.0 { return 0.0; }
    -r * cos_theta + discriminant.sqrt()
}
