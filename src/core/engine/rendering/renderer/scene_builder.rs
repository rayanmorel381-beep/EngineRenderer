use crate::core::engine::rendering::{
    loader::content_loader::ContentLoader,
    environment::procedural::ProceduralEnvironment,
    loader::glb_loader::GlbLoader,
    materials::material::MaterialLibrary,
    mesh::asset::MeshAsset,
    mesh::operations::{compute_tangents, recalculate_normals, subdivide},
    mesh::vertex::geometric_density,
    loader::obj_loader::ObjLoader,
    raytracing::{
        AreaLight, Camera, DirectionalLight, Material, Scene, Sphere, Triangle, Vec3,
    },
    effects::volumetric_effects::medium::VolumetricMedium,
};

pub fn build_realistic_scene(aspect_ratio: f64) -> (Scene, Camera) {
    let polished_ceramic =
        Material::new(Vec3::new(0.92, 0.94, 0.98), 0.16, 0.02, 0.22, Vec3::ZERO)
            .with_layers(0.98, 0.18, Vec3::new(0.06, 0.08, 0.10))
            .with_optics(0.10, 0.06, 0.22);
    let brushed_copper =
        Material::new(Vec3::new(0.82, 0.45, 0.20), 0.22, 0.88, 0.72, Vec3::ZERO)
            .with_layers(0.90, 0.24, Vec3::new(0.12, 0.05, 0.02))
            .with_optics(0.03, 0.76, 0.18);
    let cobalt_lacquer =
        Material::new(Vec3::new(0.19, 0.33, 0.69), 0.09, 0.12, 0.54, Vec3::ZERO)
            .with_layers(0.96, 0.36, Vec3::new(0.04, 0.06, 0.12))
            .with_optics(0.14, 0.12, 0.44);
    let obsidian = Material::new(Vec3::new(0.06, 0.07, 0.10), 0.04, 0.72, 0.86, Vec3::ZERO)
        .with_layers(0.88, 0.28, Vec3::new(0.05, 0.05, 0.08))
        .with_optics(0.02, 0.64, 0.26);
    let floor = MaterialLibrary::rocky_world(Vec3::new(0.55, 0.56, 0.60));
    let emitter = MaterialLibrary::stellar_surface();
    let ocean_world = MaterialLibrary::ocean_world();
    let icy_world = MaterialLibrary::icy_world();

    let content_bundle = ContentLoader.load_showcase_bundle();
    let obj_meshes = ObjLoader.load_embedded_showcase();
    let glb_meshes = GlbLoader.load_embedded_showcase();

    let asset_spheres = asset_meshes_to_spheres(
        &content_bundle.primary_meshes,
        &content_bundle.cinematic_meshes,
        &obj_meshes,
        &glb_meshes,
    );
    let mut triangles = asset_meshes_to_triangles(
        &content_bundle.primary_meshes,
        &content_bundle.cinematic_meshes,
        &obj_meshes,
        &glb_meshes,
    );
    triangles.push(Triangle::flat(
        Vec3::new(-2.8, 0.02, 1.6),
        Vec3::new(-1.2, 0.42, 2.4),
        Vec3::new(-0.6, 0.02, 0.7),
        MaterialLibrary::rocky_world(Vec3::new(0.48, 0.44, 0.40)),
    ));

    let mut objects = vec![
        Sphere {
            center: Vec3::new(0.0, -1002.0, 0.0),
            radius: 1000.0,
            material: floor,
        },
        Sphere {
            center: Vec3::new(-2.4, 0.7, -1.2),
            radius: 1.7,
            material: cobalt_lacquer,
        },
        Sphere {
            center: Vec3::new(1.2, 0.55, 0.3),
            radius: 1.55,
            material: polished_ceramic,
        },
        Sphere {
            center: Vec3::new(3.4, 0.15, -2.4),
            radius: 1.15,
            material: brushed_copper,
        },
        Sphere {
            center: Vec3::new(-0.4, 0.2, 2.8),
            radius: 1.1,
            material: obsidian,
        },
        Sphere {
            center: Vec3::new(-4.8, 0.35, 4.6),
            radius: 0.95,
            material: ocean_world,
        },
        Sphere {
            center: Vec3::new(4.9, 0.25, 5.2),
            radius: 0.88,
            material: icy_world,
        },
        Sphere {
            center: Vec3::new(0.0, 6.5, -4.0),
            radius: 1.0,
            material: emitter,
        },
    ];
    objects.extend(asset_spheres);

    let scene = Scene {
        objects,
        triangles,
        sun: DirectionalLight {
            direction: Vec3::new(-0.7, -1.0, -0.45).normalize(),
            color: Vec3::new(1.0, 0.95, 0.9),
            intensity: 2.8,
            angular_radius: 0.04,
        },
        area_lights: vec![
            AreaLight {
                position: Vec3::new(-3.2, 3.8, 2.4),
                u: Vec3::new(1.2, 0.0, 0.0),
                v: Vec3::new(0.0, 0.0, 1.0),
                color: Vec3::new(1.0, 0.82, 0.72),
                intensity: 1.6,
            },
            AreaLight {
                position: Vec3::new(2.6, 2.9, -1.8),
                u: Vec3::new(0.0, 0.9, 0.0),
                v: Vec3::new(0.0, 0.0, 1.4),
                color: Vec3::new(0.62, 0.76, 1.0),
                intensity: 1.4,
            },
        ],
        sky_top: Vec3::new(0.06, 0.10, 0.28),
        sky_bottom: Vec3::new(0.22, 0.26, 0.36),
        exposure: 1.18,
        volume: VolumetricMedium::cinematic_nebula().with_density_multiplier(1.1),
        hdri: Some(ProceduralEnvironment::cinematic_space()),
        solar_elevation: 0.48,
    };

    let camera = Camera::look_at(
        Vec3::new(0.6, 2.0, 9.0),
        Vec3::new(0.2, 0.8, 0.2),
        Vec3::new(0.0, 1.0, 0.0),
        36.0,
        aspect_ratio,
    );

    (scene, camera)
}

