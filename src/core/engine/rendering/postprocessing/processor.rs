//! Full-pipeline post-processor with bloom, grain, aberration,
//! sharpening, and tone-mapping.

use crate::core::engine::rendering::raytracing::Vec3;
use crate::core::engine::rendering::framebuffer::FrameBuffer;

use super::blur::separable_blur;
use super::effects::{chromatic_aberration_sample, extract_bright, film_grain, sharpen_pixel};

/// Configurable post-processing pipeline applied after rendering.
#[derive(Debug, Clone, Copy)]
pub struct PostProcessor {
    /// Bloom luminance threshold.
    pub bloom_threshold: f64,
    /// Bloom blur radius (pixels).
    pub bloom_radius: usize,
    /// Bloom blur sigma.
    pub bloom_sigma: f64,
    /// Bloom blend intensity.
    pub bloom_intensity: f64,
    /// Film-grain strength.
    pub grain_intensity: f64,
    /// Chromatic-aberration strength.
    pub aberration_strength: f64,
    /// Sharpening amount.
    pub sharpen_amount: f64,
    /// Exposure multiplier for final tone-map.
    pub exposure: f64,
}

impl PostProcessor {
    /// Cinematic preset with moderate bloom, subtle grain, and mild
    /// aberration.
    pub fn cinematic() -> Self {
        Self {
            bloom_threshold: 1.2,
            bloom_radius: 8,
            bloom_sigma: 3.0,
            bloom_intensity: 0.25,
            grain_intensity: 0.015,
            aberration_strength: 0.003,
            sharpen_amount: 0.3,
            exposure: 1.0,
        }
    }

    /// Sets the bloom threshold.
    pub fn with_bloom_threshold(mut self, t: f64) -> Self {
        self.bloom_threshold = t;
        self
    }

    /// Sets the bloom blur radius.
    pub fn with_bloom_radius(mut self, r: usize) -> Self {
        self.bloom_radius = r;
        self
    }

    /// Sets the bloom intensity.
    pub fn with_bloom_intensity(mut self, i: f64) -> Self {
        self.bloom_intensity = i;
        self
    }

    /// Sets the film-grain intensity.
    pub fn with_grain(mut self, g: f64) -> Self {
        self.grain_intensity = g;
        self
    }

    /// Sets the chromatic-aberration strength.
    pub fn with_aberration(mut self, a: f64) -> Self {
        self.aberration_strength = a;
        self
    }

    /// Sets the sharpening amount.
    pub fn with_sharpen(mut self, s: f64) -> Self {
        self.sharpen_amount = s;
        self
    }

    /// Sets the exposure multiplier.
    pub fn with_exposure(mut self, e: f64) -> Self {
        self.exposure = e;
        self
    }

    /// Runs the full post-processing pipeline on a [`FrameBuffer`].
    ///
    /// Steps (in order):
    /// 1. Extract bright pixels → bloom buffer.
    /// 2. Blur the bloom buffer (separable Gaussian).
    /// 3. Composite bloom back into the frame.
    /// 4. Apply chromatic aberration.
    /// 5. Apply film grain.
    /// 6. Apply sharpening.
    /// 7. Apply exposure (HDR-preserving, no tone-map).
    pub fn apply(&self, fb: &mut FrameBuffer) {
        let w = fb.width;
        let h = fb.height;
        let len = w * h;

        // --- 1. Bloom extraction ---
        let mut bloom: Vec<Vec3> = fb
            .color
            .iter()
            .map(|&c| extract_bright(c, self.bloom_threshold))
            .collect();

        // --- 2. Blur bloom buffer ---
        if self.bloom_radius > 0 {
            separable_blur(&mut bloom, w, h, self.bloom_radius, self.bloom_sigma);
        }

        // --- 3. Composite bloom ---
        for (pixel, bloom_val) in fb.color.iter_mut().zip(bloom.iter()).take(len) {
            *pixel += *bloom_val * self.bloom_intensity;
        }

        // --- 4. Chromatic aberration ---
        if self.aberration_strength > 0.0 {
            let src = fb.color.clone();
            for y in 0..h {
                for x in 0..w {
                    let u = x as f64 / w as f64;
                    let v = y as f64 / h as f64;
                    let (ru, rv) = chromatic_aberration_sample(u, v, self.aberration_strength);
                    let rx = (ru * w as f64).clamp(0.0, (w - 1) as f64) as usize;
                    let ry = (rv * h as f64).clamp(0.0, (h - 1) as f64) as usize;
                    let orig = fb.color[y * w + x];
                    let shifted = src[ry * w + rx];
                    fb.color[y * w + x] = Vec3::new(shifted.x, orig.y, orig.z);
                }
            }
        }

        // --- 5. Film grain ---
        if self.grain_intensity > 0.0 {
            for i in 0..len {
                let noise = pseudo_noise(i as f64);
                fb.color[i] = film_grain(fb.color[i], noise, self.grain_intensity);
            }
        }

        // --- 6. Sharpen ---
        if self.sharpen_amount > 0.0 {
            let mut blurred = fb.color.clone();
            separable_blur(&mut blurred, w, h, 1, 0.8);
            for (pixel, blur) in fb.color.iter_mut().zip(blurred.iter()).take(len) {
                *pixel = sharpen_pixel(*pixel, *blur, self.sharpen_amount);
            }
        }

        // --- 7. Exposure (HDR-preserving, tone-map happens later) ---
        for i in 0..len {
            fb.color[i] = fb.color[i] * self.exposure;
        }
    }

    /// A lighter pipeline that **only** applies bloom and exposure.
    pub fn apply_bloom_only(&self, fb: &mut FrameBuffer) {
        let w = fb.width;
        let h = fb.height;
        let len = w * h;

        let mut bloom: Vec<Vec3> = fb
            .color
            .iter()
            .map(|&c| extract_bright(c, self.bloom_threshold))
            .collect();

        if self.bloom_radius > 0 {
            separable_blur(&mut bloom, w, h, self.bloom_radius, self.bloom_sigma);
        }

        for (pixel, bloom_val) in fb.color.iter_mut().zip(bloom.iter()).take(len) {
            *pixel = (*pixel + *bloom_val * self.bloom_intensity) * self.exposure;
        }
    }
}

/// Simple Reinhard tone-mapping operator.
pub fn reinhard_tonemap(c: Vec3) -> Vec3 {
    Vec3::new(
        c.x / (1.0 + c.x),
        c.y / (1.0 + c.y),
        c.z / (1.0 + c.z),
    )
}

/// Fast integer-seeded pseudo-noise in `[-1, 1]` for grain.
fn pseudo_noise(seed: f64) -> f64 {
    let s = (seed * 12.9898 + 78.233).sin() * 43758.5453;
    s.fract() * 2.0 - 1.0
}
