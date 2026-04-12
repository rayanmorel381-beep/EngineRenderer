use crate::core::engine::rendering::{
    loader::content_loader::ContentLoader,
    environment::procedural::ProceduralEnvironment,
    loader::glb_loader::GlbLoader,
    materials::material::MaterialLibrary,
    mesh::{asset::MeshAsset, operations::{compute_tangents, recalculate_normals, subdivide}},
    loader::obj_loader::ObjLoader,
    raytracing::{
        AreaLight, Camera, DirectionalLight, Material, Scene, Sphere, Vec3,
    },
    effects::volumetric_effects::medium::VolumetricMedium,
};

use super::{
    celestial::CelestialBodies,
    graph::SceneGraph,
    objects::{append_car, append_house, append_tree},
    primitives::{append_box, append_ring},
    world::{append_celestial_panorama, append_showcase_world},
};

use crate::core::coremanager::camera_manager::CameraManager;
use crate::core::scheduler::resource::ResourceManager;

#[derive(Debug, Clone)]
pub struct EngineScene {
    pub scene: Scene,
    pub camera: Camera,
    pub graph: SceneGraph,
}

#[derive(Debug, Clone)]
pub struct ShowcaseShot {
    pub name: &'static str,
    pub scene: Scene,
    pub camera: Camera,
}

impl EngineScene {
    pub fn from_bodies(
        catalog: &CelestialBodies,
        camera_manager: &CameraManager,
        resource_manager: &ResourceManager,
        graph: SceneGraph,
        aspect_ratio: f64,
        time: f64,
    ) -> Self {
        let mut scene = catalog.to_scene();
        let environment = resource_manager.environment();
        let luminous_nodes = graph.luminous_node_count().max(1) as f64;
        let geometric_scale = (graph.average_radius() + graph.radial_extent_hint() * 0.05).max(1.0);

        scene.sky_top = environment.sky_top;
        scene.sky_bottom = environment.sky_bottom;
        scene.sun.direction = environment.sun_direction;
        scene.sun.color = environment.sun_color;
        scene.sun.intensity = environment.sun_intensity * (1.0 + luminous_nodes * 0.03);
        scene.sun.angular_radius = environment.sun_angular_radius;
        scene.exposure = environment.exposure * resource_manager.surface_detail_scale() / geometric_scale.powf(0.08);
        scene.volume = scene
            .volume
            .with_density_multiplier(0.92 + luminous_nodes * 0.04 + resource_manager.surface_detail_scale() * 0.06);
        scene.hdri = Some(ProceduralEnvironment::cinematic_space());
        scene.solar_elevation = environment.solar_elevation;

        let showcase_anchor = graph.focus_point() + Vec3::new(0.2, 0.0, 5.4);
        let content_bundle = ContentLoader.load_showcase_bundle();
        let obj_meshes = ObjLoader.load_embedded_showcase();
        let glb_meshes = GlbLoader.load_embedded_showcase();

        scene.triangles = content_bundle
            .primary_meshes
            .iter()
            .chain(content_bundle.cinematic_meshes.iter())
            .chain(obj_meshes.iter())
            .chain(glb_meshes.iter())
            .enumerate()
            .flat_map(|(index, mesh)| {
                let refined = refine_mesh_for_render(mesh);
                let translation = showcase_anchor
                    + Vec3::new(
                        -3.8 + index as f64 * 1.25,
                        0.38 + (index % 3) as f64 * 0.46,
                        -1.0 + index as f64 * 0.72,
                    );
                let scale = (0.16 + refined.descriptor.bounding_radius * 0.10).clamp(0.14, 0.48);
                let fallback_material = match index % 4 {
                    0 => MaterialLibrary::metallic_moon(),
                    1 => MaterialLibrary::rocky_world(Vec3::new(0.58, 0.46, 0.34)),
                    2 => MaterialLibrary::icy_world(),
                    _ => MaterialLibrary::ocean_world(),
                };
                refined.to_triangles(translation, scale, refined.material_or(fallback_material))
            })
            .collect();

        scene.area_lights = vec![
            AreaLight {
                position: graph.focus_point() + Vec3::new(-2.6, 3.0, 2.2),
                u: Vec3::new(1.1, 0.0, 0.0),
                v: Vec3::new(0.0, 0.0, 1.3),
                color: Vec3::new(1.0, 0.84, 0.72),
                intensity: 1.8,
            },
            AreaLight {
                position: graph.focus_point() + Vec3::new(2.2, 2.5, -1.6),
                u: Vec3::new(0.0, 0.8, 0.0),
                v: Vec3::new(0.0, 0.0, 1.1),
                color: Vec3::new(0.66, 0.78, 1.0),
                intensity: 1.5,
            },
            AreaLight {
                position: graph.focus_point() + Vec3::new(0.4, 4.4, -3.2),
                u: Vec3::new(1.4, 0.0, 0.0),
                v: Vec3::new(0.0, 0.7, 1.2),
                color: Vec3::new(0.94, 0.96, 1.0),
                intensity: 1.0,
            },
        ];

        append_celestial_panorama(&mut scene, graph.focus_point());

        scene.sun.intensity *= 1.12;
        scene.exposure *= 1.04 + (scene.triangles.len() as f64).ln().max(0.0) * 0.010;

        let camera = build_showcase_camera(camera_manager, &graph, showcase_anchor, aspect_ratio, time);

        Self { scene, camera, graph }
    }

