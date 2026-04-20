
use crate::core::engine::rendering::raytracing::Vec3;

// ── Phase functions ─────────────────────────────────────────────────────

#[inline]
pub fn rayleigh_phase(cos_theta: f64) -> f64 {
    (3.0 / (16.0 * std::f64::consts::PI)) * (1.0 + cos_theta * cos_theta)
}

pub fn mie_phase(cos_theta: f64, g: f64) -> f64 {
    let g2 = g * g;
    let denom = 1.0 + g2 - 2.0 * g * cos_theta;
    (3.0 / (8.0 * std::f64::consts::PI))
        * ((1.0 - g2) * (1.0 + cos_theta * cos_theta))
        / ((2.0 + g2) * denom.powf(1.5).max(0.001))
}

#[inline]
pub fn rayleigh_scatter(wavelength_nm: f64) -> f64 {
    let lambda = wavelength_nm * 1e-9;
    1.0 / (lambda * lambda * lambda * lambda)
}

// ── Atmospheric scattering model ────────────────────────────────────────

#[derive(Debug, Clone, Copy)]
pub struct AtmosphereParams {
    pub planet_radius: f64,
    pub atmosphere_height: f64,
    pub rayleigh_scale_height: f64,
    pub mie_scale_height: f64,
    pub rayleigh_coeff: Vec3,
    pub mie_coeff: f64,
    pub mie_g: f64,
    pub sun_intensity: f64,
}

impl AtmosphereParams {
    pub fn earth_like() -> Self {
        Self {
            planet_radius: 6371e3,
            atmosphere_height: 100e3,
            rayleigh_scale_height: 8500.0,
            mie_scale_height: 1200.0,
            rayleigh_coeff: Vec3::new(5.8e-6, 13.5e-6, 33.1e-6),
            mie_coeff: 21e-6,
            mie_g: 0.758,
            sun_intensity: 20.0,
        }
    }

    pub fn mars_like() -> Self {
        Self {
            planet_radius: 3389.5e3,
            atmosphere_height: 60e3,
            rayleigh_scale_height: 11100.0,
            mie_scale_height: 1500.0,
            rayleigh_coeff: Vec3::new(19.918e-6, 13.57e-6, 5.75e-6),
            mie_coeff: 40e-6,
            mie_g: 0.65,
            sun_intensity: 10.0,
        }
    }

    pub fn compute_sky_color(&self, view_dir: Vec3, sun_dir: Vec3, steps: u32) -> Vec3 {
        let cos_theta = view_dir.dot(sun_dir).clamp(-1.0, 1.0);
        let ray_phase = rayleigh_phase(cos_theta);
        let m_phase = mie_phase(cos_theta, self.mie_g);

        let up = Vec3::new(0.0, 1.0, 0.0);
        let altitude_factor = view_dir.dot(up).max(0.0);

        // Use `steps` to refine the optical depth integration along the view ray
        let step_scale = 1.0 / steps.max(1) as f64;
        let mut rayleigh_accum = Vec3::ZERO;
        let mut mie_accum = 0.0_f64;
        for i in 0..steps.max(1) {
            let t = (i as f64 + 0.5) * step_scale;
            let alt = altitude_factor * t;
            rayleigh_accum += self.rayleigh_coeff
                    * ((-alt * self.atmosphere_height / self.rayleigh_scale_height).exp());
            mie_accum +=
                self.mie_coeff * (-alt * self.atmosphere_height / self.mie_scale_height).exp();
        }
        let rayleigh_optical = rayleigh_accum * step_scale;
        let mie_optical = mie_accum * step_scale;

        let scatter = rayleigh_optical * ray_phase + Vec3::splat(mie_optical * m_phase);
        let extinction = rayleigh_optical + Vec3::splat(mie_optical);

        let transmittance = Vec3::new(
            (-extinction.x * self.atmosphere_height * 0.001).exp(),
            (-extinction.y * self.atmosphere_height * 0.001).exp(),
            (-extinction.z * self.atmosphere_height * 0.001).exp(),
        );

        scatter * self.sun_intensity * transmittance
    }
}
