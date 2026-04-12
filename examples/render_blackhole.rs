use std::error::Error;

use enginerenderer::api::materials::catalog::MaterialCatalog;
use enginerenderer::api::objects::primitives::{icosphere, torus, Vec3, MeshAsset};
use enginerenderer::api::scenes::builder::SceneBuilder;
use enginerenderer::api::types::CameraDesc;
use enginerenderer::api::EngineApi;

fn main() -> Result<(), Box<dyn Error>> {
    let api = EngineApi::new();
    let request = api.request_hd().with_output("output/BLACKHOLE", "blackhole.ppm");
    let cat = MaterialCatalog;

    let sphere = icosphere(4, 1.0);
    // Accretion disk: large torus, thin cross-section
    let disk_inner = torus(1.0, 0.06, 96, 12);
    let disk_mid = torus(1.0, 0.04, 96, 12);
    let disk_outer = torus(1.0, 0.025, 96, 12);

    // Materials
    let event_horizon = cat.custom([0.005, 0.005, 0.008], 0.02, 0.14, 0.96, [0.0, 0.0, 0.0]);
    let hot_accretion = cat.custom([1.0, 0.62, 0.18], 0.06, 0.12, 0.08, [2.8, 1.4, 0.3]);
    let warm_accretion = cat.custom([0.92, 0.44, 0.12], 0.10, 0.08, 0.10, [1.4, 0.6, 0.12]);
    let cool_accretion = cat.custom([0.72, 0.24, 0.08], 0.18, 0.06, 0.12, [0.5, 0.18, 0.04]);
    let star_distant = cat.custom([1.0, 0.86, 0.52], 0.04, 0.0, 0.04, [2.2, 1.6, 0.6]);
    let ice_world = cat.custom([0.72, 0.84, 0.94], 0.14, 0.04, 0.32, [0.0, 0.0, 0.0]);
    let moon_mat = cat.custom([0.52, 0.50, 0.48], 0.52, 0.06, 0.04, [0.0, 0.0, 0.0]);

    let builder = SceneBuilder::new()
        // Event horizon (dark icosphere)
        .add_mesh(&sphere, Vec3::new(0.0, 0.0, 0.0), 2.0, event_horizon)
        // Accretion disk layers (torus meshes at different radii)
        .add_mesh(&disk_inner, Vec3::new(0.0, 0.0, 0.0), 3.2, hot_accretion)
        .add_mesh(&disk_mid, Vec3::new(0.0, 0.0, 0.0), 4.4, warm_accretion)
        .add_mesh(&disk_outer, Vec3::new(0.0, 0.0, 0.0), 5.6, cool_accretion)
        // Distant star
        .add_mesh(&sphere, Vec3::new(8.0, 4.2, -12.0), 0.85, star_distant)
        // Nearby ice world being pulled in
        .add_mesh(&sphere, Vec3::new(-7.0, -1.0, -8.0), 1.1, ice_world)
        // Moon near ice world
        .add_mesh(&sphere, Vec3::new(-6.0, -0.4, -7.0), 0.22, moon_mat)
        // Debris asteroids near the black hole
        .add_mesh(&MeshAsset::procedural_asteroid("deb1", 0.15, 2), Vec3::new(3.6, 1.2, 2.4), 1.0, moon_mat)
        .add_mesh(&MeshAsset::procedural_asteroid("deb2", 0.10, 2), Vec3::new(-2.8, -0.8, 3.2), 1.0, moon_mat)
        .add_mesh(&MeshAsset::procedural_asteroid("deb3", 0.08, 1), Vec3::new(1.4, 2.0, -3.6), 1.0, moon_mat)
        // Lighting — subtle, mostly from accretion disk emission
        .add_area_light([0.0, 5.0, 0.0], [0.92, 0.52, 0.22], 0.6, [3.0, 3.0])
        .sun_direction([0.24, -0.30, -1.0])
        .sun_intensity(0.3)
        .sky([0.001, 0.001, 0.004], [0.0, 0.0, 0.001])
        .exposure(1.1)
        .with_vacuum()
        .with_camera(CameraDesc {
            eye: [8.0, 5.0, 10.0],
            target: [0.0, 0.0, 0.0],
            fov_degrees: 36.0,
            aperture: 0.001,
        });

    let result = api.render(builder, &request)?;
    eprintln!(
        "blackhole: {}x{} {:.0}ms objs={} tris={} -> {}",
        result.width, result.height, result.duration_ms,
        result.object_count, result.triangle_count,
        result.output_path.display(),
    );
    Ok(())
}
