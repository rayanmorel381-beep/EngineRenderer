
use crate::core::engine::rendering::raytracing::Vec3;

use super::FrameBuffer;

impl FrameBuffer {
    // ── Progressive accumulation ────────────────────────────────────────

    /// Accumulates one sample into a pixel and tracks nearest depth.
    pub fn accumulate(&mut self, x: usize, y: usize, color: Vec3, depth: f64) {
        if x >= self.width || y >= self.height {
            return;
        }
        let idx = y * self.width + x;
        let n = self.sample_count[idx] as f64;
        self.color[idx] = (self.color[idx] * n + color) / (n + 1.0);
        if depth < self.depth[idx] {
            self.depth[idx] = depth;
        }
        self.sample_count[idx] += 1;
    }

    /// Merges another framebuffer into this one using sample-count weighting.
    pub fn merge(&mut self, other: &FrameBuffer) {
        if self.width != other.width || self.height != other.height {
            return;
        }
        for i in 0..self.color.len() {
            let n1 = self.sample_count[i] as f64;
            let n2 = other.sample_count[i] as f64;
            let total = n1 + n2;
            if total > 0.0 {
                self.color[i] = (self.color[i] * n1 + other.color[i] * n2) / total;
                self.sample_count[i] += other.sample_count[i];
            }
            if other.depth[i] < self.depth[i] {
                self.depth[i] = other.depth[i];
            }
        }
    }

    // ── Tile access ─────────────────────────────────────────────────────

    /// Returns a copy of pixel colors inside a tile region.
    pub fn tile_region(
        &self,
        tile_x: usize,
        tile_y: usize,
        tile_w: usize,
        tile_h: usize,
    ) -> Vec<Vec3> {
        let mut pixels = Vec::with_capacity(tile_w * tile_h);
        for y in tile_y..(tile_y + tile_h).min(self.height) {
            for x in tile_x..(tile_x + tile_w).min(self.width) {
                pixels.push(self.color[y * self.width + x]);
            }
        }
        pixels
    }

    /// Writes a tile of pixel colors into the framebuffer.
    pub fn write_tile(
        &mut self,
        tile_x: usize,
        tile_y: usize,
        tile_w: usize,
        tile_h: usize,
        pixels: &[Vec3],
    ) {
        let mut idx = 0;
        for y in tile_y..(tile_y + tile_h).min(self.height) {
            for x in tile_x..(tile_x + tile_w).min(self.width) {
                if idx < pixels.len() {
                    self.color[y * self.width + x] = pixels[idx];
                    idx += 1;
                }
            }
        }
    }
}
