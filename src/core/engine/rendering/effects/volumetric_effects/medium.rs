
use crate::core::engine::rendering::raytracing::{DirectionalLight, Ray, Scene, Vec3};
use crate::core::engine::rendering::utils::fbm_3d;

// ── Volumetric medium ───────────────────────────────────────────────────

#[derive(Debug, Clone, Copy)]
pub struct VolumetricMedium {
    pub density: f64,
    pub anisotropy: f64,
    pub height_falloff: f64,
    pub color: Vec3,
    pub emission: Vec3,
    pub absorption: f64,
    pub noise_scale: f64,
    pub noise_octaves: u32,
    pub wind_direction: Vec3,
    pub wind_speed: f64,
}

// ── Presets ──────────────────────────────────────────────────────────────

impl VolumetricMedium {
    pub fn vacuum() -> Self {
        Self {
            density: 0.0,
            anisotropy: 0.0,
            height_falloff: 0.0,
            color: Vec3::ZERO,
            emission: Vec3::ZERO,
            absorption: 0.0,
            noise_scale: 1.0,
            noise_octaves: 1,
            wind_direction: Vec3::ZERO,
            wind_speed: 0.0,
        }
    }

    pub fn cinematic_nebula() -> Self {
        Self {
            density: 0.028,
            anisotropy: 0.42,
            height_falloff: 0.14,
            color: Vec3::new(0.34, 0.40, 0.56),
            emission: Vec3::new(0.018, 0.022, 0.035),
            absorption: 0.12,
            noise_scale: 0.18,
            noise_octaves: 4,
            wind_direction: Vec3::new(1.0, 0.0, 0.3).normalize(),
            wind_speed: 0.0,
        }
    }

    pub fn thin_atmosphere() -> Self {
        Self {
            density: 0.005,
            anisotropy: 0.76,
            height_falloff: 0.35,
            color: Vec3::new(0.55, 0.70, 0.95),
            emission: Vec3::ZERO,
            absorption: 0.02,
            noise_scale: 0.05,
            noise_octaves: 2,
            wind_direction: Vec3::new(1.0, 0.0, 0.0),
            wind_speed: 0.0,
        }
    }

    pub fn dense_fog() -> Self {
        Self {
            density: 0.15,
            anisotropy: 0.10,
            height_falloff: 0.60,
            color: Vec3::new(0.80, 0.82, 0.85),
            emission: Vec3::ZERO,
            absorption: 0.35,
            noise_scale: 0.08,
            noise_octaves: 3,
            wind_direction: Vec3::new(0.5, 0.0, 0.5).normalize(),
            wind_speed: 0.0,
        }
    }
}

// ── Builders ────────────────────────────────────────────────────────────

impl VolumetricMedium {
    pub fn with_density_multiplier(mut self, factor: f64) -> Self {
        self.density = (self.density * factor).clamp(0.0, 1.0);
        self
    }

    pub fn with_wind(mut self, direction: Vec3, speed: f64) -> Self {
        self.wind_direction = direction.normalize();
        self.wind_speed = speed;
        self
    }
}

// ── Density sampling ────────────────────────────────────────────────────

impl VolumetricMedium {
    #[inline]
    pub fn local_density(&self, point: Vec3) -> f64 {
        self.local_density_at_time(point, 0.0)
    }

    #[inline]
    pub fn local_density_fast(&self, point: Vec3) -> f64 {
        if self.density <= f64::EPSILON {
            return 0.0;
        }
        let height_term = (-point.y.abs() * self.height_falloff).exp();
        let noise = 0.72
            + ((point.x * self.noise_scale).sin()
                * (point.z * self.noise_scale * 0.61).cos())
            .abs()
                * 0.58
            + ((point.x + point.y) * self.noise_scale * 0.33)
                .sin()
                .abs()
                * 0.15;
        (self.density * height_term * noise.max(0.0)).clamp(0.0, 1.5)
    }

    pub fn local_density_at_time(&self, point: Vec3, time: f64) -> f64 {
        if self.density <= f64::EPSILON {
            return 0.0;
        }

        let animated = point + self.wind_direction * self.wind_speed * time;
        let height_term = (-animated.y.abs() * self.height_falloff).exp();

        let noise = if self.noise_octaves > 1 {
            fbm_3d(
                animated * self.noise_scale,
                self.noise_octaves,
                2.0,
                0.5,
            )
        } else {
            0.72
                + ((animated.x * self.noise_scale).sin()
                    * (animated.z * self.noise_scale * 0.61).cos())
                .abs()
                    * 0.58
                + ((animated.x + animated.y) * self.noise_scale * 0.33)
                    .sin()
                    .abs()
                    * 0.15
        };

        (self.density * height_term * noise.max(0.0)).clamp(0.0, 1.5)
    }
}

