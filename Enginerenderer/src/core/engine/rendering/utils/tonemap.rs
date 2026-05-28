//! Tone mapping operators (ACES, Reinhard extended, Uncharted2) and exposure
//! conversion helpers (EV100 ↔ luminance/exposure).

use crate::core::engine::rendering::raytracing::Vec3;

pub fn aces_tonemap(color: Vec3) -> Vec3 {
    let a = 2.51;
    let b = 0.03;
    let c = 1.43;
    let d = 0.59;
    let e = 0.14;
    let num = color * (color * a + Vec3::splat(b));
    let den = color * (color * c + Vec3::splat(d)) + Vec3::splat(e);
    let mapped = Vec3::new(
        num.x / den.x.max(f64::EPSILON),
        num.y / den.y.max(f64::EPSILON),
        num.z / den.z.max(f64::EPSILON),
    );
    mapped.clamp(0.0, 1.0)
}

pub fn reinhard_extended(color: Vec3, white_point: f64) -> Vec3 {
    let wp2 = white_point * white_point;
    Vec3::new(
        color.x * (1.0 + color.x / wp2) / (1.0 + color.x),
        color.y * (1.0 + color.y / wp2) / (1.0 + color.y),
        color.z * (1.0 + color.z / wp2) / (1.0 + color.z),
    )
}

pub fn uncharted2_tonemap(color: Vec3) -> Vec3 {
    fn partial(x: Vec3) -> Vec3 {
        let a = 0.15;
        let b = 0.50;
        let c = 0.10;
        let d = 0.20;
        let e = 0.02;
        let f = 0.30;
        let num = x * (x * a + Vec3::splat(c * b)) + Vec3::splat(d * e);
        let den = x * (x * a + Vec3::splat(b)) + Vec3::splat(d * f);
        Vec3::new(
            num.x / den.x.max(f64::EPSILON) - e / f,
            num.y / den.y.max(f64::EPSILON) - e / f,
            num.z / den.z.max(f64::EPSILON) - e / f,
        )
    }
    let white = Vec3::splat(11.2);
    let numerator = partial(color * 2.0);
    let denominator = partial(white);
    Vec3::new(
        numerator.x / denominator.x.max(f64::EPSILON),
        numerator.y / denominator.y.max(f64::EPSILON),
        numerator.z / denominator.z.max(f64::EPSILON),
    )
    .clamp(0.0, 1.0)
}

pub fn exposure_from_ev100(ev100: f64) -> f64 {
    1.0 / (1.2 * 2.0_f64.powf(ev100))
}

pub fn ev100_from_luminance(avg_luminance: f64) -> f64 {
    (avg_luminance * 100.0 / 12.5).max(f64::EPSILON).log2()
}
