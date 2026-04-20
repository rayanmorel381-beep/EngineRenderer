
pub mod accumulation;
pub mod analysis;

use crate::core::engine::rendering::raytracing::{Image, Vec3};

#[derive(Debug, Clone)]
pub struct FrameBuffer {
    pub width: usize,
    pub height: usize,
    pub color: Vec<Vec3>,
    pub alpha: Vec<f64>,
    pub depth: Vec<f64>,
    pub sample_count: Vec<u32>,
}

// ── Conversion ──────────────────────────────────────────────────────────

impl From<Image> for FrameBuffer {
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

    #[inline]
    pub fn pixel_count(&self) -> usize {
        self.width * self.height
    }

    // ── Pixel access ────────────────────────────────────────────────────

    #[inline]
    pub fn get_pixel(&self, x: usize, y: usize) -> Vec3 {
        if x < self.width && y < self.height {
            self.color[y * self.width + x]
        } else {
            Vec3::ZERO
        }
    }

    #[inline]
    pub fn set_pixel(&mut self, x: usize, y: usize, color: Vec3) {
        if x < self.width && y < self.height {
            self.color[y * self.width + x] = color;
        }
    }

    #[inline]
    pub fn get_depth(&self, x: usize, y: usize) -> f64 {
        if x < self.width && y < self.height {
            self.depth[y * self.width + x]
        } else {
            f64::INFINITY
        }
    }

    #[inline]
    pub fn set_depth(&mut self, x: usize, y: usize, d: f64) {
        if x < self.width && y < self.height {
            self.depth[y * self.width + x] = d;
        }
    }

    pub fn into_image(self) -> Image {
        Image {
            width: self.width,
            height: self.height,
            pixels: self.color,
        }
    }

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
