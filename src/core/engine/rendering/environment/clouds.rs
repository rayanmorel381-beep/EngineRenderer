//! Procedural cloud layer with fBm density and Beer-powder lighting.
//!
//! [`CloudLayer`] models a horizontal slab of clouds parameterised by
//! altitude, thickness, coverage, and noise scale.

use crate::core::engine::rendering::raytracing::Vec3;
use crate::core::engine::rendering::utils::{fbm_3d, smoothstep};

/// A horizontal cloud slab defined by altitude, thickness, and coverage.
///
/// Two convenience presets are provided: [`cirrus`](Self::cirrus) (thin,
/// high-altitude wisps) and [`cumulus`](Self::cumulus) (thick, low-level
/// puffy clouds).
#[derive(Debug, Clone, Copy)]
pub struct CloudLayer {
    /// Centre altitude of the layer (metres above sea level).
    pub altitude: f64,
    /// Vertical extent of the layer (metres).
    pub thickness: f64,
    /// Coverage fraction `[0, 1]`: `0` = clear sky, `1` = overcast.
    pub coverage: f64,
    /// Peak density (optical thickness multiplier).
    pub density: f64,
    /// Spatial noise frequency (smaller = larger features).
    pub scale: f64,
}

impl CloudLayer {
    /// High-altitude, thin streaky clouds.
    pub fn cirrus() -> Self {
        Self {
            altitude: 8000.0,
            thickness: 500.0,
            coverage: 0.35,
            density: 0.15,
            scale: 0.0005,
        }
    }

    /// Low, thick, puffy clouds.
    pub fn cumulus() -> Self {
        Self {
            altitude: 2000.0,
            thickness: 1500.0,
            coverage: 0.55,
            density: 0.45,
            scale: 0.002,
        }
    }

    /// Samples cloud density at a world-space `point` using fBm noise
    /// and a bell-shaped height profile.
    ///
    /// Returns `0.0` for points outside the layer or where coverage
    /// carves away the noise.
    pub fn sample_density(&self, point: Vec3) -> f64 {
        let height_in_layer =
            ((point.y - self.altitude) / self.thickness.max(1.0)).clamp(0.0, 1.0);
        let height_profile =
            smoothstep(0.0, 0.3, height_in_layer) * (1.0 - smoothstep(0.7, 1.0, height_in_layer));

        let noise = fbm_3d(point * self.scale, 4, 2.0, 0.5);
        let shape = (noise - (1.0 - self.coverage)).max(0.0) / self.coverage.max(0.01);

        shape * height_profile * self.density
    }

    /// Computes the lit colour of a cloud point using a Beer-powder
    /// approximation.
    ///
    /// * `sun_dir`   – normalised direction **toward** the sun.
    /// * `sun_color` – sun irradiance (RGB).
    /// * `ambient`   – sky ambient colour for indirect lighting.
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
