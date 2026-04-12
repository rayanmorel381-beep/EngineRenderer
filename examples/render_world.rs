use std::error::Error;

use enginerenderer::api::materials::catalog::MaterialCatalog;
use enginerenderer::api::objects::primitives::{icosphere, torus, Vec3, MeshAsset};
use enginerenderer::api::scenes::builder::SceneBuilder;
use enginerenderer::api::types::CameraDesc;
use enginerenderer::api::EngineApi;

fn main() -> Result<(), Box<dyn Error>> {
    let api = EngineApi::new();
    let request = api.request_hd().with_output("output/WORLD", "world.ppm");
    let cat = MaterialCatalog;

    // Icosphere meshes for planets (smooth at subdivisions=4)
    let planet_hi = icosphere(4, 1.0);
    let planet_lo = icosphere(3, 1.0);
    // Torus for planet rings
    let ring = torus(1.0, 0.015, 64, 8);

    // Materials
    let star_mat = cat.custom([1.0, 0.82, 0.36], 0.04, 0.0, 0.04, [3.0, 2.2, 0.8]);
    let earth_mat = cat.custom([0.14, 0.38, 0.22], 0.32, 0.02, 0.04, [0.0, 0.0, 0.0]);
    let ocean_mat = cat.custom([0.08, 0.22, 0.52], 0.08, 0.04, 0.28, [0.0, 0.0, 0.0]);
    let ice_mat = cat.custom([0.72, 0.84, 0.94], 0.14, 0.04, 0.32, [0.0, 0.0, 0.0]);
    let rocky_mat = cat.custom([0.42, 0.36, 0.28], 0.56, 0.06, 0.04, [0.0, 0.0, 0.0]);
    let moon_mat = cat.custom([0.62, 0.60, 0.58], 0.48, 0.08, 0.04, [0.0, 0.0, 0.0]);
    let ring_mat = cat.custom([0.56, 0.52, 0.44], 0.34, 0.12, 0.04, [0.0, 0.0, 0.0]);
    let gas_giant = cat.custom([0.72, 0.54, 0.34], 0.22, 0.04, 0.04, [0.0, 0.0, 0.0]);

    let builder = SceneBuilder::new()
        // Star (emissive icosphere)
        .add_mesh(&planet_hi, Vec3::new(-6.0, 1.5, -8.0), 2.0, star_mat)
        // Earth-like planet
        .add_mesh(&planet_hi, Vec3::new(0.0, 0.0, 0.0), 1.4, earth_mat)
        // Ocean planet
        .add_mesh(&planet_hi, Vec3::new(4.5, 0.3, -2.0), 1.0, ocean_mat)
        // Gas giant with ring
        .add_mesh(&planet_hi, Vec3::new(-3.8, -0.2, 3.0), 1.8, gas_giant)
        .add_mesh(&ring, Vec3::new(-3.8, -0.2, 3.0), 2.8, ring_mat)
        // Ice world
        .add_mesh(&planet_lo, Vec3::new(7.0, 0.8, -5.0), 0.7, ice_mat)
        // Rocky planet
        .add_mesh(&planet_lo, Vec3::new(2.2, 0.6, 4.0), 0.5, rocky_mat)
        // Moons (small icospheres)
        .add_mesh(&planet_lo, Vec3::new(1.8, 0.6, 0.4), 0.18, moon_mat)
        .add_mesh(&planet_lo, Vec3::new(5.2, 0.8, -1.2), 0.14, moon_mat)
        .add_mesh(&planet_lo, Vec3::new(-2.4, 0.5, 3.8), 0.22, moon_mat)
        // Distant asteroid (procedural)
        .add_mesh(&MeshAsset::procedural_asteroid("ast1", 0.3, 3), Vec3::new(8.5, 2.0, -7.0), 1.0, rocky_mat)
        .add_mesh(&MeshAsset::procedural_asteroid("ast2", 0.2, 2), Vec3::new(-8.0, 1.2, -4.0), 1.0, moon_mat)
        // Lighting
        .add_area_light([-6.0, 4.0, -8.0], [1.0, 0.88, 0.60], 1.2, [2.0, 2.0])
        .sun_direction([-0.52, -0.40, -1.0])
        .sun_intensity(1.2)
        .sky([0.003, 0.005, 0.018], [0.0006, 0.0006, 0.002])
        .exposure(0.95)
        .with_vacuum()
        .with_camera(CameraDesc {
            eye: [10.0, 4.0, 10.0],
            target: [0.0, 0.2, 0.0],
            fov_degrees: 30.0,
            aperture: 0.001,
        });

    let result = api.render(builder, &request)?;
    eprintln!(
        "world: {}x{} {:.0}ms objs={} tris={} -> {}",
        result.width, result.height, result.duration_ms,
        result.object_count, result.triangle_count,
        result.output_path.display(),
    );
    Ok(())
}
