//! Progressive rendering accumulation and tile-based buffer access.
//!
//! Extends [`FrameBuffer`] with methods for incremental sample
//! accumulation, buffer merging, and region-based read/write (tiles).

use crate::core::engine::rendering::raytracing::Vec3;

use super::FrameBuffer;

impl FrameBuffer {
    // ── Progressive accumulation ────────────────────────────────────────

    /// Accumulates a new `color` sample at `(x, y)`, running a weighted
    /// average with all previous samples.
    ///
    /// The depth buffer is updated only when the new sample is closer
    /// than the existing value.
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

    /// Merges `other` into `self` using per-pixel sample-count weighting.
    ///
    /// Both buffers must have matching dimensions; a mismatch is a silent
    /// no-op.
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

    /// Extracts a rectangular sub-region of the colour buffer.
    ///
    /// The returned `Vec` is in row-major order and may be shorter than
    /// `tile_w × tile_h` when the tile exceeds the buffer edges.
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

    /// Writes `pixels` (row-major) into the colour buffer starting at
    /// `(tile_x, tile_y)` for a region of `tile_w × tile_h`.
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
