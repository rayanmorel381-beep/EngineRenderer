use std::path::{Path, PathBuf};

use crate::{
    core::engine::config::EngineConfig,
    core::engine::rendering::{environment::procedural::ProceduralEnvironment, raytracing::Vec3},
};

#[derive(Debug, Clone, Copy)]
pub struct SceneEnvironment {
    pub sky_top: Vec3,
    pub sky_bottom: Vec3,
    pub sun_direction: Vec3,
    pub sun_color: Vec3,
    pub sun_intensity: f64,
    pub sun_angular_radius: f64,
    pub exposure: f64,
    pub solar_elevation: f64,
}

#[derive(Debug, Clone)]
pub struct ResourceManager {
    output_path: PathBuf,
    environment: SceneEnvironment,
    surface_detail_scale: f64,
}

impl ResourceManager {
    pub fn from_config(config: &EngineConfig) -> Self {
        let pixel_count = (config.width * config.height) as f64;
        let hd_reference = (1280 * 720) as f64;
        let surface_detail_scale = (pixel_count / hd_reference).sqrt().clamp(1.0, 2.5);
        let procedural_environment = ProceduralEnvironment::cinematic_space();
        let solar_elevation: f64 = 0.48;
        let sun_dir = Vec3::new(
            solar_elevation.cos(),
            solar_elevation.sin(),
            0.0,
        );
        let sky_top = procedural_environment.hdri_probe(Vec3::new(0.0, 1.0, 0.0), sun_dir);
        let sky_bottom = procedural_environment.hdri_probe(
            Vec3::new(0.0, 0.12, 1.0).normalize(),
            sun_dir,
        );

        Self {
            output_path: config.output_path.clone(),
            environment: SceneEnvironment {
                sky_top,
                sky_bottom,
                sun_direction: Vec3::new(-0.65, -0.35, -1.0).normalize(),
                sun_color: procedural_environment.sun_color(Vec3::new(0.0, 1.0, 0.0), sun_dir),
                sun_intensity: 1.45 + surface_detail_scale * 0.14,
                sun_angular_radius: 0.03,
                exposure: procedural_environment.exposure_for_detail(surface_detail_scale),
                solar_elevation,
            },
            surface_detail_scale,
        }
    }

    pub fn output_path(&self) -> &Path {
        &self.output_path
    }

    pub fn environment(&self) -> SceneEnvironment {
        self.environment
    }

    pub fn surface_detail_scale(&self) -> f64 {
        self.surface_detail_scale
    }
}
