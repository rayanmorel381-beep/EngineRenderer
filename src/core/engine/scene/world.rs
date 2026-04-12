use crate::core::engine::rendering::{
    materials::material::MaterialLibrary,
    raytracing::{AreaLight, Material, Scene, Sphere, Vec3},
};

use super::{
    objects::{append_car, append_house, append_tree},
    primitives::{append_box, append_ring},
};

pub(crate) fn append_showcase_world(scene: &mut Scene, anchor: Vec3) {
    let terrain = MaterialLibrary::architectural_plaster();
    let road = MaterialLibrary::asphalt();
    let lane_mark = Material::new(Vec3::new(0.86, 0.82, 0.66), 0.44, 0.02, 0.04, Vec3::ZERO)
        .with_layers(0.96, 0.02, Vec3::new(0.02, 0.02, 0.01))
        .with_optics(0.04, 0.02, 0.01);

    append_box(
        &mut scene.triangles,
        anchor + Vec3::new(0.0, -0.16, 0.0),
        Vec3::new(8.2, 0.14, 5.0),
        terrain,
    );
    append_box(
        &mut scene.triangles,
        anchor + Vec3::new(0.0, 0.03, 0.0),
        Vec3::new(6.6, 0.03, 1.5),
        road,
    );

    for stripe in [-4.6, -2.3, 0.0, 2.3, 4.6] {
        append_box(
            &mut scene.triangles,
            anchor + Vec3::new(stripe, 0.05, 0.0),
            Vec3::new(0.46, 0.005, 0.05),
            lane_mark,
        );
    }

    append_house(scene, anchor + Vec3::new(-4.6, 0.0, -2.7), 1.05);
    append_house(scene, anchor + Vec3::new(-1.2, 0.0, -2.9), 0.92);
    append_house(scene, anchor + Vec3::new(2.4, 0.0, -2.8), 1.10);
    append_house(scene, anchor + Vec3::new(5.4, 0.0, -2.5), 0.88);

    append_car(scene, anchor + Vec3::new(-2.8, 0.0, -0.15), 0.95, Vec3::new(0.84, 0.16, 0.10));
    append_car(scene, anchor + Vec3::new(0.4, 0.0, 0.18), 1.00, Vec3::new(0.18, 0.32, 0.82));
    append_car(scene, anchor + Vec3::new(3.5, 0.0, -0.22), 0.90, Vec3::new(0.72, 0.74, 0.78));

    for &(x, z, scale) in &[
        (-6.4, -1.8, 1.25),
        (-5.8, 2.0, 1.10),
        (-3.6, 2.6, 0.95),
        (-0.8, 2.9, 1.05),
        (1.8, 2.8, 1.20),
        (4.6, 2.5, 1.08),
        (6.7, 1.8, 1.15),
    ] {
        append_tree(scene, anchor + Vec3::new(x, 0.0, z), scale);
    }
}

pub(crate) fn append_celestial_panorama(scene: &mut Scene, focus: Vec3) {
    let sun_center = focus + Vec3::new(-16.0, 9.5, -24.0);
    let planet_center = focus + Vec3::new(11.5, 5.6, -18.5);
    let black_hole_center = focus + Vec3::new(15.8, 7.4, -31.0);

    scene.objects.extend([
        Sphere {
            center: sun_center,
            radius: 3.6,
            material: MaterialLibrary::solar_corona(),
        },
        Sphere {
            center: planet_center,
            radius: 2.35,
            material: MaterialLibrary::lush_planet(),
        },
        Sphere {
            center: planet_center + Vec3::new(2.7, 0.6, 1.5),
            radius: 0.52,
            material: MaterialLibrary::metallic_moon(),
        },
        Sphere {
            center: black_hole_center,
            radius: 2.45,
            material: MaterialLibrary::event_horizon(),
        },
    ]);

    append_ring(
        &mut scene.triangles,
        planet_center,
        2.9,
        4.0,
        Vec3::new(1.0, 0.0, 0.0),
        Vec3::new(0.0, 0.12, 1.0).normalize(),
        64,
        Material::new(Vec3::new(0.66, 0.60, 0.48), 0.72, 0.04, 0.06, Vec3::ZERO)
            .with_layers(0.94, 0.03, Vec3::new(0.03, 0.03, 0.02))
            .with_optics(0.12, 0.05, 0.02),
    );
    append_ring(
        &mut scene.triangles,
        black_hole_center,
        3.4,
        6.1,
        Vec3::new(1.0, 0.0, 0.0),
        Vec3::new(0.0, 0.26, 1.0).normalize(),
        96,
        MaterialLibrary::accretion_disk(),
    );

    scene.area_lights.extend([
        AreaLight {
            position: sun_center + Vec3::new(-0.4, 0.8, 0.3),
            u: Vec3::new(1.8, 0.0, 0.0),
            v: Vec3::new(0.0, 1.4, 0.8),
            color: Vec3::new(1.0, 0.82, 0.60),
            intensity: 6.4,
        },
        AreaLight {
            position: black_hole_center + Vec3::new(-1.6, 0.4, 0.9),
            u: Vec3::new(1.2, 0.1, 0.0),
            v: Vec3::new(0.0, 0.3, 1.5),
            color: Vec3::new(0.92, 0.58, 0.24),
            intensity: 4.8,
        },
    ]);
}
