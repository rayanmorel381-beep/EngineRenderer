//! Multi-channel render-target (colour, alpha, depth, sample count).
//!
//! [`FrameBuffer`] is the engine's primary render target.  It stores
//! colour, alpha, depth, and per-pixel sample counts and supports
//! conversion to/from the simpler [`Image`](crate::core::engine::rendering::raytracing::Image).
//!
//! * [`accumulation`]  — progressive rendering, merging, tile I/O.
//! * [`analysis`]      — luminance stats, histogram, auto-exposure, depth viz.

pub mod accumulation;
pub mod analysis;

use crate::core::engine::rendering::raytracing::{Image, Vec3};

/// Multi-channel render target: colour (RGB), alpha, depth, and sample count.
///
/// Indexing follows row-major order: `pixel[y * width + x]`.
///
/// # Construction
/// ```ignore
/// // Empty buffer
/// let mut fb = FrameBuffer::new(1920, 1080);
///
/// // From a rendered Image
/// let fb = FrameBuffer::from(image);
/// ```
#[derive(Debug, Clone)]
pub struct FrameBuffer {
    /// Horizontal resolution in pixels.
    pub width: usize,
    /// Vertical resolution in pixels.
    pub height: usize,
    /// Per-pixel RGB colour (linear, HDR).
    pub color: Vec<Vec3>,
    /// Per-pixel alpha (default `1.0` = fully opaque).
    pub alpha: Vec<f64>,
    /// Per-pixel closest hit distance (default `∞`).
    pub depth: Vec<f64>,
    /// Number of accumulated samples per pixel (for progressive rendering).
    pub sample_count: Vec<u32>,
}

// ── Conversion ──────────────────────────────────────────────────────────

impl From<Image> for FrameBuffer {
    /// Wraps an [`Image`] into a full `FrameBuffer`, initialising alpha,
    /// depth, and sample count to sensible defaults.
    fn from(image: Image) -> Self {
        let len = image.pixels.len();
        Self {
            width: image.width,
            height: image.height,
            color: image.pixels,
            alpha: vec![1.0; len],
            depth: vec![f64::INFINITY; len],
            sample_count: vec![1; len],
        }
    }
}

impl FrameBuffer {
    /// Creates a blank framebuffer with all channels zero / default.
    pub fn new(width: usize, height: usize) -> Self {
        let len = width * height;
        Self {
            width,
            height,
            color: vec![Vec3::ZERO; len],
            alpha: vec![1.0; len],
            depth: vec![f64::INFINITY; len],
            sample_count: vec![0; len],
        }
    }

    /// Total number of pixels (`width × height`).
    #[inline]
    pub fn pixel_count(&self) -> usize {
        self.width * self.height
    }

    // ── Pixel access ────────────────────────────────────────────────────

    /// Reads the colour at `(x, y)`.  Returns `Vec3::ZERO` for
    /// out-of-bounds coordinates.
    #[inline]
    pub fn get_pixel(&self, x: usize, y: usize) -> Vec3 {
        if x < self.width && y < self.height {
            self.color[y * self.width + x]
        } else {
            Vec3::ZERO
        }
    }

    /// Writes `color` at `(x, y)`.  Out-of-bounds writes are silently
    /// ignored.
    #[inline]
    pub fn set_pixel(&mut self, x: usize, y: usize, color: Vec3) {
        if x < self.width && y < self.height {
            self.color[y * self.width + x] = color;
        }
    }

    /// Reads the depth value at `(x, y)`.  Returns `INFINITY` for
    /// out-of-bounds.
    #[inline]
    pub fn get_depth(&self, x: usize, y: usize) -> f64 {
        if x < self.width && y < self.height {
            self.depth[y * self.width + x]
        } else {
            f64::INFINITY
        }
    }

    /// Writes depth `d` at `(x, y)`.
    #[inline]
    pub fn set_depth(&mut self, x: usize, y: usize, d: f64) {
        if x < self.width && y < self.height {
            self.depth[y * self.width + x] = d;
        }
    }

    /// Consumes this framebuffer, returning a plain [`Image`] (RGB only).
    pub fn into_image(self) -> Image {
        Image {
            width: self.width,
            height: self.height,
            pixels: self.color,
        }
    }

    /// Resets all channels to their default state (black, alpha 1,
    /// depth ∞, sample count 0).
    pub fn clear(&mut self) {
        for pixel in &mut self.color {
            *pixel = Vec3::ZERO;
        }
        for a in &mut self.alpha {
            *a = 1.0;
        }
        for d in &mut self.depth {
            *d = f64::INFINITY;
        }
        for s in &mut self.sample_count {
            *s = 0;
        }
    }
}
