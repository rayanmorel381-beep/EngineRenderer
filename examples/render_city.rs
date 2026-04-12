use std::error::Error;

use enginerenderer::api::materials::catalog::MaterialCatalog;
use enginerenderer::api::objects::primitives::{unit_cube, torus, icosphere, Vec3, MeshAsset};
use enginerenderer::api::scenes::builder::SceneBuilder;
use enginerenderer::api::types::CameraDesc;
use enginerenderer::api::EngineApi;

fn add_tower(
    builder: SceneBuilder,
    cube: &MeshAsset,
    pos: [f64; 3],
    floors: usize,
    width: f64,
    rgb: [f64; 3],
) -> SceneBuilder {
    let cat = MaterialCatalog;
    let facade = cat.custom(rgb, 0.12, 0.56, 0.04, [0.0, 0.0, 0.0]);
    let glass = cat.custom([0.62, 0.78, 0.94], 0.03, 0.08, 0.82, [0.0, 0.0, 0.0]);
    let antenna_mat = cat.custom([0.86, 0.88, 0.92], 0.06, 0.92, 0.04, [0.0, 0.0, 0.0]);

    let mut b = builder;
    let floor_h = width * 0.7;
    for i in 0..floors {
        let y = pos[1] + i as f64 * floor_h + floor_h * 0.5;
        // Main floor block
        b = b.add_mesh(cube, Vec3::new(pos[0], y, pos[2]), width, facade);
        // Glass strip on each floor
        b = b.add_mesh(cube, Vec3::new(pos[0], y + floor_h * 0.3, pos[2] + width * 0.51), width * 0.9, glass);
    }
    // Antenna on top
    let top_y = pos[1] + floors as f64 * floor_h + 0.3;
    b = b.add_mesh(cube, Vec3::new(pos[0], top_y, pos[2]), width * 0.08, antenna_mat);
    b
}

fn main() -> Result<(), Box<dyn Error>> {
    let api = EngineApi::new();
    let request = api.request_hd().with_output("output/CITY", "city.ppm");
    let cat = MaterialCatalog;

    let cube = unit_cube();
    let ground_mesh = enginerenderer::api::objects::primitives::ground_plane(6, 20.0);
    let road_mat = cat.custom([0.08, 0.08, 0.10], 0.68, 0.02, 0.04, [0.0, 0.0, 0.0]);

    let mut builder = SceneBuilder::new()
        .add_mesh(&ground_mesh, Vec3::new(0.0, -0.02, 0.0), 1.0, road_mat);

    // Skyline of towers
    let towers: &[(f64, f64, usize, f64, [f64; 3])] = &[
        (-6.0, -3.0, 8, 1.2, [0.72, 0.76, 0.84]),
        (-3.2, 1.0, 6, 1.0, [0.58, 0.68, 0.90]),
        (-0.8, -2.0, 10, 1.4, [0.78, 0.78, 0.84]),
        (2.0, 1.2, 7, 1.1, [0.64, 0.76, 0.92]),
        (4.8, -1.4, 12, 1.5, [0.76, 0.80, 0.88]),
        (7.6, 2.0, 5, 0.9, [0.56, 0.70, 0.88]),
    ];

    for &(x, z, floors, width, rgb) in towers {
        builder = add_tower(builder, &cube, [x, 0.0, z], floors, width, rgb);
    }

    // Street lamps (small emissive spheres along the road)
    let lamp_emit = cat.custom([1.0, 0.90, 0.66], 0.04, 0.0, 0.04, [1.2, 0.92, 0.48]);
    for i in -6i32..=6 {
        let x = i as f64 * 1.8;
        builder = builder.add_sphere(Vec3::new(x, 0.6, 4.0), 0.08, lamp_emit);
    }

    builder = builder
        .add_area_light([-8.0, 12.0, 6.0], [1.0, 0.94, 0.85], 2.6, [5.0, 5.0])
        .add_area_light([8.0, 10.0, -3.0], [0.72, 0.82, 1.0], 1.8, [3.0, 3.0])
        .sun_direction([-0.36, -0.70, -0.34])
        .sun_intensity(1.4)
        .sky([0.08, 0.10, 0.16], [0.22, 0.24, 0.30])
        .exposure(0.96)
        .with_vacuum()
        .with_camera(CameraDesc {
            eye: [0.0, 6.0, 16.0],
            target: [1.0, 2.8, -0.5],
            fov_degrees: 36.0,
            aperture: 0.004,
        });

    let result = api.render(builder, &request)?;
    eprintln!(
        "city: {}x{} {:.0}ms objs={} tris={} -> {}",
        result.width, result.height, result.duration_ms,
        result.object_count, result.triangle_count,
        result.output_path.display(),
    );
    Ok(())
}
