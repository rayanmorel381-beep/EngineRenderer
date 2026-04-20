use crate::api::scenes::builder::SceneBuilder;
use crate::api::types::CameraDesc;

// ===========================================================================
// Pre-configured scene builders — ready to customise or render directly.
// ===========================================================================

// ---------------------------------------------------------------------------
// Empty / minimal
// ---------------------------------------------------------------------------

pub fn empty() -> SceneBuilder {
    SceneBuilder::new()
        .with_vacuum()
        .sun_intensity(0.0)
        .sky([0.0, 0.0, 0.0], [0.0, 0.0, 0.0])
        .exposure(1.0)
}

pub fn lit_void(direction: [f64; 3], color: [f64; 3], intensity: f64) -> SceneBuilder {
    empty()
        .sun_direction(direction)
        .sun_color(color)
        .sun_intensity(intensity)
}

// ---------------------------------------------------------------------------
// Studio / controlled lighting
// ---------------------------------------------------------------------------

pub fn studio(eye: [f64; 3], target: [f64; 3]) -> SceneBuilder {
    SceneBuilder::new()
        .with_vacuum()
        .sun_direction([-0.5, -0.8, -0.3])
        .sun_color([1.0, 0.98, 0.95])
        .sun_intensity(1.2)
        .add_area_light(
            [eye[0] + 5.0, eye[1] + 4.0, eye[2] - 3.0],
            [0.9, 0.92, 1.0],
            0.8,
            [3.0, 3.0],
        )
        .add_area_light(
            [eye[0] - 4.0, eye[1] + 1.0, eye[2] + 2.0],
            [0.6, 0.65, 0.8],
            0.4,
            [2.0, 2.0],
        )
        .sky([0.02, 0.02, 0.03], [0.005, 0.005, 0.008])
        .exposure(1.3)
        .camera_position(eye, target)
}

pub fn studio_high_key(eye: [f64; 3], target: [f64; 3]) -> SceneBuilder {
    studio(eye, target)
        .sun_intensity(2.0)
        .exposure(1.6)
}

pub fn studio_low_key(eye: [f64; 3], target: [f64; 3]) -> SceneBuilder {
    studio(eye, target)
        .sun_intensity(0.6)
        .exposure(0.9)
}

// ---------------------------------------------------------------------------
// Outdoor / natural environments
// ---------------------------------------------------------------------------

pub fn outdoor_daylight(
    sun_dir: [f64; 3],
    sun_color: [f64; 3],
    sun_intensity: f64,
    sky_zenith: [f64; 3],
    sky_horizon: [f64; 3],
) -> SceneBuilder {
    SceneBuilder::new()
        .with_vacuum()
        .sun_direction(sun_dir)
        .sun_color(sun_color)
        .sun_intensity(sun_intensity)
        .sky(sky_zenith, sky_horizon)
        .exposure(1.4)
}

pub fn golden_hour() -> SceneBuilder {
    outdoor_daylight(
        [-0.15, -0.12, -1.0],
        [1.0, 0.78, 0.48],
        1.6,
        [0.12, 0.18, 0.42],
        [0.52, 0.32, 0.18],
    )
}

pub fn night(moon_dir: [f64; 3]) -> SceneBuilder {
    SceneBuilder::new()
        .with_vacuum()
        .sun_direction(moon_dir)
        .sun_color([0.6, 0.65, 0.8])
        .sun_intensity(0.15)
        .sky([0.001, 0.001, 0.004], [0.0, 0.0, 0.001])
        .exposure(2.5)
}

// ---------------------------------------------------------------------------
// Volumetric / atmospheric
// ---------------------------------------------------------------------------

pub fn volumetric(
    density: f64,
    anisotropy: f64,
    height_falloff: f64,
    color: [f64; 3],
    emission: [f64; 3],
) -> SceneBuilder {
    use crate::core::engine::rendering::effects::volumetric_effects::medium::VolumetricMedium;

    let medium = VolumetricMedium {
        density,
        anisotropy,
        height_falloff,
        color: crate::core::engine::rendering::raytracing::Vec3::new(color[0], color[1], color[2]),
        emission: crate::core::engine::rendering::raytracing::Vec3::new(emission[0], emission[1], emission[2]),
        absorption: 0.0,
        noise_scale: 1.0,
        noise_octaves: 4,
        wind_direction: crate::core::engine::rendering::raytracing::Vec3::ZERO,
        wind_speed: 0.0,
    };

    SceneBuilder::new()
        .with_volume(medium)
        .sun_direction([-0.5, -0.4, -1.0])
        .sun_intensity(1.2)
        .exposure(1.4)
}

pub fn foggy() -> SceneBuilder {
    volumetric(0.06, 0.2, 0.08, [0.7, 0.72, 0.75], [0.0, 0.0, 0.0])
}

// ---------------------------------------------------------------------------
// Space / celestial
// ---------------------------------------------------------------------------

pub fn deep_space() -> SceneBuilder {
    SceneBuilder::new()
        .with_vacuum()
        .sun_direction([-0.65, -0.35, -1.0])
        .sun_color([1.0, 0.96, 0.90])
        .sun_intensity(1.5)
        .sky([0.002, 0.003, 0.008], [0.0, 0.0, 0.001])
        .exposure(1.45)
}

pub fn nebula(
    density: f64,
    color: [f64; 3],
    emission: [f64; 3],
) -> SceneBuilder {
    volumetric(density, 0.42, 0.14, color, emission)
        .sky([0.015, 0.020, 0.050], [0.001, 0.001, 0.006])
        .exposure(1.45)
}

// ---------------------------------------------------------------------------
// Testing / debug
// ---------------------------------------------------------------------------

pub fn test_single_sphere() -> SceneBuilder {
    use crate::core::engine::rendering::raytracing::{Material, Vec3};

    let mat = Material::new(Vec3::new(0.8, 0.8, 0.8), 0.5, 0.0, 0.04, Vec3::ZERO);
    SceneBuilder::new()
        .add_sphere(Vec3::new(0.0, 0.0, 0.0), 1.0, mat)
        .sun_direction([-0.5, -0.8, -0.3])
        .sun_intensity(1.0)
        .camera_position([0.0, 0.0, 5.0], [0.0, 0.0, 0.0])
}

pub fn material_lineup(materials: Vec<crate::core::engine::rendering::raytracing::Material>, spacing: f64) -> SceneBuilder {
    use crate::core::engine::rendering::raytracing::Vec3;

    let count = materials.len();
    let total_width = (count as f64 - 1.0) * spacing;
    let mut builder = studio(
        [0.0, 2.0, total_width * 0.6 + 4.0],
        [0.0, 0.0, 0.0],
    );
    for (i, mat) in materials.into_iter().enumerate() {
        let x = -total_width / 2.0 + i as f64 * spacing;
        builder = builder.add_sphere(Vec3::new(x, 0.0, 0.0), 0.8, mat);
    }
    builder
}

pub fn turntable(target: [f64; 3], distance: f64, height: f64, angle_degrees: f64) -> CameraDesc {
    let angle = angle_degrees.to_radians();
    CameraDesc {
        eye: [
            target[0] + distance * angle.cos(),
            target[1] + height,
            target[2] + distance * angle.sin(),
        ],
        target,
        fov_degrees: 45.0,
        aperture: 0.0,
    }
}
