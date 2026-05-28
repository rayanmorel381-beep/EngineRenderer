
use crate::core::engine::rendering::raytracing::Vec3;
use crate::core::engine::rendering::utils::luminance;

use super::FrameBuffer;

impl FrameBuffer {
    // ── Luminance analysis ──────────────────────────────────────────────

    /// Computes average luminance over all pixels.
    pub fn average_luminance(&self) -> f64 {
        if self.color.is_empty() {
            0.0
        } else {
            self.color.iter().map(|pixel| luminance(*pixel)).sum::<f64>()
                / self.color.len() as f64
        }
    }

    /// Computes logarithmic average luminance.
    pub fn log_average_luminance(&self) -> f64 {
        if self.color.is_empty() {
            return 0.0;
        }
        let delta = 0.0001;
        let sum: f64 = self
            .color
            .iter()
            .map(|pixel| (luminance(*pixel) + delta).ln())
            .sum();
        (sum / self.color.len() as f64).exp()
    }

    /// Returns minimum and maximum luminance.
    pub fn luminance_range(&self) -> (f64, f64) {
        if self.color.is_empty() {
            (0.0, 0.0)
        } else {
            self.color.iter().fold(
                (f64::INFINITY, f64::NEG_INFINITY),
                |(min_luma, max_luma), pixel| {
                    let luma = luminance(*pixel);
                    (min_luma.min(luma), max_luma.max(luma))
                },
            )
        }
    }

    /// Returns the brightest pixel by luminance.
    pub fn brightest_pixel(&self) -> Vec3 {
        self.color
            .iter()
            .copied()
            .max_by(|left, right| {
                luminance(*left)
                    .partial_cmp(&luminance(*right))
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .unwrap_or(Vec3::ZERO)
    }

    // ── Histogram ───────────────────────────────────────────────────────

    /// Builds a luminance histogram with the requested bin count.
    pub fn luminance_histogram(&self, bins: usize) -> Vec<u32> {
        let bins = bins.max(1);
        let mut histogram = vec![0u32; bins];
        let (min_l, max_l) = self.luminance_range();
        let range = (max_l - min_l).max(f64::EPSILON);

        for pixel in &self.color {
            let luma = luminance(*pixel);
            let normalized = ((luma - min_l) / range).clamp(0.0, 0.9999);
            let bin = (normalized * bins as f64) as usize;
            histogram[bin.min(bins - 1)] += 1;
        }

        histogram
    }

    /// Estimates luminance at a percentile in [0, 1].
    pub fn percentile_luminance(&self, percentile: f64) -> f64 {
        let histogram = self.luminance_histogram(256);
        let target = (self.color.len() as f64 * percentile.clamp(0.0, 1.0)) as u32;
        let (min_l, max_l) = self.luminance_range();
        let range = (max_l - min_l).max(f64::EPSILON);

        let mut cumulative = 0u32;
        for (i, &count) in histogram.iter().enumerate() {
            cumulative += count;
            if cumulative >= target {
                return min_l + range * (i as f64 / 256.0);
            }
        }
        max_l
    }

    // ── Auto exposure ───────────────────────────────────────────────────

    /// Computes auto-exposure factor targeting mid-gray.
    pub fn auto_exposure(&self, target_mid_gray: f64, min_ev: f64, max_ev: f64) -> f64 {
        let avg = self.log_average_luminance().max(0.0001);
        let ev = (target_mid_gray / avg).log2().clamp(min_ev, max_ev);
        2.0_f64.powf(ev)
    }

    /// Applies an exposure factor to all color pixels.
    pub fn apply_exposure(&mut self, factor: f64) {
        for pixel in &mut self.color {
            *pixel = *pixel * factor;
        }
    }

    // ── Depth buffer utilities ──────────────────────────────────────────

    /// Returns minimum and maximum finite depth values.
    pub fn depth_range(&self) -> (f64, f64) {
        let mut min_d = f64::INFINITY;
        let mut max_d = f64::NEG_INFINITY;
        for &d in &self.depth {
            if d < f64::INFINITY {
                min_d = min_d.min(d);
                max_d = max_d.max(d);
            }
        }
        (min_d, max_d)
    }

    /// Converts depth values to grayscale colors.
    pub fn depth_to_color(&self) -> Vec<Vec3> {
        let (min_d, max_d) = self.depth_range();
        let range = (max_d - min_d).max(f64::EPSILON);
        self.depth
            .iter()
            .map(|&d| {
                if d >= f64::INFINITY {
                    Vec3::ZERO
                } else {
                    let normalized = ((d - min_d) / range).clamp(0.0, 1.0);
                    Vec3::splat(1.0 - normalized)
                }
            })
            .collect()
    }
}
