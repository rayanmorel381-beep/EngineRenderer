//! Spherical/Cartesian conversions, tangent frames, reflection, triangle area
//! and barycentric coordinates.

use crate::core::engine::rendering::raytracing::Vec3;

pub fn spherical_to_cartesian(theta: f64, phi: f64) -> Vec3 {
    Vec3::new(
        phi.cos() * theta.sin(),
        theta.cos(),
        phi.sin() * theta.sin(),
    )
}

pub fn cartesian_to_spherical(dir: Vec3) -> (f64, f64) {
    let theta = dir.y.clamp(-1.0, 1.0).acos();
    let phi = dir.z.atan2(dir.x);
    (theta, phi)
}

pub fn build_tangent_frame(normal: Vec3) -> (Vec3, Vec3) {
    let helper = if normal.y.abs() < 0.999 {
        Vec3::new(0.0, 1.0, 0.0)
    } else {
        Vec3::new(1.0, 0.0, 0.0)
    };
    let tangent = normal.cross(helper).normalize();
    let bitangent = normal.cross(tangent).normalize();
    (tangent, bitangent)
}

pub fn reflect(incident: Vec3, normal: Vec3) -> Vec3 {
    incident - normal * 2.0 * incident.dot(normal)
}

pub fn triangle_area(a: Vec3, b: Vec3, c: Vec3) -> f64 {
    (b - a).cross(c - a).length() * 0.5
}

pub fn barycentric(p: Vec3, a: Vec3, b: Vec3, c: Vec3) -> (f64, f64, f64) {
    let v0 = b - a;
    let v1 = c - a;
    let v2 = p - a;
    let d00 = v0.dot(v0);
    let d01 = v0.dot(v1);
    let d11 = v1.dot(v1);
    let d20 = v2.dot(v0);
    let d21 = v2.dot(v1);
    let denom = d00 * d11 - d01 * d01;
    if denom.abs() < f64::EPSILON {
        return (1.0, 0.0, 0.0);
    }
    let v = (d11 * d20 - d01 * d21) / denom;
    let w = (d00 * d21 - d01 * d20) / denom;
    let u = 1.0 - v - w;
    (u, v, w)
}