    pub fn node_count(&self) -> usize {
        self.graph.node_count()
    }

    pub fn dedicated_gallery_shots() -> Vec<ShowcaseShot> {
        vec![
            build_car_showcase(),
            build_tree_showcase(),
            build_house_showcase(),
            build_world_showcase(),
            build_planet_showcase(),
            build_sun_showcase(),
            build_black_hole_showcase(),
        ]
    }
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

fn build_showcase_camera(
    camera_manager: &CameraManager,
    graph: &SceneGraph,
    showcase_anchor: Vec3,
    aspect_ratio: f64,
    time: f64,
) -> Camera {
    let hero_target = graph.focus_point() * 0.58
        + showcase_anchor * 0.42
        + Vec3::new(0.6, 0.9, -0.25);
    let scene_scale = graph.scene_radius().max(1.0);
    let orbit = (camera_manager.distance_to_focus() * 0.72)
        .max(scene_scale * 1.45)
        .clamp(9.0, 18.0);
    let lift = orbit * 0.24 + (time * 0.22).sin() * 0.25;
    let origin = hero_target + Vec3::new(-orbit * 0.82, lift, orbit * 0.94);
    let fov = (34.0 - scene_scale * 0.18).clamp(28.0, 36.0);
    let aperture = (scene_scale / 1600.0).clamp(0.002, 0.005);

    Camera::look_at(
        origin,
        hero_target,
        Vec3::new(0.0, 1.0, 0.0),
        fov,
        aspect_ratio,
    )
    .with_physical_lens(aperture, 0.0, Vec3::ZERO)
}

// ── Gallery helpers ─────────────────────────────────────────────────────

fn gallery_scene_template(
    sky_top: Vec3,
    sky_bottom: Vec3,
    exposure: f64,
    sun_intensity: f64,
) -> Scene {
    Scene {
        objects: Vec::new(),
        triangles: Vec::new(),
        sun: DirectionalLight {
            direction: Vec3::new(-0.55, -0.85, -0.45).normalize(),
            color: Vec3::new(1.0, 0.95, 0.88),
            intensity: sun_intensity,
            angular_radius: 0.045,
        },
        area_lights: vec![AreaLight {
            position: Vec3::new(-2.0, 3.0, 2.2),
            u: Vec3::new(1.2, 0.0, 0.0),
            v: Vec3::new(0.0, 0.0, 1.2),
            color: Vec3::new(1.0, 0.84, 0.72),
            intensity: 1.6,
        }],
        sky_top,
        sky_bottom,
        exposure,
        volume: VolumetricMedium::cinematic_nebula().with_density_multiplier(0.45),
        hdri: Some(ProceduralEnvironment::cinematic_space()),
        solar_elevation: 0.52,
    }
}

fn build_car_showcase() -> ShowcaseShot {
    let mut scene = gallery_scene_template(
        Vec3::new(0.06, 0.10, 0.28),
        Vec3::new(0.22, 0.26, 0.36),
        1.08,
        1.6,
    );
    append_box(
        &mut scene.triangles,
        Vec3::new(0.0, -0.14, 0.0),
        Vec3::new(4.4, 0.12, 2.4),
        MaterialLibrary::architectural_plaster(),
    );
    append_box(
        &mut scene.triangles,
        Vec3::new(0.0, 0.02, 0.0),
        Vec3::new(3.8, 0.02, 1.1),
        MaterialLibrary::asphalt(),
    );
    append_car(&mut scene, Vec3::new(0.0, 0.0, 0.0), 1.4, Vec3::new(0.86, 0.14, 0.10));
    append_tree(&mut scene, Vec3::new(-2.6, 0.0, -1.4), 0.9);
    append_tree(&mut scene, Vec3::new(2.9, 0.0, -1.7), 1.0);
    let camera = Camera::look_at(
        Vec3::new(-2.4, 1.5, 4.8),
        Vec3::new(0.2, 0.55, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
        32.0,
        16.0 / 9.0,
    )
    .with_physical_lens(0.022, 0.012, Vec3::ZERO);
    ShowcaseShot { name: "car", scene, camera }
}

fn build_tree_showcase() -> ShowcaseShot {
    let mut scene = gallery_scene_template(
        Vec3::new(0.05, 0.09, 0.24),
        Vec3::new(0.20, 0.24, 0.34),
        1.02,
        1.55,
    );
    append_box(
        &mut scene.triangles,
        Vec3::new(0.0, -0.14, 0.0),
        Vec3::new(3.8, 0.12, 3.2),
        MaterialLibrary::architectural_plaster(),
    );
    append_tree(&mut scene, Vec3::new(0.0, 0.0, 0.0), 1.8);
    append_house(&mut scene, Vec3::new(2.6, 0.0, -1.2), 0.7);
    let camera = Camera::look_at(
        Vec3::new(2.2, 2.0, 4.4),
        Vec3::new(0.0, 1.5, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
        30.0,
        16.0 / 9.0,
    )
    .with_physical_lens(0.020, 0.010, Vec3::ZERO);
    ShowcaseShot { name: "tree", scene, camera }
}

fn build_house_showcase() -> ShowcaseShot {
    let mut scene = gallery_scene_template(
        Vec3::new(0.06, 0.10, 0.26),
        Vec3::new(0.21, 0.25, 0.35),
        1.06,
        1.62,
    );
    append_box(
        &mut scene.triangles,
        Vec3::new(0.0, -0.14, 0.0),
        Vec3::new(4.8, 0.12, 3.6),
        MaterialLibrary::architectural_plaster(),
    );
    append_house(&mut scene, Vec3::new(0.0, 0.0, 0.0), 1.4);
    append_car(&mut scene, Vec3::new(-1.8, 0.0, 1.2), 0.8, Vec3::new(0.18, 0.32, 0.80));
    append_tree(&mut scene, Vec3::new(2.6, 0.0, -0.8), 1.0);
    let camera = Camera::look_at(
        Vec3::new(-3.4, 2.1, 5.8),
        Vec3::new(0.2, 1.1, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
        34.0,
        16.0 / 9.0,
    )
    .with_physical_lens(0.022, 0.014, Vec3::ZERO);
    ShowcaseShot { name: "house", scene, camera }
}

fn build_world_showcase() -> ShowcaseShot {
    let mut scene = gallery_scene_template(
        Vec3::new(0.05, 0.10, 0.24),
        Vec3::new(0.20, 0.24, 0.34),
        1.08,
        1.65,
    );
    append_showcase_world(&mut scene, Vec3::new(0.0, 0.0, 0.0));
    scene.area_lights.push(AreaLight {
        position: Vec3::new(0.0, 4.8, -2.4),
        u: Vec3::new(2.4, 0.0, 0.0),
        v: Vec3::new(0.0, 0.8, 2.0),
        color: Vec3::new(0.82, 0.88, 1.0),
        intensity: 1.2,
    });
    let camera = Camera::look_at(
        Vec3::new(-7.8, 3.6, 8.8),
        Vec3::new(0.0, 0.8, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
        34.0,
        16.0 / 9.0,
    )
    .with_physical_lens(0.020, 0.016, Vec3::ZERO);
    ShowcaseShot { name: "world", scene, camera }
}

fn build_planet_showcase() -> ShowcaseShot {
    let mut scene = gallery_scene_template(
        Vec3::new(0.010, 0.016, 0.045),
        Vec3::new(0.001, 0.001, 0.006),
        1.32,
        1.25,
    );
    scene.volume = VolumetricMedium::cinematic_nebula().with_density_multiplier(0.35);
    let center = Vec3::new(0.0, 0.2, 0.0);
    scene.objects.push(Sphere {
        center,
        radius: 2.2,
        material: MaterialLibrary::lush_planet(),
    });
    scene.objects.push(Sphere {
        center: center + Vec3::new(2.6, 0.55, 1.1),
        radius: 0.42,
        material: MaterialLibrary::metallic_moon(),
    });
    append_ring(
        &mut scene.triangles,
        center,
        2.6,
        3.8,
        Vec3::new(1.0, 0.0, 0.0),
        Vec3::new(0.0, 0.22, 1.0).normalize(),
        28,
        Material::new(Vec3::new(0.70, 0.66, 0.54), 0.76, 0.03, 0.05, Vec3::ZERO)
            .with_layers(0.94, 0.02, Vec3::new(0.04, 0.04, 0.03))
            .with_optics(0.08, 0.04, 0.02),
    );
    let camera = Camera::look_at(
        Vec3::new(-5.0, 2.0, 5.8),
        Vec3::new(0.0, 0.2, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
        24.0,
        16.0 / 9.0,
    )
    .with_physical_lens(0.018, 0.020, Vec3::ZERO);
    ShowcaseShot { name: "planet", scene, camera }
}

fn build_sun_showcase() -> ShowcaseShot {
    let mut scene = gallery_scene_template(
        Vec3::new(0.012, 0.018, 0.050),
        Vec3::new(0.001, 0.001, 0.005),
        1.55,
        1.10,
    );
    scene.volume = VolumetricMedium::cinematic_nebula().with_density_multiplier(0.35);
    scene.objects.push(Sphere {
        center: Vec3::new(0.0, 0.0, 0.0),
        radius: 2.8,
        material: MaterialLibrary::solar_corona(),
    });
    scene.area_lights.push(AreaLight {
        position: Vec3::new(-0.4, 0.6, 1.0),
        u: Vec3::new(1.8, 0.0, 0.0),
        v: Vec3::new(0.0, 1.6, 0.7),
        color: Vec3::new(1.0, 0.72, 0.34),
        intensity: 1.2,
    });
    let camera = Camera::look_at(
        Vec3::new(-7.0, 2.5, 8.5),
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
        22.0,
        16.0 / 9.0,
    )
    .with_physical_lens(0.016, 0.018, Vec3::ZERO);
    ShowcaseShot { name: "sun", scene, camera }
}

fn build_black_hole_showcase() -> ShowcaseShot {
    let mut scene = gallery_scene_template(
        Vec3::new(0.008, 0.010, 0.032),
        Vec3::new(0.001, 0.001, 0.004),
        1.60,
        0.95,
    );
    scene.volume = VolumetricMedium::cinematic_nebula().with_density_multiplier(0.40);
    let center = Vec3::new(0.0, 0.0, 0.0);
    scene.objects.push(Sphere {
        center,
        radius: 2.1,
        material: MaterialLibrary::event_horizon(),
    });
    append_ring(
        &mut scene.triangles,
        center,
        2.6,
        4.6,
        Vec3::new(1.0, 0.0, 0.0),
        Vec3::new(0.0, 0.34, 1.0).normalize(),
        36,
        MaterialLibrary::accretion_disk(),
    );
    scene.area_lights.push(AreaLight {
        position: Vec3::new(-1.4, 0.5, 1.3),
        u: Vec3::new(1.3, 0.0, 0.0),
        v: Vec3::new(0.0, 0.4, 1.6),
        color: Vec3::new(0.96, 0.56, 0.22),
        intensity: 1.8,
    });
    let camera = Camera::look_at(
        Vec3::new(-5.2, 1.8, 6.2),
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
        23.0,
        16.0 / 9.0,
    )
    .with_physical_lens(0.018, 0.020, Vec3::ZERO);
    ShowcaseShot { name: "black_hole", scene, camera }
}
