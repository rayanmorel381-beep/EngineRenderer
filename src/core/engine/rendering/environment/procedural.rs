
use crate::core::engine::rendering::raytracing::Vec3;
use crate::core::engine::rendering::utils::{fbm_3d, smoothstep, value_noise_3d};

use super::scattering::AtmosphereParams;

#[derive(Debug, Clone)]
pub struct ProceduralEnvironment {
    pub horizon_tint: Vec3,
    pub zenith_tint: Vec3,
    pub ground_tint: Vec3,
    pub exposure_bias: f64,
    pub rotation_angle: f64,
    pub atmosphere: Option<AtmosphereParams>,
}

impl ProceduralEnvironment {
    pub fn cinematic_space() -> Self {
        Self {
            horizon_tint: Vec3::new(0.08, 0.04, 0.12),
            zenith_tint: Vec3::new(0.01, 0.005, 0.03),
            ground_tint: Vec3::new(0.005, 0.003, 0.008),
            exposure_bias: 0.0,
            rotation_angle: 0.0,
            atmosphere: None,
        }
    }

    pub fn with_atmosphere(mut self, params: AtmosphereParams) -> Self {
        self.atmosphere = Some(params);
        self
    }

    pub fn with_rotation(mut self, radians: f64) -> Self {
        self.rotation_angle = radians;
        self
    }

    pub fn rotate_direction(&self, dir: Vec3) -> Vec3 {
        let (s, c) = self.rotation_angle.sin_cos();
        Vec3::new(dir.x * c + dir.z * s, dir.y, -dir.x * s + dir.z * c)
    }

    pub fn sky_colors(&self, direction: Vec3, sun_dir: Vec3) -> (Vec3, bool) {
        let dir = self.rotate_direction(direction.normalize());
        let up = dir.y.clamp(-1.0, 1.0);
        let below = up < 0.0;

        let base = if below {
            Vec3::lerp(self.horizon_tint, self.ground_tint, (-up).min(1.0))
        } else {
            Vec3::lerp(self.horizon_tint, self.zenith_tint, up)
        };

        let sky = match &self.atmosphere {
            Some(atmo) => {
                let atmo_color = atmo.compute_sky_color(dir, sun_dir, 32);
                base + atmo_color
            }
            None => base,
        };

        (sky, below)
    }

    pub fn sun_color(&self, direction: Vec3, sun_dir: Vec3) -> Vec3 {
        let dir = self.rotate_direction(direction.normalize());
        let cos_angle = dir.dot(sun_dir);
        let sun_angular_radius = 0.00465;
        if cos_angle > (1.0 - sun_angular_radius) {
            let t = ((cos_angle - (1.0 - sun_angular_radius)) / sun_angular_radius).min(1.0);
            let limb = smoothstep(0.0, 1.0, t);
            Vec3::new(10.0, 8.5, 7.0) * limb
        } else {
            Vec3::ZERO
        }
    }

    pub fn hdri_probe(&self, direction: Vec3, sun_dir: Vec3) -> Vec3 {
        let (sky, below) = self.sky_colors(direction, sun_dir);
        if below {
            return sky;
        }

        let dir = self.rotate_direction(direction.normalize());
        let sun = self.sun_color(direction, sun_dir);

        let nebula_noise = fbm_3d(dir * 3.0, 5, 2.0, 0.5) * 0.5 + 0.5;
        let nebula = Vec3::new(0.15, 0.05, 0.2) * nebula_noise * 0.08;

        let aurora = self.aurora_contribution(dir);

        let star_noise = value_noise_3d(dir * 400.0);
        let stars = if star_noise > 0.97 {
            let brightness = (star_noise - 0.97) / 0.03;
            Vec3::splat(brightness * 0.5)
        } else {
            Vec3::ZERO
        };

        sky + sun + nebula + aurora + stars
    }

    pub fn aurora_contribution(&self, dir: Vec3) -> Vec3 {
        let latitude = dir.y;
        let aurora_band = smoothstep(0.3, 0.5, latitude) * (1.0 - smoothstep(0.6, 0.8, latitude));
        if aurora_band < 0.001 {
            return Vec3::ZERO;
        }
        let wave = (dir.x * 8.0 + dir.z * 4.0).sin() * 0.5 + 0.5;
        let noise = fbm_3d(dir * 6.0, 3, 2.0, 0.5) * 0.5 + 0.5;
        let intensity = aurora_band * wave * noise * 0.12;
        Vec3::new(0.1, 0.8, 0.4) * intensity
    }

    pub fn exposure_for_detail(&self, key_light_luminance: f64) -> f64 {
        let base = if key_light_luminance > 0.001 {
            0.5 / key_light_luminance
        } else {
            4.0
        };
        (base + self.exposure_bias).max(0.01)
    }

    pub fn ambient_color(&self) -> Vec3 {
        (self.zenith_tint + self.horizon_tint) * 0.15
    }
}
