use crate::core::engine::rendering::raytracing::{Ray, Vec3};
use crate::core::engine::rendering::utils::fbm_3d;

#[derive(Debug, Clone, Copy)]
pub enum DensityField {
    Uniform { density: f64 },
    Exponential { base_density: f64, height_falloff: f64 },
    FbmNoise { base_density: f64, scale: f64, octaves: u32, offset: Vec3 },
    Sphere { center: Vec3, radius: f64, density: f64, falloff: f64 },
}

impl DensityField {
    pub fn sample(&self, pos: Vec3) -> f64 {
        match *self {
            DensityField::Uniform { density } => density,
            DensityField::Exponential { base_density, height_falloff } => {
                base_density * (-height_falloff * pos.y.max(0.0)).exp()
            }
            DensityField::FbmNoise { base_density, scale, octaves, offset } => {
                let p = (pos + offset) * scale;
                let noise = fbm_3d(p, octaves, 2.0, 0.5);
                (base_density * noise.max(0.0)).max(0.0)
            }
            DensityField::Sphere { center, radius, density, falloff } => {
                let d = (pos - center).length();
                if d >= radius { return 0.0; }
                let t = 1.0 - d / radius;
                density * t.powf(falloff)
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct HeterogeneousVolume {
    pub density_field: DensityField,
    pub scattering_albedo: Vec3,
    pub emission: Vec3,
    pub phase_g: f64,
    pub step_size: f64,
    pub max_steps: u32,
    pub absorption_coeff: f64,
}

impl Default for HeterogeneousVolume {
    fn default() -> Self {
        Self {
            density_field: DensityField::Uniform { density: 0.0 },
            scattering_albedo: Vec3::new(0.8, 0.8, 0.9),
            emission: Vec3::ZERO,
            phase_g: 0.0,
            step_size: 0.5,
            max_steps: 64,
            absorption_coeff: 0.1,
        }
    }
}

impl HeterogeneousVolume {
    pub fn nebula_cloud() -> Self {
        Self {
            density_field: DensityField::FbmNoise {
                base_density: 0.8,
                scale: 0.3,
                octaves: 5,
                offset: Vec3::ZERO,
            },
            scattering_albedo: Vec3::new(0.6, 0.7, 0.9),
            emission: Vec3::new(0.05, 0.03, 0.08),
            phase_g: 0.2,
            step_size: 0.25,
            max_steps: 128,
            absorption_coeff: 0.15,
        }
    }

    pub fn atmosphere_low() -> Self {
        Self {
            density_field: DensityField::Exponential { base_density: 0.15, height_falloff: 0.3 },
            scattering_albedo: Vec3::new(0.8, 0.85, 1.0),
            emission: Vec3::ZERO,
            phase_g: 0.5,
            step_size: 1.0,
            max_steps: 48,
            absorption_coeff: 0.02,
        }
    }

    pub fn march(
        &self,
        ray: Ray,
        t_min: f64,
        t_max: f64,
        sun_dir: Vec3,
        sun_color: Vec3,
        sun_intensity: f64,
    ) -> (Vec3, f64) {
        let step = self.step_size;
        let steps = ((t_max - t_min) / step).ceil() as u32;
        let steps = steps.min(self.max_steps);
        let actual_step = (t_max - t_min) / steps.max(1) as f64;

        let mut transmittance = 1.0_f64;
        let mut radiance = Vec3::ZERO;

        for i in 0..steps {
            let t = t_min + (i as f64 + 0.5) * actual_step;
            let pos = ray.at(t);
            let density = self.density_field.sample(pos);
            if density < 1e-6 { continue; }

            let sigma_s = density * self.scattering_albedo;
            let sigma_a = Vec3::splat(density * self.absorption_coeff);
            let sigma_t = sigma_s + sigma_a;
            let mean_sigma_t = (sigma_t.x + sigma_t.y + sigma_t.z) / 3.0;

            let step_transmittance = (-mean_sigma_t * actual_step).exp();

            let sun_phase = henyey_greenstein(ray.direction.dot(sun_dir.normalize()), self.phase_g);

            let sun_visibility = self.shadow_march(pos, sun_dir.normalize(), 20.0);

            let in_scatter = sigma_s * sun_color * sun_intensity * sun_phase * sun_visibility;

            let emission_term = self.emission * density;

            let contrib = (in_scatter + emission_term) * transmittance * (1.0 - step_transmittance) / mean_sigma_t.max(1e-12);
            radiance += contrib;
            transmittance *= step_transmittance;

            if transmittance < 1e-4 { break; }
        }

        (radiance, transmittance)
    }

    fn shadow_march(&self, origin: Vec3, dir: Vec3, max_dist: f64) -> f64 {
        let shadow_step = self.step_size * 2.0;
        let steps = (max_dist / shadow_step).ceil() as u32;
        let steps = steps.min(16);
        let actual_step = max_dist / steps.max(1) as f64;
        let mut optical_depth = 0.0_f64;

        for i in 0..steps {
            let t = (i as f64 + 0.5) * actual_step;
            let pos = Vec3::new(
                origin.x + dir.x * t,
                origin.y + dir.y * t,
                origin.z + dir.z * t,
            );
            optical_depth += self.density_field.sample(pos) * self.absorption_coeff * actual_step;
            if optical_depth > 10.0 { break; }
        }

        (-optical_depth).exp()
    }

    pub fn total_transmittance(&self, ray: Ray, t_min: f64, t_max: f64) -> f64 {
        let steps = ((t_max - t_min) / self.step_size).ceil() as u32;
        let steps = steps.min(self.max_steps);
        let actual_step = (t_max - t_min) / steps.max(1) as f64;
        let mut optical_depth = 0.0_f64;

        for i in 0..steps {
            let t = t_min + (i as f64 + 0.5) * actual_step;
            let pos = ray.at(t);
            let density = self.density_field.sample(pos);
            let sigma_t = density * (self.absorption_coeff + (self.scattering_albedo.x + self.scattering_albedo.y + self.scattering_albedo.z) / 3.0);
            optical_depth += sigma_t * actual_step;
        }

        (-optical_depth).exp()
    }
}

#[inline]
fn henyey_greenstein(cos_theta: f64, g: f64) -> f64 {
    let g2 = g * g;
    let denom = (1.0 + g2 - 2.0 * g * cos_theta).powf(1.5).max(1e-9);
    (1.0 - g2) / (4.0 * std::f64::consts::PI * denom)
}
