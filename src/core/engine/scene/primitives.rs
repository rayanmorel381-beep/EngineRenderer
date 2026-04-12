use crate::core::engine::rendering::raytracing::{Material, Triangle, Vec3};

pub fn append_box(triangles: &mut Vec<Triangle>, center: Vec3, half: Vec3, material: Material) {
    let p000 = center + Vec3::new(-half.x, -half.y, -half.z);
    let p001 = center + Vec3::new(-half.x, -half.y, half.z);
    let p010 = center + Vec3::new(-half.x, half.y, -half.z);
    let p011 = center + Vec3::new(-half.x, half.y, half.z);
    let p100 = center + Vec3::new(half.x, -half.y, -half.z);
    let p101 = center + Vec3::new(half.x, -half.y, half.z);
    let p110 = center + Vec3::new(half.x, half.y, -half.z);
    let p111 = center + Vec3::new(half.x, half.y, half.z);

    append_quad(triangles, p001, p101, p111, p011, material);
    append_quad(triangles, p100, p000, p010, p110, material);
    append_quad(triangles, p000, p001, p011, p010, material);
    append_quad(triangles, p101, p100, p110, p111, material);
    append_quad(triangles, p010, p011, p111, p110, material);
    append_quad(triangles, p000, p100, p101, p001, material);
}

pub fn append_quad(triangles: &mut Vec<Triangle>, a: Vec3, b: Vec3, c: Vec3, d: Vec3, material: Material) {
    triangles.push(Triangle::flat(a, b, c, material));
    triangles.push(Triangle::flat(a, c, d, material));
}

pub fn append_ring(
    triangles: &mut Vec<Triangle>,
    center: Vec3,
    inner_radius: f64,
    outer_radius: f64,
    axis_u: Vec3,
    axis_v: Vec3,
    segments: usize,
    material: Material,
) {
    let u = axis_u.normalize();
    let v = axis_v.normalize();
    let segment_count = segments.max(12);

    for segment in 0..segment_count {
        let start = std::f64::consts::TAU * segment as f64 / segment_count as f64;
        let end = std::f64::consts::TAU * (segment + 1) as f64 / segment_count as f64;
        let dir_start = u * start.cos() + v * start.sin();
        let dir_end = u * end.cos() + v * end.sin();

        let inner_start = center + dir_start * inner_radius;
        let outer_start = center + dir_start * outer_radius;
        let inner_end = center + dir_end * inner_radius;
        let outer_end = center + dir_end * outer_radius;
        append_quad(triangles, inner_start, outer_start, outer_end, inner_end, material);
    }
}

pub fn append_gabled_roof(
    triangles: &mut Vec<Triangle>,
    center: Vec3,
    half_width: f64,
    half_depth: f64,
    roof_height: f64,
    material: Material,
) {
    let left_front = center + Vec3::new(-half_width, 0.0, half_depth);
    let right_front = center + Vec3::new(half_width, 0.0, half_depth);
    let left_back = center + Vec3::new(-half_width, 0.0, -half_depth);
    let right_back = center + Vec3::new(half_width, 0.0, -half_depth);
    let ridge_front = center + Vec3::new(0.0, roof_height, half_depth);
    let ridge_back = center + Vec3::new(0.0, roof_height, -half_depth);

    append_quad(triangles, left_front, left_back, ridge_back, ridge_front, material);
    append_quad(triangles, right_front, ridge_front, ridge_back, right_back, material);
    triangles.push(Triangle::flat(left_front, right_front, ridge_front, material));
    triangles.push(Triangle::flat(left_back, ridge_back, right_back, material));
}
