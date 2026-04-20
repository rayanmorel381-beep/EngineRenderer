
use crate::core::engine::rendering::raytracing::Vec3;
use crate::core::engine::rendering::utils::luminance;

pub fn extract_bright(color: Vec3, threshold: f64) -> Vec3 {
    let lum = luminance(color);
    if lum > threshold {
        color * ((lum - threshold) / lum)
    } else {
        Vec3::ZERO
    }
}

pub fn film_grain(color: Vec3, noise: f64, intensity: f64) -> Vec3 {
    let grain = Vec3::splat(noise * intensity);
    Vec3::new(
        (color.x + grain.x).max(0.0),
        (color.y + grain.y).max(0.0),
        (color.z + grain.z).max(0.0),
    )
}

pub fn chromatic_aberration_sample(u: f64, v: f64, strength: f64) -> (f64, f64) {
    let cu = u - 0.5;
    let cv = v - 0.5;
    let dist = (cu * cu + cv * cv).sqrt();
    let offset = dist * strength;
    let angle = cv.atan2(cu);
    (u + offset * angle.cos(), v + offset * angle.sin())
}

pub fn sharpen_pixel(center: Vec3, blurred: Vec3, amount: f64) -> Vec3 {
    let detail = center - blurred;
    Vec3::new(
        (center.x + detail.x * amount).max(0.0),
        (center.y + detail.y * amount).max(0.0),
        (center.z + detail.z * amount).max(0.0),
    )
}
