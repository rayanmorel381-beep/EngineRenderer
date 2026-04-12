//! Gaussian blur kernels and separable convolution passes.
//!
//! The blur is performed as two 1-D passes (horizontal then vertical)
//! to keep the cost linear in kernel width.

use crate::core::engine::rendering::raytracing::Vec3;

/// Computes normalised 1-D Gaussian weights for a kernel of
/// `2 * radius + 1` taps.
///
/// Returns an empty vector if `sigma` is not positive.
pub fn gaussian_weights(radius: usize, sigma: f64) -> Vec<f64> {
    if sigma <= 0.0 {
        return Vec::new();
    }
    let mut weights = Vec::with_capacity(2 * radius + 1);
    let mut sum = 0.0;

    for i in 0..=(2 * radius) {
        let x = i as f64 - radius as f64;
        let w = (-x * x / (2.0 * sigma * sigma)).exp();
        weights.push(w);
        sum += w;
    }

    let inv = 1.0 / sum;
    for w in &mut weights {
        *w *= inv;
    }

    weights
}

/// Applies a horizontal 1-D convolution to `src`, writing into
/// `dst`.
///
/// Both buffers are row-major with `width` columns and `height` rows.
pub fn horizontal_blur(
    src: &[Vec3],
    dst: &mut [Vec3],
    width: usize,
    height: usize,
    weights: &[f64],
) {
    let radius = weights.len() / 2;

    for y in 0..height {
        for x in 0..width {
            let mut acc = Vec3::ZERO;
            for (k, &w) in weights.iter().enumerate() {
                let sx = (x as isize + k as isize - radius as isize)
                    .clamp(0, width as isize - 1) as usize;
                acc += src[y * width + sx] * w;
            }
            dst[y * width + x] = acc;
        }
    }
}

/// Applies a vertical 1-D convolution to `src`, writing into `dst`.
pub fn vertical_blur(
    src: &[Vec3],
    dst: &mut [Vec3],
    width: usize,
    height: usize,
    weights: &[f64],
) {
    let radius = weights.len() / 2;

    for y in 0..height {
        for x in 0..width {
            let mut acc = Vec3::ZERO;
            for (k, &w) in weights.iter().enumerate() {
                let sy = (y as isize + k as isize - radius as isize)
                    .clamp(0, height as isize - 1) as usize;
                acc += src[sy * width + x] * w;
            }
            dst[y * width + x] = acc;
        }
    }
}

/// Performs a full separable Gaussian blur (horizontal then vertical)
/// on `buffer` in place, using an intermediate scratch allocation.
pub fn separable_blur(buffer: &mut [Vec3], width: usize, height: usize, radius: usize, sigma: f64) {
    let weights = gaussian_weights(radius, sigma);
    let mut tmp = vec![Vec3::ZERO; buffer.len()];

    horizontal_blur(buffer, &mut tmp, width, height, &weights);
    vertical_blur(&tmp, buffer, width, height, &weights);
}
