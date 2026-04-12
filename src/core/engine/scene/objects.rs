use crate::core::engine::rendering::{
    materials::material::MaterialLibrary,
    raytracing::{Material, Scene, Sphere, Vec3},
};

use super::primitives::{append_box, append_gabled_roof};

pub fn append_house(scene: &mut Scene, base: Vec3, scale: f64) {
    let walls = MaterialLibrary::architectural_plaster();
    let roof = MaterialLibrary::roof_tiles();
    let glass = MaterialLibrary::window_glass();
    let wood = MaterialLibrary::tree_bark();

    append_box(
        &mut scene.triangles,
        base + Vec3::new(0.0, 0.10 * scale, 0.0),
        Vec3::new(1.12 * scale, 0.10 * scale, 0.98 * scale),
        wood,
    );
    append_box(
        &mut scene.triangles,
        base + Vec3::new(0.0, 0.85 * scale, 0.0),
        Vec3::new(1.05 * scale, 0.85 * scale, 0.92 * scale),
        walls,
    );
    append_gabled_roof(
        &mut scene.triangles,
        base + Vec3::new(0.0, 1.68 * scale, 0.0),
        1.18 * scale,
        1.02 * scale,
        0.72 * scale,
        roof,
    );
    append_box(
        &mut scene.triangles,
        base + Vec3::new(-0.48 * scale, 2.02 * scale, -0.24 * scale),
        Vec3::new(0.12 * scale, 0.38 * scale, 0.12 * scale),
        walls,
    );
    append_box(
        &mut scene.triangles,
        base + Vec3::new(0.0, 0.42 * scale, 0.93 * scale),
        Vec3::new(0.22 * scale, 0.42 * scale, 0.03 * scale),
        wood,
    );

    for &(x, y) in &[(-0.42, 1.00), (0.42, 1.00), (-0.42, 0.62), (0.42, 0.62)] {
        append_box(
            &mut scene.triangles,
            base + Vec3::new(x * scale, y * scale, 0.94 * scale),
            Vec3::new(0.16 * scale, 0.12 * scale, 0.02 * scale),
            glass,
        );
    }
    for &(z, y) in &[(-0.44, 0.96), (0.44, 0.96)] {
        append_box(
            &mut scene.triangles,
            base + Vec3::new(1.06 * scale, y * scale, z * scale),
            Vec3::new(0.02 * scale, 0.14 * scale, 0.18 * scale),
            glass,
        );
        append_box(
            &mut scene.triangles,
            base + Vec3::new(-1.06 * scale, y * scale, z * scale),
            Vec3::new(0.02 * scale, 0.14 * scale, 0.18 * scale),
            glass,
        );
    }
}

pub fn append_car(scene: &mut Scene, base: Vec3, scale: f64, color: Vec3) {
    let body = MaterialLibrary::automotive_paint(color);
    let glass = MaterialLibrary::window_glass();
    let tire = MaterialLibrary::rubber_tire();
    let rim = MaterialLibrary::metallic_moon();
    let light = Material::new(Vec3::new(0.94, 0.92, 0.84), 0.06, 0.01, 0.04, Vec3::new(1.1, 1.0, 0.8))
        .with_layers(1.0, 0.12, Vec3::new(0.04, 0.04, 0.02))
        .with_optics(0.02, 0.02, 0.04);

    append_box(
        &mut scene.triangles,
        base + Vec3::new(0.0, 0.34 * scale, 0.0),
        Vec3::new(0.95 * scale, 0.22 * scale, 0.46 * scale),
        body,
    );
    append_box(
        &mut scene.triangles,
        base + Vec3::new(-0.34 * scale, 0.42 * scale, 0.0),
        Vec3::new(0.28 * scale, 0.10 * scale, 0.42 * scale),
        body,
    );
    append_box(
        &mut scene.triangles,
        base + Vec3::new(0.44 * scale, 0.40 * scale, 0.0),
        Vec3::new(0.20 * scale, 0.08 * scale, 0.40 * scale),
        body,
    );
    append_box(
        &mut scene.triangles,
        base + Vec3::new(0.10 * scale, 0.60 * scale, 0.0),
        Vec3::new(0.48 * scale, 0.18 * scale, 0.34 * scale),
        glass,
    );

    for &(x, z) in &[(-0.62, -0.38), (-0.62, 0.38), (0.62, -0.38), (0.62, 0.38)] {
        scene.objects.push(Sphere {
            center: base + Vec3::new(x * scale, 0.14 * scale, z * scale),
            radius: 0.19 * scale,
            material: tire,
        });
        scene.objects.push(Sphere {
            center: base + Vec3::new(x * scale, 0.14 * scale, z * scale),
            radius: 0.08 * scale,
            material: rim,
        });
    }

    for z in [-0.20, 0.20] {
        scene.objects.push(Sphere {
            center: base + Vec3::new(-0.94 * scale, 0.36 * scale, z * scale),
            radius: 0.06 * scale,
            material: light,
        });
    }
}

pub fn append_tree(scene: &mut Scene, base: Vec3, scale: f64) {
    append_box(
        &mut scene.triangles,
        base + Vec3::new(0.0, 0.70 * scale, 0.0),
        Vec3::new(0.12 * scale, 0.70 * scale, 0.12 * scale),
        MaterialLibrary::tree_bark(),
    );

    for &(offset, radius) in &[
        (Vec3::new(0.0, 1.72, 0.0), 0.58),
        (Vec3::new(-0.28, 1.46, 0.18), 0.42),
        (Vec3::new(0.30, 1.42, -0.16), 0.40),
        (Vec3::new(-0.08, 1.92, -0.12), 0.34),
        (Vec3::new(0.22, 1.86, 0.16), 0.30),
        (Vec3::new(-0.34, 1.70, -0.20), 0.28),
    ] {
        scene.objects.push(Sphere {
            center: base + offset * scale,
            radius: radius * scale,
            material: MaterialLibrary::tree_foliage(),
        });
    }
}
