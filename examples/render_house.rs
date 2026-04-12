use std::error::Error;

use enginerenderer::api::materials::catalog::MaterialCatalog;
use enginerenderer::api::objects::primitives::{unit_cube, icosphere, Vec3, MeshAsset};
use enginerenderer::api::scenes::builder::SceneBuilder;
use enginerenderer::api::types::CameraDesc;
use enginerenderer::api::EngineApi;

fn main() -> Result<(), Box<dyn Error>> {
    let api = EngineApi::new();
    let request = api.request_hd().with_output("output/HOUSES", "house.ppm");
    let cat = MaterialCatalog;

    let cube = unit_cube();
    let sphere_mesh = icosphere(3, 1.0);
    let ground_mesh = enginerenderer::api::objects::primitives::ground_plane(6, 16.0);

    // Materials
    let stone_wall = cat.custom([0.78, 0.72, 0.64], 0.62, 0.02, 0.04, [0.0, 0.0, 0.0]);
    let dark_wood = cat.custom([0.24, 0.14, 0.08], 0.48, 0.02, 0.04, [0.0, 0.0, 0.0]);
    let clay_roof = cat.custom([0.72, 0.32, 0.18], 0.54, 0.02, 0.04, [0.0, 0.0, 0.0]);
    let glass_window = cat.custom([0.62, 0.78, 0.94], 0.03, 0.04, 0.82, [0.0, 0.0, 0.0]);
    let ground_mat = cat.custom([0.18, 0.28, 0.12], 0.82, 0.02, 0.04, [0.0, 0.0, 0.0]);
    let foliage = cat.custom([0.14, 0.38, 0.12], 0.68, 0.02, 0.04, [0.0, 0.0, 0.0]);
    let bark = cat.custom([0.22, 0.14, 0.08], 0.72, 0.02, 0.04, [0.0, 0.0, 0.0]);
    let warm_light = cat.custom([1.0, 0.88, 0.62], 0.04, 0.0, 0.04, [1.8, 1.2, 0.5]);

    let builder = SceneBuilder::new()
        // Ground
        .add_mesh(&ground_mesh, Vec3::new(0.0, -0.02, 0.0), 1.0, ground_mat)
        // Main house body (wide cube)
        .add_mesh(&cube, Vec3::new(0.0, 0.8, 0.0), 3.2, stone_wall)
        // Roof (flattened icosphere dome)
        .add_mesh(&sphere_mesh, Vec3::new(0.0, 2.2, 0.0), 2.2, clay_roof)
        // Door (thin dark cube)
        .add_mesh(&cube, Vec3::new(0.0, 0.5, 1.62), 0.6, dark_wood)
        // Windows (glass cubes on front face)
        .add_mesh(&cube, Vec3::new(-0.9, 1.1, 1.62), 0.4, glass_window)
        .add_mesh(&cube, Vec3::new(0.9, 1.1, 1.62), 0.4, glass_window)
        // Chimney
        .add_mesh(&cube, Vec3::new(1.1, 2.8, -0.3), 0.4, stone_wall)
        // Trees (trunk = tall cube, crown = icosphere)
        .add_mesh(&cube, Vec3::new(-4.0, 0.8, 1.0), 0.3, bark)
        .add_mesh(&sphere_mesh, Vec3::new(-4.0, 2.2, 1.0), 1.2, foliage)
        .add_mesh(&cube, Vec3::new(4.2, 1.0, -0.6), 0.35, bark)
        .add_mesh(&sphere_mesh, Vec3::new(4.2, 2.6, -0.6), 1.5, foliage)
        .add_mesh(&cube, Vec3::new(-3.0, 0.6, -2.8), 0.25, bark)
        .add_mesh(&sphere_mesh, Vec3::new(-3.0, 1.8, -2.8), 1.0, foliage)
        // Warm interior light visible through windows
        .add_sphere(Vec3::new(0.0, 1.0, 1.2), 0.15, warm_light)
        // Lighting
        .add_area_light([-5.0, 6.0, 6.0], [1.0, 0.94, 0.86], 2.2, [3.0, 3.0])
        .add_area_light([5.0, 4.0, 3.0], [0.66, 0.80, 1.0], 1.0, [1.6, 1.6])
        .sun_direction([-0.55, -0.64, -0.52])
        .sun_intensity(1.5)
        .sky([0.32, 0.44, 0.68], [0.62, 0.68, 0.78])
        .exposure(1.1)
        .with_vacuum()
        .with_camera(CameraDesc {
            eye: [7.0, 4.0, 8.0],
            target: [0.0, 1.0, 0.0],
            fov_degrees: 36.0,
            aperture: 0.004,
        });

    let result = api.render(builder, &request)?;
    eprintln!(
        "house: {}x{} {:.0}ms objs={} tris={} -> {}",
        result.width, result.height, result.duration_ms,
        result.object_count, result.triangle_count,
        result.output_path.display(),
    );
    Ok(())
}