// ── Transmittance ───────────────────────────────────────────────────────

impl VolumetricMedium {
    pub fn transmittance(&self, point: Vec3, distance: f64) -> f64 {
        let sigma = self.local_density(point) * (1.0 + self.absorption);
        (-sigma * distance * 0.18).exp().clamp(0.0, 1.0)
    }

    pub fn transmittance_ray_march(
        &self,
        ray: Ray,
        t_start: f64,
        t_end: f64,
        steps: u32,
    ) -> f64 {
        let step_size = (t_end - t_start) / steps.max(1) as f64;
        let mut optical_depth = 0.0;

        for i in 0..steps {
            let t = t_start + (i as f64 + 0.5) * step_size;
            let sample = ray.at(t);
            optical_depth += self.local_density(sample) * step_size;
        }

        (-optical_depth * (1.0 + self.absorption)).exp().clamp(0.0, 1.0)
    }
}

// ── Phase functions ─────────────────────────────────────────────────────

impl VolumetricMedium {
    fn henyey_greenstein(&self, cos_theta: f64) -> f64 {
        let g = self.anisotropy.clamp(-0.95, 0.95);
        let denominator = 1.0 + g * g - 2.0 * g * cos_theta;
        (1.0 - g * g) / (4.0 * std::f64::consts::PI * denominator.powf(1.5)).max(0.02)
    }

    fn dual_lobe_phase(&self, cos_theta: f64) -> f64 {
        let forward = self.henyey_greenstein_g(cos_theta, self.anisotropy.abs());
        let back = self.henyey_greenstein_g(cos_theta, -self.anisotropy.abs() * 0.3);
        forward * 0.7 + back * 0.3
    }

    fn henyey_greenstein_g(&self, cos_theta: f64, g: f64) -> f64 {
        let g = g.clamp(-0.95, 0.95);
        let denom = 1.0 + g * g - 2.0 * g * cos_theta;
        (1.0 - g * g) / (4.0 * std::f64::consts::PI * denom.powf(1.5)).max(0.02)
    }
}

// ── Inscattering ────────────────────────────────────────────────────────

impl VolumetricMedium {
    pub fn inscattering(&self, ray: Ray, distance: f64, sun: DirectionalLight) -> Vec3 {
        if self.density <= f64::EPSILON {
            return Vec3::ZERO;
        }

        let sample_point = ray.at(distance * 0.35);
        let density = self.local_density_fast(sample_point);
        let sun_dir = (-sun.direction).normalize();
        let phase = self.henyey_greenstein(ray.direction.dot(sun_dir).max(0.0));
        let sigma = density * (1.0 + self.absorption);
        let absorption = 1.0 - (-sigma * distance * 0.18).exp().clamp(0.0, 1.0);

        (self.color * density * sun.intensity * 0.32 * phase + self.emission * density * 0.75)
            * absorption
    }

    pub fn inscattering_ray_march(
        &self,
        ray: Ray,
        t_start: f64,
        t_end: f64,
        steps: u32,
        sun: DirectionalLight,
        scene: &Scene,
    ) -> Vec3 {
        if self.density <= f64::EPSILON {
            return Vec3::ZERO;
        }

        let step_size = (t_end - t_start) / steps.max(1) as f64;
        let sun_dir = (-sun.direction).normalize();
        let mut accumulated_light = Vec3::ZERO;
        let mut transmittance = 1.0;

        for i in 0..steps {
            let t = t_start + (i as f64 + 0.5) * step_size;
            let sample = ray.at(t);
            let density = self.local_density(sample);

            if density < 0.0001 {
                continue;
            }

            // Light visibility (volumetric shadow)
            let shadow_ray = Ray::new(sample, sun_dir);
            let shadow_vis = if scene.is_occluded(&shadow_ray, 1e5) {
                0.05
            } else {
                self.transmittance_ray_march(Ray::new(sample, sun_dir), 0.0, 50.0, 4)
            };

            let phase = self.dual_lobe_phase(ray.direction.dot(sun_dir));
            let in_scatter =
                self.color * density * sun.intensity * sun.color * phase * shadow_vis;
            let emit = self.emission * density;

            accumulated_light += (in_scatter + emit) * transmittance * step_size;

            let extinction = density * (1.0 + self.absorption);
            transmittance *= (-extinction * step_size).exp();

            // Early termination when almost fully opaque.
            if transmittance < 0.01 {
                break;
            }
        }

        accumulated_light
    }
}
