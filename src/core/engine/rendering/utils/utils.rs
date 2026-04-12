use crate::core::engine::rendering::raytracing::Vec3;

// ── Clamping & interpolation ────────────────────────────────────────────

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

// ── Color & perception ──────────────────────────────────────────────────

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

// ── Fresnel & optics ────────────────────────────────────────────────────

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

// ── Hashing & noise ─────────────────────────────────────────────────────

pub fn hash_u32(mut x: u32) -> u32 {
    x = x.wrapping_mul(0x9E3779B9);
    x ^= x >> 16;
    x = x.wrapping_mul(0x85EBCA6B);
    x ^= x >> 13;
    x = x.wrapping_mul(0xC2B2AE35);
    x ^= x >> 16;
    x
}

pub fn hash_to_float(seed: u32) -> f64 {
    (hash_u32(seed) & 0x00FF_FFFF) as f64 / 16_777_215.0
}

pub fn hash_2d(x: i32, y: i32) -> f64 {
    hash_to_float((x as u32).wrapping_mul(1597334673) ^ (y as u32).wrapping_mul(3812015801))
}

pub fn value_noise_2d(x: f64, y: f64) -> f64 {
    let ix = x.floor() as i32;
    let iy = y.floor() as i32;
    let fx = x - x.floor();
    let fy = y - y.floor();
    let ux = fx * fx * (3.0 - 2.0 * fx);
    let uy = fy * fy * (3.0 - 2.0 * fy);

    let c00 = hash_2d(ix, iy);
    let c10 = hash_2d(ix + 1, iy);
    let c01 = hash_2d(ix, iy + 1);
    let c11 = hash_2d(ix + 1, iy + 1);

    lerp(lerp(c00, c10, ux), lerp(c01, c11, ux), uy)
}

pub fn value_noise_3d(point: Vec3) -> f64 {
    let base = value_noise_2d(point.x, point.z);
    let layer = value_noise_2d(point.x + point.y * 0.317, point.z + point.y * 0.719);
    (base + layer) * 0.5
}

pub fn fbm_3d(point: Vec3, octaves: u32, lacunarity: f64, persistence: f64) -> f64 {
    let mut value = 0.0;
    let mut amplitude = 1.0;
    let mut frequency = 1.0;
    let mut max_amplitude = 0.0;

    for _ in 0..octaves {
        value += value_noise_3d(point * frequency) * amplitude;
        max_amplitude += amplitude;
        amplitude *= persistence;
        frequency *= lacunarity;
    }

    if max_amplitude > 0.0 {
        value / max_amplitude
    } else {
        0.0
    }
}

// ── Geometric utilities ─────────────────────────────────────────────────

pub fn spherical_to_cartesian(theta: f64, phi: f64) -> Vec3 {
    Vec3::new(phi.cos() * theta.sin(), theta.cos(), phi.sin() * theta.sin())
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

// ── Tone mapping utilities ──────────────────────────────────────────────

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

// ── Exposure helpers ────────────────────────────────────────────────────

pub fn exposure_from_ev100(ev100: f64) -> f64 {
    1.0 / (1.2 * 2.0_f64.powf(ev100))
}

pub fn ev100_from_luminance(avg_luminance: f64) -> f64 {
    (avg_luminance * 100.0 / 12.5).max(f64::EPSILON).log2()
}