fn refine_mesh_for_render(mesh: &MeshAsset) -> MeshAsset {
    let mut refined = mesh.clone();
    let extra_passes = if refined.descriptor.triangle_count < 256 {
        2
    } else if refined.descriptor.triangle_count < 2_048 {
        1
    } else {
        0
    };

    for _ in 0..extra_passes {
        subdivide(&mut refined);
    }

    if extra_passes > 0 {
        recalculate_normals(&mut refined);
        compute_tangents(&mut refined);
        refined.descriptor.vertex_count = refined.vertices.len();
        refined.descriptor.triangle_count = refined.indices.len() / 3;
        refined.descriptor.bounding_radius = refined
            .vertices
            .iter()
            .map(|vertex| vertex.position.length())
            .fold(mesh.descriptor.bounding_radius.max(0.001), f64::max);
    }

    refined
}

pub fn asset_meshes_to_spheres(
    primary_meshes: &[MeshAsset],
    cinematic_meshes: &[MeshAsset],
    obj_meshes: &[MeshAsset],
    glb_meshes: &[MeshAsset],
) -> Vec<Sphere> {
    primary_meshes
        .iter()
        .chain(cinematic_meshes.iter())
        .chain(obj_meshes.iter())
        .chain(glb_meshes.iter())
        .enumerate()
        .map(|(index, mesh)| {
            let centroid = mesh.centroid();
            let orbit_offset = index as f64 * 1.35;
            let fallback_material = if index % 3 == 0 {
                MaterialLibrary::metallic_moon()
            } else if index % 3 == 1 {
                MaterialLibrary::rocky_world(Vec3::new(0.62, 0.48, 0.34))
            } else {
                MaterialLibrary::icy_world()
            };
            let material = mesh.material_or(fallback_material);

            Sphere {
                center: centroid
                    + Vec3::new(
                        -6.0 + orbit_offset,
                        0.45 + index as f64 * 0.06,
                        3.5 + orbit_offset * 0.55,
                    ),
                radius: mesh.effective_radius()
                    * (0.38 + geometric_density(&mesh.descriptor, mesh.effective_radius().powi(2) * std::f64::consts::PI * 4.0).ln().max(0.2) * 0.015),
                material,
            }
        })
        .collect()
}

pub fn asset_meshes_to_triangles(
    primary_meshes: &[MeshAsset],
    cinematic_meshes: &[MeshAsset],
    obj_meshes: &[MeshAsset],
    glb_meshes: &[MeshAsset],
) -> Vec<Triangle> {
    primary_meshes
        .iter()
        .chain(cinematic_meshes.iter())
        .chain(obj_meshes.iter())
        .chain(glb_meshes.iter())
        .enumerate()
        .flat_map(|(index, mesh)| {
            let refined = refine_mesh_for_render(mesh);
            let translation = Vec3::new(
                -7.5 + index as f64 * 1.8,
                0.35 + (index % 4) as f64 * 0.28,
                1.8 + index as f64 * 1.15,
            );
            let scale = (0.16 + refined.effective_radius() * 0.10).clamp(0.14, 0.58);
            let fallback_material = match index % 4 {
                0 => MaterialLibrary::metallic_moon(),
                1 => MaterialLibrary::rocky_world(Vec3::new(0.64, 0.46, 0.30)),
                2 => MaterialLibrary::icy_world(),
                _ => MaterialLibrary::ocean_world(),
            };
            refined.to_triangles(translation, scale, refined.material_or(fallback_material))
        })
        .collect()
}
