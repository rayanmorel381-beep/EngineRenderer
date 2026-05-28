
use crate::core::engine::rendering::raytracing::Vec3;
use crate::core::engine::rendering::framebuffer::FrameBuffer;

#[derive(Debug, Clone, Copy)]
pub struct DepthOfField {
    pub focal_distance: f64,
    pub aperture: f64,
    pub max_blur: f64,
}

impl DepthOfField {
    pub fn new(focal_distance: f64, aperture: f64, max_blur: f64) -> Self {
        Self {
            focal_distance,
            aperture,
            max_blur,
        }
    }

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

    pub fn max_coc_for_range(&self, depth_min: f64, depth_max: f64) -> f64 {
        self.circle_of_confusion(depth_min)
            .max(self.circle_of_confusion(depth_max))
    }

    fn circle_of_confusion(&self, depth: f64) -> f64 {
        let diff = (depth - self.focal_distance).abs();
        let coc = self.aperture * diff / depth.max(0.001);
        coc.min(self.max_blur)
    }
}
