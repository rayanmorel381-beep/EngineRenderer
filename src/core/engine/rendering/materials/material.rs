use crate::core::engine::rendering::raytracing::{Material, Vec3};

pub struct MaterialLibrary;

impl MaterialLibrary {
    pub fn stellar_surface() -> Material {
        Material::new(
            Vec3::new(1.0, 0.74, 0.35),
            0.04,
            0.0,
            0.05,
            Vec3::new(3.8, 2.4, 1.2),
        )
        .with_layers(1.0, 0.18, Vec3::new(0.35, 0.18, 0.08))
        .with_optics(0.22, 0.05, 0.12)
    }

    pub fn rocky_world(base_color: Vec3) -> Material {
        Material::new(base_color, 0.62, 0.03, 0.08, Vec3::ZERO)
            .with_layers(0.92, 0.04, base_color * 0.06)
            .with_optics(0.34, 0.08, 0.02)
    }

    pub fn ocean_world() -> Material {
        Material::new(Vec3::new(0.16, 0.34, 0.72), 0.18, 0.04, 0.24, Vec3::ZERO)
            .with_layers(0.98, 0.42, Vec3::new(0.04, 0.08, 0.16))
            .with_transmission(0.08, 1.33)
            .with_optics(0.26, 0.22, 0.28)
    }

    pub fn icy_world() -> Material {
        Material::new(Vec3::new(0.78, 0.88, 0.96), 0.10, 0.02, 0.28, Vec3::ZERO)
            .with_layers(0.96, 0.32, Vec3::new(0.10, 0.18, 0.24))
            .with_transmission(0.30, 1.31)
            .with_optics(0.42, 0.10, 0.18)
    }

    pub fn metallic_moon() -> Material {
        Material::new(Vec3::new(0.62, 0.60, 0.64), 0.24, 0.85, 0.72, Vec3::ZERO)
            .with_layers(0.88, 0.26, Vec3::new(0.12, 0.12, 0.14))
            .with_optics(0.04, 0.58, 0.10)
    }

    pub fn automotive_paint(base_color: Vec3) -> Material {
        Material::new(base_color, 0.08, 0.22, 0.74, Vec3::ZERO)
            .with_layers(0.97, 0.58, base_color * 0.08 + Vec3::new(0.02, 0.02, 0.02))
            .with_optics(0.10, 0.18, 0.52)
    }

    pub fn window_glass() -> Material {
        Material::new(Vec3::new(0.78, 0.86, 0.96), 0.02, 0.01, 0.14, Vec3::ZERO)
            .with_layers(1.0, 0.24, Vec3::new(0.08, 0.12, 0.18))
            .with_transmission(0.82, 1.48)
            .with_optics(0.04, 0.06, 0.22)
    }

    pub fn architectural_plaster() -> Material {
        Material::new(Vec3::new(0.83, 0.79, 0.72), 0.64, 0.02, 0.06, Vec3::ZERO)
            .with_layers(0.94, 0.05, Vec3::new(0.03, 0.02, 0.01))
            .with_optics(0.18, 0.04, 0.02)
    }

    pub fn roof_tiles() -> Material {
        Material::new(Vec3::new(0.44, 0.16, 0.12), 0.56, 0.06, 0.10, Vec3::ZERO)
            .with_layers(0.96, 0.06, Vec3::new(0.06, 0.02, 0.01))
            .with_optics(0.12, 0.08, 0.04)
    }

    pub fn tree_bark() -> Material {
        Material::new(Vec3::new(0.30, 0.20, 0.12), 0.82, 0.03, 0.04, Vec3::ZERO)
            .with_layers(0.93, 0.02, Vec3::new(0.03, 0.02, 0.01))
            .with_optics(0.14, 0.04, 0.01)
    }

    pub fn tree_foliage() -> Material {
        Material::new(Vec3::new(0.16, 0.42, 0.18), 0.72, 0.02, 0.06, Vec3::ZERO)
            .with_layers(0.98, 0.08, Vec3::new(0.04, 0.10, 0.03))
            .with_optics(0.24, 0.05, 0.04)
    }

    pub fn asphalt() -> Material {
        Material::new(Vec3::new(0.08, 0.09, 0.10), 0.88, 0.01, 0.02, Vec3::ZERO)
            .with_layers(0.96, 0.01, Vec3::new(0.01, 0.01, 0.01))
            .with_optics(0.06, 0.02, 0.01)
    }

    pub fn lush_planet() -> Material {
        Material::new(Vec3::new(0.28, 0.44, 0.24), 0.38, 0.03, 0.18, Vec3::ZERO)
            .with_layers(0.97, 0.18, Vec3::new(0.06, 0.10, 0.14))
            .with_transmission(0.04, 1.22)
            .with_optics(0.34, 0.12, 0.16)
    }

    pub fn solar_corona() -> Material {
        Material::new(
            Vec3::new(1.0, 0.72, 0.30),
            0.05,
            0.0,
            0.08,
            Vec3::new(1.2, 0.85, 0.40),
        )
        .with_layers(1.0, 0.22, Vec3::new(0.34, 0.18, 0.08))
        .with_optics(0.26, 0.08, 0.18)
    }

    pub fn accretion_disk() -> Material {
        Material::new(
            Vec3::new(0.96, 0.54, 0.20),
            0.12,
            0.18,
            0.34,
            Vec3::new(1.2, 0.5, 0.2),
        )
        .with_layers(0.98, 0.30, Vec3::new(0.14, 0.08, 0.22))
        .with_optics(0.10, 0.24, 0.48)
    }

    pub fn event_horizon() -> Material {
        Material::new(Vec3::new(0.01, 0.01, 0.02), 0.02, 0.12, 0.96, Vec3::ZERO)
            .with_layers(1.0, 0.62, Vec3::new(0.02, 0.04, 0.08))
            .with_optics(0.00, 0.74, 0.52)
    }

    pub fn rubber_tire() -> Material {
        Material::new(Vec3::new(0.04, 0.04, 0.05), 0.92, 0.01, 0.02, Vec3::ZERO)
            .with_layers(0.94, 0.02, Vec3::new(0.01, 0.01, 0.01))
            .with_optics(0.04, 0.03, 0.01)
    }
}
