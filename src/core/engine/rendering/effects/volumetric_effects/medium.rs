//! Participating medium model with Henyey-Greenstein phase functions,
//! noise-driven density and full inscattering ray-march.
//!
//! [`VolumetricMedium`] describes the optical properties of a volume
//! (nebula, fog, atmosphere…) and provides both cheap single-sample
//! evaluation and multi-step integration paths.

use crate::core::engine::rendering::raytracing::{DirectionalLight, Ray, Scene, Vec3};
use crate::core::engine::rendering::utils::fbm_3d;

// ── Volumetric medium ───────────────────────────────────────────────────

/// Describes a homogeneous or noise-modulated participating medium.
///
/// Medium presets can be composed with builder methods:
/// ```ignore
/// let foggy = VolumetricMedium::dense_fog()
///     .with_density_multiplier(1.4)
///     .with_wind(Vec3::new(1.0, 0.0, 0.0), 0.5);
/// ```
#[derive(Debug, Clone, Copy)]
pub struct VolumetricMedium {
    /// Base density multiplier (`0.0` = vacuum).
    pub density: f64,
    /// Henyey-Greenstein asymmetry parameter `g` (`-1..1`).
    /// Positive → forward scattering, negative → back scattering.
    pub anisotropy: f64,
    /// Exponential height falloff rate.
    pub height_falloff: f64,
    /// Scattering albedo colour.
    pub color: Vec3,
    /// Volumetric self-emission.
    pub emission: Vec3,
    /// Absorption coefficient (beer-law extra).
    pub absorption: f64,
    /// Spatial frequency of the noise field.
    pub noise_scale: f64,
    /// Number of fBm octaves for density noise.
    pub noise_octaves: u32,
    /// Normalised direction the noise field scrolls.
    pub wind_direction: Vec3,
    /// Speed of wind-driven noise animation (world units / s).
    pub wind_speed: f64,
}

// ── Presets ──────────────────────────────────────────────────────────────

impl VolumetricMedium {
    /// A perfectly transparent medium (no scattering, no absorption).
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

    /// Rich, colourful nebula medium suitable for space scenes.
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

    /// Thin, blue-ish atmosphere for planetary horizons.
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

    /// Thick ground-level fog with gentle turbulence.
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
    /// Returns a copy with density scaled by `factor`, clamped to `[0, 1]`.
    pub fn with_density_multiplier(mut self, factor: f64) -> Self {
        self.density = (self.density * factor).clamp(0.0, 1.0);
        self
    }

    /// Adds a wind animation to the noise field.
    pub fn with_wind(mut self, direction: Vec3, speed: f64) -> Self {
        self.wind_direction = direction.normalize();
        self.wind_speed = speed;
        self
    }
}

// ── Density sampling ────────────────────────────────────────────────────

impl VolumetricMedium {
    /// Evaluates density at `point` without wind animation (`time = 0`).
    #[inline]
    pub fn local_density(&self, point: Vec3) -> f64 {
        self.local_density_at_time(point, 0.0)
    }

    /// Evaluates density at `point` with wind-driven noise scrolling.
    ///
    /// Returns a clamped density in `[0, 1.5]`.
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
    /// Cheap single-sample transmittance approximation at `point` for
    /// a given traversal `distance`.
    pub fn transmittance(&self, point: Vec3, distance: f64) -> f64 {
        let sigma = self.local_density(point) * (1.0 + self.absorption);
        (-sigma * distance * 0.18).exp().clamp(0.0, 1.0)
    }

    /// Multi-step ray-march transmittance (Beer-Lambert) along `ray`
    /// from `t_start` to `t_end`.
    ///
    /// Higher `steps` improves accuracy at the cost of more density
    /// evaluations.
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
    /// Standard Henyey-Greenstein phase function using the medium's
    /// `anisotropy` parameter.
    fn henyey_greenstein(&self, cos_theta: f64) -> f64 {
        let g = self.anisotropy.clamp(-0.95, 0.95);
        let denominator = 1.0 + g * g - 2.0 * g * cos_theta;
        (1.0 - g * g) / (4.0 * std::f64::consts::PI * denominator.powf(1.5)).max(0.02)
    }

    /// Dual-lobe phase: 70 % forward lobe + 30 % backward lobe for
    /// more realistic volumetric lighting.
    fn dual_lobe_phase(&self, cos_theta: f64) -> f64 {
        let forward = self.henyey_greenstein_g(cos_theta, self.anisotropy.abs());
        let back = self.henyey_greenstein_g(cos_theta, -self.anisotropy.abs() * 0.3);
        forward * 0.7 + back * 0.3
    }

    /// Henyey-Greenstein with an explicit `g` parameter.
    fn henyey_greenstein_g(&self, cos_theta: f64, g: f64) -> f64 {
        let g = g.clamp(-0.95, 0.95);
        let denom = 1.0 + g * g - 2.0 * g * cos_theta;
        (1.0 - g * g) / (4.0 * std::f64::consts::PI * denom.powf(1.5)).max(0.02)
    }
}

// ── Inscattering ────────────────────────────────────────────────────────

impl VolumetricMedium {
    /// Cheap single-sample inscattering — samples at 35 % of the ray
    /// distance for a visually plausible result without a full march.
    pub fn inscattering(&self, ray: Ray, distance: f64, sun: DirectionalLight) -> Vec3 {
        if self.density <= f64::EPSILON {
            return Vec3::ZERO;
        }

        let sample_point = ray.at(distance * 0.35);
        let density = self.local_density(sample_point);
        let sun_dir = (-sun.direction).normalize();
        let phase = self.henyey_greenstein(ray.direction.dot(sun_dir).max(0.0));
        let absorption = 1.0 - self.transmittance(sample_point, distance);

        (self.color * density * sun.intensity * 0.32 * phase + self.emission * density * 0.75)
            * absorption
    }

    /// Full-quality inscattering ray-march with volumetric shadows.
    ///
    /// For each step along the primary ray:
    /// 1. Evaluate local density.
    /// 2. Cast a shadow ray toward the sun (scene occlusion check +
    ///    secondary transmittance march).
    /// 3. Accumulate in-scattered light weighted by the dual-lobe phase
    ///    function and the running transmittance.
    ///
    /// Early-terminates when transmittance drops below 1 %.
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
