use std::error::Error;

use enginerenderer::api::materials::catalog::MaterialCatalog;
use enginerenderer::api::objects::primitives::{unit_cube, torus, icosphere, Vec3, MeshAsset};
use enginerenderer::api::scenes::builder::SceneBuilder;
use enginerenderer::api::types::CameraDesc;
use enginerenderer::api::EngineApi;

fn main() -> Result<(), Box<dyn Error>> {
    let api = EngineApi::new();
    let request = api.request_hd().with_output("output/CAR", "car.ppm");
    let cat = MaterialCatalog;

    let cube = unit_cube();
    let wheel = torus(0.32, 0.10, 24, 12);
    let ground_mesh = enginerenderer::api::objects::primitives::ground_plane(4, 10.0);

    // Materials
    let body_red = cat.custom([0.82, 0.12, 0.10], 0.06, 0.32, 0.04, [0.0, 0.0, 0.0]);
    let windshield = cat.custom([0.56, 0.72, 0.92], 0.02, 0.06, 0.86, [0.0, 0.0, 0.0]);
    let tire_rubber = cat.custom([0.04, 0.04, 0.05], 0.88, 0.01, 0.02, [0.0, 0.0, 0.0]);
    let chrome_rim = cat.custom([0.90, 0.92, 0.94], 0.03, 0.96, 0.04, [0.0, 0.0, 0.0]);
    let headlight = cat.custom([1.0, 0.96, 0.88], 0.04, 0.0, 0.04, [2.2, 2.0, 1.6]);
    let tail_light = cat.custom([1.0, 0.12, 0.06], 0.04, 0.0, 0.04, [1.6, 0.08, 0.04]);
    let ground_mat = cat.custom([0.10, 0.10, 0.12], 0.64, 0.02, 0.04, [0.0, 0.0, 0.0]);
    let underbody = cat.custom([0.06, 0.06, 0.08], 0.52, 0.12, 0.04, [0.0, 0.0, 0.0]);

    let builder = SceneBuilder::new()
        .add_mesh(&ground_mesh, Vec3::new(0.0, -0.02, 0.0), 1.0, ground_mat)
        // Main body (long, low cube)
        .add_mesh(&cube, Vec3::new(0.0, 0.42, 0.0), 2.8, body_red)
        // Cabin / roof (smaller cube on top)
        .add_mesh(&cube, Vec3::new(-0.2, 0.88, 0.0), 1.4, body_red)
        // Windshield (angled glass cube)
        .add_mesh(&cube, Vec3::new(0.6, 0.82, 0.0), 0.5, windshield)
        // Rear window
        .add_mesh(&cube, Vec3::new(-0.9, 0.82, 0.0), 0.4, windshield)
        // Underbody
        .add_mesh(&cube, Vec3::new(0.0, 0.14, 0.0), 2.6, underbody)
        // Wheels (torus meshes)
        .add_mesh(&wheel, Vec3::new(-0.9, 0.22, 0.72), 1.0, tire_rubber)
        .add_mesh(&wheel, Vec3::new(-0.9, 0.22, -0.72), 1.0, tire_rubber)
        .add_mesh(&wheel, Vec3::new(0.8, 0.22, 0.72), 1.0, tire_rubber)
        .add_mesh(&wheel, Vec3::new(0.8, 0.22, -0.72), 1.0, tire_rubber)
        // Chrome hub caps (small icospheres at wheel centers)
        .add_sphere(Vec3::new(-0.9, 0.22, 0.72), 0.06, chrome_rim)
        .add_sphere(Vec3::new(-0.9, 0.22, -0.72), 0.06, chrome_rim)
        .add_sphere(Vec3::new(0.8, 0.22, 0.72), 0.06, chrome_rim)
        .add_sphere(Vec3::new(0.8, 0.22, -0.72), 0.06, chrome_rim)
        // Headlights
        .add_sphere(Vec3::new(1.42, 0.40, 0.42), 0.08, headlight)
        .add_sphere(Vec3::new(1.42, 0.40, -0.42), 0.08, headlight)
        // Tail lights
        .add_sphere(Vec3::new(-1.42, 0.40, 0.38), 0.07, tail_light)
        .add_sphere(Vec3::new(-1.42, 0.40, -0.38), 0.07, tail_light)
        // Lighting
        .add_area_light([-4.0, 5.0, 5.0], [1.0, 0.96, 0.90], 2.4, [3.0, 3.0])
        .add_area_light([5.0, 3.5, 3.0], [0.80, 0.86, 1.0], 1.8, [2.0, 2.0])
        .add_area_light([0.0, 6.0, -2.0], [1.0, 1.0, 1.0], 1.2, [4.0, 2.0])
        .sun_direction([-0.3, -0.8, -0.5])
        .sun_intensity(1.4)
        .sky([0.08, 0.10, 0.14], [0.20, 0.22, 0.28])
        .exposure(1.0)
        .with_vacuum()
        .with_camera(CameraDesc {
            eye: [4.2, 1.8, 4.8],
            target: [0.0, 0.4, 0.0],
            fov_degrees: 34.0,
            aperture: 0.005,
        });

    let result = api.render(builder, &request)?;
    eprintln!(
        "car: {}x{} {:.0}ms objs={} tris={} -> {}",
        result.width, result.height, result.duration_ms,
        result.object_count, result.triangle_count,
        result.output_path.display(),
    );
    Ok(())
}
