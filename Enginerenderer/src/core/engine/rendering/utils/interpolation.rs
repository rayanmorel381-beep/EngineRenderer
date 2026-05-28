//! Clamping, interpolation and shaping curves operating on `f64` scalars.

pub fn saturate(value: f64) -> f64 {
    value.clamp(0.0, 1.0)
}

pub fn lerp(a: f64, b: f64, t: f64) -> f64 {
    a + (b - a) * t
}

pub fn inverse_lerp(a: f64, b: f64, value: f64) -> f64 {
    let range = b - a;
    if range.abs() < f64::EPSILON {
        0.0
    } else {
        ((value - a) / range).clamp(0.0, 1.0)
    }
}

pub fn remap(value: f64, from_min: f64, from_max: f64, to_min: f64, to_max: f64) -> f64 {
    let t = inverse_lerp(from_min, from_max, value);
    lerp(to_min, to_max, t)
}

pub fn smoothstep(edge0: f64, edge1: f64, x: f64) -> f64 {
    let width = (edge1 - edge0).abs().max(f64::EPSILON);
    let t = saturate((x - edge0) / width);
    t * t * (3.0 - 2.0 * t)
}

pub fn quintic_smooth(edge0: f64, edge1: f64, x: f64) -> f64 {
    let width = (edge1 - edge0).abs().max(f64::EPSILON);
    let t = saturate((x - edge0) / width);
    t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
}

pub fn bias(value: f64, b: f64) -> f64 {
    value.powf((1.0 - b).max(f64::EPSILON).ln() / 0.5_f64.ln())
}

pub fn gain(value: f64, g: f64) -> f64 {
    if value < 0.5 {
        bias(2.0 * value, g) * 0.5
    } else {
        1.0 - bias(2.0 - 2.0 * value, g) * 0.5
    }
}
