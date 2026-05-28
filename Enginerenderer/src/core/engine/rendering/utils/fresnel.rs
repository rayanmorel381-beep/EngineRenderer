//! Schlick (scalar/vector) and dielectric Fresnel reflectance approximations.

use crate::core::engine::rendering::raytracing::Vec3;

pub fn fresnel_schlick(cos_theta: f64, f0: f64) -> f64 {
    f0 + (1.0 - f0) * (1.0 - cos_theta.clamp(0.0, 1.0)).powi(5)
}

pub fn fresnel_schlick_vec(cos_theta: f64, f0: Vec3) -> Vec3 {
    let t = (1.0 - cos_theta.clamp(0.0, 1.0)).powi(5);
    f0 + (Vec3::ONE - f0) * t
}

pub fn fresnel_dielectric(cos_i: f64, eta: f64) -> f64 {
    let sin2_t = eta * eta * (1.0 - cos_i * cos_i).max(0.0);
    if sin2_t > 1.0 {
        return 1.0;
    }
    let cos_t = (1.0 - sin2_t).sqrt();
    let ci = cos_i.abs();
    let rs = ((eta * ci - cos_t) / (eta * ci + cos_t)).powi(2);
    let rp = ((ci - eta * cos_t) / (ci + eta * cos_t)).powi(2);
    (rs + rp) * 0.5
}
