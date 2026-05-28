use std::ops::{Add, Mul};
use super::math::Vec3;

pub const BAND_COUNT: usize = 16;
pub const LAMBDA_MIN_NM: f64 = 380.0;
pub const LAMBDA_MAX_NM: f64 = 720.0;
pub const LAMBDA_STEP_NM: f64 = (LAMBDA_MAX_NM - LAMBDA_MIN_NM) / (BAND_COUNT as f64 - 1.0);

#[derive(Debug, Clone, Copy)]
pub struct SpectralSample {
    pub bands: [f64; BAND_COUNT],
}

impl SpectralSample {
    pub const ZERO: Self = Self { bands: [0.0; BAND_COUNT] };

    pub fn wavelength_nm(band: usize) -> f64 {
        LAMBDA_MIN_NM + band as f64 * LAMBDA_STEP_NM
    }

    pub fn from_scalar(v: f64) -> Self {
        Self { bands: [v; BAND_COUNT] }
    }

    pub fn from_blackbody(temperature_k: f64) -> Self {
        let mut bands = [0.0f64; BAND_COUNT];
        for (i, b) in bands.iter_mut().enumerate() {
            let lambda = Self::wavelength_nm(i) * 1e-9;
            let c1 = 3.741_771_9e-16_f64;
            let c2 = 1.438_776_9e-2_f64;
            *b = c1 / (lambda.powi(5) * ((c2 / (lambda * temperature_k)).exp() - 1.0).max(1e-300));
        }
        let peak = bands.iter().cloned().fold(0.0_f64, f64::max).max(1e-300);
        for b in &mut bands { *b /= peak; }
        Self { bands }
    }

    pub fn component_mul(self, other: Self) -> Self {
        let mut bands = [0.0; BAND_COUNT];
        for (i, b) in bands.iter_mut().enumerate() { *b = self.bands[i] * other.bands[i]; }
        Self { bands }
    }

    pub fn scale(self, s: f64) -> Self {
        let mut bands = self.bands;
        for b in &mut bands { *b *= s; }
        Self { bands }
    }

    pub fn to_xyz(&self) -> (f64, f64, f64) {
        let mut x = 0.0_f64;
        let mut y = 0.0_f64;
        let mut z = 0.0_f64;
        for (i, &val) in self.bands.iter().enumerate() {
            let lambda = Self::wavelength_nm(i);
            let (cx, cy, cz) = cie_xyz_cmf(lambda);
            x += cx * val;
            y += cy * val;
            z += cz * val;
        }
        let norm = LAMBDA_STEP_NM;
        (x * norm, y * norm, z * norm)
    }

    pub fn to_rgb(&self) -> Vec3 {
        let (x, y, z) = self.to_xyz();
        let r =  3.2404542 * x - 1.5371385 * y - 0.4985314 * z;
        let g = -0.9692660 * x + 1.8760108 * y + 0.0415560 * z;
        let b =  0.0556434 * x - 0.2040259 * y + 1.0572252 * z;
        Vec3::new(r.max(0.0), g.max(0.0), b.max(0.0))
    }

    pub fn wavelength_dependent_ior(base_ior: f64, band: usize) -> f64 {
        let lambda = Self::wavelength_nm(band);
        let b_coeff = 0.0042;
        base_ior + b_coeff / (lambda * 1e-3 * lambda * 1e-3)
    }

    pub fn rayleigh_weight(band: usize) -> f64 {
        let lambda = Self::wavelength_nm(band) * 1e-9;
        1.0 / (lambda * lambda * lambda * lambda)
    }
}

fn cie_xyz_cmf(lambda_nm: f64) -> (f64, f64, f64) {
    let t = (lambda_nm - LAMBDA_MIN_NM) / (LAMBDA_MAX_NM - LAMBDA_MIN_NM);
    let t = t.clamp(0.0, 1.0);

    let cx = gaussian(lambda_nm, 600.0, 33.0) * 1.056
           + gaussian(lambda_nm, 449.0, 20.0) * 0.362
           - gaussian(lambda_nm, 525.0, 14.5) * 0.065;
    let cy = gaussian(lambda_nm, 559.0, 38.0) * 0.821
           + gaussian(lambda_nm, 445.0, 22.0) * 0.286;
    let cz = gaussian(lambda_nm, 450.0, 22.0) * 1.217
           + gaussian(lambda_nm, 386.0, 16.0) * 0.681;

    let _ = t;
    (cx.max(0.0), cy.max(0.0), cz.max(0.0))
}

#[inline]
fn gaussian(x: f64, mean: f64, std: f64) -> f64 {
    let d = (x - mean) / std;
    (-0.5 * d * d).exp()
}

pub fn rgb_to_spectral(rgb: Vec3) -> SpectralSample {
    let mut bands = [0.0f64; BAND_COUNT];
    for (i, b) in bands.iter_mut().enumerate() {
        let t = i as f64 / (BAND_COUNT as f64 - 1.0);
        let w_r = 0.299 * (1.0 - (t - 0.85).abs().min(0.4) / 0.4);
        let w_g = 0.587 * (1.0 - (t - 0.50).abs().min(0.4) / 0.4);
        let w_b = 0.114 * (1.0 - (t - 0.18).abs().min(0.4) / 0.4);
        let total = (w_r + w_g + w_b).max(1e-9);
        *b = (rgb.x * w_r + rgb.y * w_g + rgb.z * w_b) / total;
    }
    SpectralSample { bands }
}

impl Mul for SpectralSample {
    type Output = Self;
    fn mul(self, other: Self) -> Self { self.component_mul(other) }
}

impl Add for SpectralSample {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        let mut bands = [0.0; BAND_COUNT];
        for (i, b) in bands.iter_mut().enumerate() { *b = self.bands[i] + other.bands[i]; }
        Self { bands }
    }
}

pub struct SpectralTraceConfig {
    pub enabled: bool,
    pub hero_wavelength_mode: bool,
}

impl Default for SpectralTraceConfig {
    fn default() -> Self {
        Self { enabled: true, hero_wavelength_mode: true }
    }
}

pub fn spectral_dispersion_offset(base_ior: f64, band: usize, normal: Vec3, ray_dir: Vec3) -> Vec3 {
    let ior = SpectralSample::wavelength_dependent_ior(base_ior, band);
    let cos_i = (-ray_dir.dot(normal)).max(0.0);
    let sin2_t = (1.0 / ior) * (1.0 / ior) * (1.0 - cos_i * cos_i);
    if sin2_t >= 1.0 { return Vec3::ZERO; }
    let cos_t = (1.0 - sin2_t).sqrt();
    let refracted = ray_dir * (1.0 / ior) + normal * (cos_i / ior - cos_t);
    refracted - ray_dir
}
