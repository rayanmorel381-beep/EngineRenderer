//! Color-space conversion (sRGB↔linear, RGB↔HSV), perceptual luminance, and
//! black-body color temperature approximation.

use crate::core::engine::rendering::raytracing::Vec3;

use super::interpolation::saturate;

pub fn luminance(color: Vec3) -> f64 {
    color.x * 0.2126 + color.y * 0.7152 + color.z * 0.0722
}

pub fn srgb_to_linear(srgb: Vec3) -> Vec3 {
    fn channel(c: f64) -> f64 {
        if c <= 0.04045 {
            c / 12.92
        } else {
            ((c + 0.055) / 1.055).powf(2.4)
        }
    }
    Vec3::new(channel(srgb.x), channel(srgb.y), channel(srgb.z))
}

pub fn linear_to_srgb(linear: Vec3) -> Vec3 {
    fn channel(c: f64) -> f64 {
        if c <= 0.0031308 {
            c * 12.92
        } else {
            1.055 * c.powf(1.0 / 2.4) - 0.055
        }
    }
    Vec3::new(
        channel(linear.x.max(0.0)),
        channel(linear.y.max(0.0)),
        channel(linear.z.max(0.0)),
    )
}

pub fn rgb_to_hsv(rgb: Vec3) -> Vec3 {
    let max = rgb.x.max(rgb.y).max(rgb.z);
    let min = rgb.x.min(rgb.y).min(rgb.z);
    let delta = max - min;

    let h = if delta < f64::EPSILON {
        0.0
    } else if (max - rgb.x).abs() < f64::EPSILON {
        60.0 * (((rgb.y - rgb.z) / delta) % 6.0)
    } else if (max - rgb.y).abs() < f64::EPSILON {
        60.0 * ((rgb.z - rgb.x) / delta + 2.0)
    } else {
        60.0 * ((rgb.x - rgb.y) / delta + 4.0)
    };

    let s = if max < f64::EPSILON { 0.0 } else { delta / max };

    Vec3::new(if h < 0.0 { h + 360.0 } else { h }, s, max)
}

pub fn hsv_to_rgb(hsv: Vec3) -> Vec3 {
    let h = hsv.x;
    let s = hsv.y;
    let v = hsv.z;
    let c = v * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = v - c;

    let (r, g, b) = if h < 60.0 {
        (c, x, 0.0)
    } else if h < 120.0 {
        (x, c, 0.0)
    } else if h < 180.0 {
        (0.0, c, x)
    } else if h < 240.0 {
        (0.0, x, c)
    } else if h < 300.0 {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };

    Vec3::new(r + m, g + m, b + m)
}

pub fn color_temperature(kelvin: f64) -> Vec3 {
    let t = (kelvin / 100.0).clamp(10.0, 400.0);
    let r = if t <= 66.0 {
        1.0
    } else {
        saturate(1.292936 * (t - 60.0).powf(-0.1332047592))
    };
    let g = if t <= 66.0 {
        saturate(0.390082 * (t).ln() - 0.631841)
    } else {
        saturate(1.129891 * (t - 60.0).powf(-0.0755148492))
    };
    let b = if t >= 66.0 {
        1.0
    } else if t <= 19.0 {
        0.0
    } else {
        saturate(0.543207 * (t - 10.0).ln() - 1.19625)
    };
    Vec3::new(r, g, b)
}
