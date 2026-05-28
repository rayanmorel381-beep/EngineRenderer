
use crate::core::engine::rendering::raytracing::Vec3;
use crate::core::engine::rendering::utils::{fbm_3d, smoothstep};

#[derive(Debug, Clone, Copy)]
pub struct CloudLayer {
    pub altitude: f64,
    pub thickness: f64,
    pub coverage: f64,
    pub density: f64,
    pub scale: f64,
}

impl CloudLayer {
    pub fn cirrus() -> Self {
        Self {
            altitude: 8000.0,
            thickness: 500.0,
            coverage: 0.35,
            density: 0.15,
            scale: 0.0005,
        }
    }

    pub fn cumulus() -> Self {
        Self {
            altitude: 2000.0,
            thickness: 1500.0,
            coverage: 0.55,
            density: 0.45,
            scale: 0.002,
        }
    }

    pub fn sample_density(&self, point: Vec3) -> f64 {
        let height_in_layer =
            ((point.y - self.altitude) / self.thickness.max(1.0)).clamp(0.0, 1.0);
        let height_profile =
            smoothstep(0.0, 0.3, height_in_layer) * (1.0 - smoothstep(0.7, 1.0, height_in_layer));

        let noise = fbm_3d(point * self.scale, 4, 2.0, 0.5);
        let shape = (noise - (1.0 - self.coverage)).max(0.0) / self.coverage.max(0.01);

        shape * height_profile * self.density
    }

    pub fn cloud_color(
        &self,
        point: Vec3,
        sun_dir: Vec3,
        sun_color: Vec3,
        ambient: Vec3,
    ) -> Vec3 {
        let density = self.sample_density(point);
        if density < 0.001 {
            return Vec3::ZERO;
        }

        let sun_alignment = sun_dir.dot(Vec3::new(0.0, -1.0, 0.0)).max(0.0);
        let beer = (-density * 2.5).exp();
        let powder = 1.0 - (-density * 5.0).exp();

        let light = sun_color * beer * sun_alignment + ambient * 0.35;
        light * powder * density
    }
}
