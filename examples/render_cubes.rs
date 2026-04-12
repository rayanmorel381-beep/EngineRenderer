use std::error::Error;

use enginerenderer::api::materials::catalog::MaterialCatalog;
use enginerenderer::api::objects::primitives::{unit_cube, Vec3, MeshAsset};
use enginerenderer::api::objects::SceneObject;
use enginerenderer::api::scenes::builder::SceneBuilder;
use enginerenderer::api::types::CameraDesc;
use enginerenderer::api::EngineApi;

fn main() -> Result<(), Box<dyn Error>> {
    let api = EngineApi::new();
    let request = api.request_hd().with_output("output/CUBES", "cubes.ppm");
    let cat = MaterialCatalog;

    let cube_mesh = unit_cube();
    let ground_mesh = enginerenderer::api::objects::primitives::ground_plane(4, 12.0);
    let ground_mat = cat.custom([0.12, 0.12, 0.14], 0.72, 0.02, 0.04, [0.0, 0.0, 0.0]);

    let purple_glass = cat.custom([0.76, 0.30, 0.88], 0.04, 0.08, 0.78, [0.0, 0.0, 0.0]);
    let blue_metal = cat.custom([0.28, 0.46, 0.82], 0.08, 0.88, 0.04, [0.0, 0.0, 0.0]);
    let chrome = cat.custom([0.92, 0.92, 0.94], 0.02, 0.98, 0.04, [0.0, 0.0, 0.0]);
    let gold = cat.custom([1.0, 0.76, 0.34], 0.12, 0.94, 0.04, [0.0, 0.0, 0.0]);
    let emissive_warm = cat.custom([1.0, 0.82, 0.44], 0.04, 0.0, 0.04, [1.4, 0.96, 0.36]);

    let builder = SceneBuilder::new()
        .add_mesh(&ground_mesh, Vec3::new(0.0, -0.5, 0.0), 1.0, ground_mat)
        .add_mesh(&cube_mesh, Vec3::new(-2.5, 0.3, 0.0), 1.6, purple_glass)
        .add_mesh(&cube_mesh, Vec3::new(0.0, 0.5, -0.4), 2.0, blue_metal)
        .add_mesh(&cube_mesh, Vec3::new(2.8, 0.15, 0.6), 1.0, chrome)
        .add_mesh(&cube_mesh, Vec3::new(0.0, 1.8, -0.4), 0.5, gold)
        .add_sphere(Vec3::new(-4.6, 3.2, -4.0), 0.6, emissive_warm)
        .add_area_light([-3.0, 5.0, 4.0], [1.0, 0.96, 0.92], 2.4, [3.0, 3.0])
        .add_area_light([4.0, 3.5, 2.0], [0.66, 0.80, 1.0], 1.6, [2.0, 2.0])
        .sun_direction([-0.44, -0.70, -0.56])
        .sun_intensity(1.4)
        .sky([0.10, 0.12, 0.18], [0.24, 0.26, 0.32])
        .exposure(1.0)
        .with_vacuum()
        .with_camera(CameraDesc {
            eye: [5.5, 3.2, 7.0],
            target: [0.0, 0.5, 0.0],
            fov_degrees: 34.0,
            aperture: 0.003,
        });

    let result = api.render(builder, &request)?;
    eprintln!(
        "cubes: {}x{} {:.0}ms objs={} tris={} -> {}",
        result.width, result.height, result.duration_ms,
        result.object_count, result.triangle_count,
        result.output_path.display(),
    );
    Ok(())
}
