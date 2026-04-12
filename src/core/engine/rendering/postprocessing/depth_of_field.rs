//! Depth-of-field simulation with circle-of-confusion blur.

use crate::core::engine::rendering::raytracing::Vec3;
use crate::core::engine::rendering::framebuffer::FrameBuffer;

/// Depth-of-field parameters.
#[derive(Debug, Clone, Copy)]
pub struct DepthOfField {
    /// Distance to the sharp focal plane.
    pub focal_distance: f64,
    /// Size of the aperture (larger = more blur).
    pub aperture: f64,
    /// Maximum blur radius in pixels.
    pub max_blur: f64,
}

impl DepthOfField {
    /// Creates a new depth-of-field configuration.
    pub fn new(focal_distance: f64, aperture: f64, max_blur: f64) -> Self {
        Self {
            focal_distance,
            aperture,
            max_blur,
        }
    }

    /// Applies the depth-of-field blur to a [`FrameBuffer`] in place.
    ///
    /// For each pixel the circle-of-confusion radius is computed from
    /// the depth buffer, then a disc-shaped gather blur is performed.
    pub fn apply(&self, fb: &mut FrameBuffer) {
        let w = fb.width;
        let h = fb.height;
        let src_color = fb.color.clone();
        let src_depth = fb.depth.clone();

        for y in 0..h {
            for x in 0..w {
                let idx = y * w + x;
                let depth = src_depth[idx];
                let coc = self.circle_of_confusion(depth);
                let radius = (coc as usize).min(self.max_blur as usize);

                if radius == 0 {
                    continue;
                }

                let mut acc = Vec3::ZERO;
                let mut count = 0.0;

                let r = radius as isize;
                for dy in -r..=r {
                    for dx in -r..=r {
                        if dx * dx + dy * dy > r * r {
                            continue;
                        }
                        let sx = (x as isize + dx).clamp(0, w as isize - 1) as usize;
                        let sy = (y as isize + dy).clamp(0, h as isize - 1) as usize;
                        acc += src_color[sy * w + sx];
                        count += 1.0;
                    }
                }

                if count > 0.0 {
                    fb.color[idx] = acc * (1.0 / count);
                }
            }
        }
    }

    /// Returns the maximum CoC radius across a depth range.
    /// Useful to skip DoF entirely when no pixel would be blurred.
    pub fn max_coc_for_range(&self, depth_min: f64, depth_max: f64) -> f64 {
        self.circle_of_confusion(depth_min)
            .max(self.circle_of_confusion(depth_max))
    }

    /// Computes the circle-of-confusion radius (in pixels) for a
    /// given `depth`.
    fn circle_of_confusion(&self, depth: f64) -> f64 {
        let diff = (depth - self.focal_distance).abs();
        let coc = self.aperture * diff / depth.max(0.001);
        coc.min(self.max_blur)
    }
}
