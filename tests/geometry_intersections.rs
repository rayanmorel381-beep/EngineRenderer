use enginerenderer::api::objects::primitives::{Material, Ray, Sphere, Triangle, Vec3};

#[test]
fn sphere_intersection_returns_expected_distance() {
    let sphere = Sphere {
        center: Vec3::new(0.0, 0.0, 0.0),
        radius: 1.0,
        material: Material::new(Vec3::ONE, 0.1, 0.0, 0.0, Vec3::ZERO),
    };

    let ray = Ray::new(
        Vec3::new(0.0, 0.0, -3.0),
        Vec3::new(0.0, 0.0, 1.0),
    );

    let hit = sphere.hit(&ray, 0.001, f64::INFINITY).expect("expected a hit");
    assert!((hit.distance - 2.0).abs() < 0.001);
}

#[test]
fn triangle_intersection_returns_expected_distance() {
    let triangle = Triangle::flat(
        Vec3::new(-1.0, -1.0, 0.0),
        Vec3::new(1.0, -1.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
        Material::new(Vec3::ONE, 0.2, 0.0, 0.0, Vec3::ZERO),
    );

    let ray = Ray::new(
        Vec3::new(0.0, 0.0, -2.0),
        Vec3::new(0.0, 0.0, 1.0),
    );

    let hit = triangle.hit(&ray, 0.001, f64::INFINITY).expect("expected triangle hit");
    assert!((hit.distance - 2.0).abs() < 0.001);
}

#[test]
fn quaternion_rotation_turns_x_axis_toward_negative_z() {
    let half_angle = std::f64::consts::FRAC_PI_4;
    let rotated = Vec3::new(1.0, 0.0, 0.0)
        .rotate_quaternion([0.0, half_angle.sin(), 0.0, half_angle.cos()]);

    assert!(rotated.x.abs() < 0.001);
    assert!((rotated.z + 1.0).abs() < 0.001);
}
