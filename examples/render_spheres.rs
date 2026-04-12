use std::error::Error;

use enginerenderer::api::materials::catalog::MaterialCatalog;
use enginerenderer::api::objects::primitives::Vec3;
use enginerenderer::api::objects::SceneObject;
use enginerenderer::api::scenes::builder::SceneBuilder;
use enginerenderer::api::types::CameraDesc;
use enginerenderer::api::EngineApi;

fn main() -> Result<(), Box<dyn Error>> {
    let api = EngineApi::new();
    let cat = MaterialCatalog;

    let request = api
        .request_hd()
        .with_output("output/SPHERES", "spheres.ppm");

    let ground = SceneObject::ground_plane(-0.5, "architectural_plaster");

    let builder = SceneBuilder::new()
        .add_object(ground)
        // Row of spheres with different materials
        .add_sphere(Vec3::new(-3.0, 0.0, 0.0), 0.5, cat.custom([0.9, 0.2, 0.15], 0.04, 0.0, 0.04, [0.0, 0.0, 0.0]))    // matte red
        .add_sphere(Vec3::new(-1.5, 0.0, 0.0), 0.5, cat.custom([0.95, 0.95, 0.95], 0.02, 1.0, 0.95, [0.0, 0.0, 0.0]))   // chrome mirror
        .add_sphere(Vec3::new(0.0, 0.0, 0.0), 0.5, cat.by_name("window_glass"))                                            // glass
        .add_sphere(Vec3::new(1.5, 0.0, 0.0), 0.5, cat.custom([0.04, 0.28, 0.72], 0.08, 0.0, 0.04, [0.0, 0.0, 0.0]))    // blue glossy
        .add_sphere(Vec3::new(3.0, 0.0, 0.0), 0.5, cat.custom([1.0, 0.76, 0.34], 0.28, 0.95, 0.82, [0.0, 0.0, 0.0]))    // gold metal
        // Back row
        .add_sphere(Vec3::new(-2.25, 0.0, -1.8), 0.5, cat.by_name("ocean_world"))
        .add_sphere(Vec3::new(-0.75, 0.0, -1.8), 0.5, cat.by_name("icy_world"))
        .add_sphere(Vec3::new(0.75, 0.0, -1.8), 0.5, cat.by_name("metallic_moon"))
        .add_sphere(Vec3::new(2.25, 0.0, -1.8), 0.5, cat.by_name("lush_planet"))
        // Lighting
        .add_area_light([-4.0, 4.0, 4.0], [1.0, 0.95, 0.88], 1.8, [2.0, 2.0])
        .add_area_light([4.0, 3.0, 2.0], [0.7, 0.8, 1.0], 1.2, [1.5, 1.5])
        .sun_direction([-0.5, -0.6, -0.8])
        .sun_intensity(1.5)
        .sky([0.4, 0.5, 0.7], [0.7, 0.75, 0.85])
        .exposure(1.3)
        .with_vacuum()
        .with_camera(CameraDesc {
            eye: [0.0, 3.5, 7.0],
            target: [0.0, -0.1, -0.5],
            fov_degrees: 38.0,
            aperture: 0.003,
        });

    let result = api.render(builder, &request)?;
    eprintln!(
        "spheres: {}x{} {:.0}ms objs={} tris={} -> {}",
        result.width, result.height, result.duration_ms,
        result.object_count, result.triangle_count,
        result.output_path.display(),
    );
    Ok(())
}
