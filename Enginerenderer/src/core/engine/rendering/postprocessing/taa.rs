use crate::core::engine::rendering::raytracing::Vec3;
use crate::core::engine::rendering::framebuffer::FrameBuffer;

pub struct TaaAccumulator {
    pub history: Vec<Vec3>,
    pub width: usize,
    pub height: usize,
    pub blend_alpha: f64,
    pub variance_clamp: bool,
    pub frame_index: u64,
}

impl TaaAccumulator {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            history: vec![Vec3::ZERO; width * height],
            width,
            height,
            blend_alpha: 0.1,
            variance_clamp: true,
            frame_index: 0,
        }
    }

    pub fn halton_jitter(frame: u64) -> (f64, f64) {
        let jx = halton_sequence(frame as usize + 1, 2) - 0.5;
        let jy = halton_sequence(frame as usize + 1, 3) - 0.5;
        (jx, jy)
    }

    pub fn accumulate(&mut self, current: &mut FrameBuffer) {
        let w = self.width;
        let h = self.height;
        if self.history.len() != w * h {
            self.history = vec![Vec3::ZERO; w * h];
        }

        for y in 0..h {
            for x in 0..w {
                let idx = y * w + x;
                let cur = current.color[idx];

                let history_color = if self.variance_clamp {
                    variance_clamp_history(self.history[idx], current, x, y, w, h)
                } else {
                    self.history[idx]
                };

                let blended = history_color * (1.0 - self.blend_alpha) + cur * self.blend_alpha;
                self.history[idx] = blended;
                current.color[idx] = blended;
            }
        }

        self.frame_index += 1;
    }

    pub fn reset(&mut self) {
        for p in &mut self.history { *p = Vec3::ZERO; }
        self.frame_index = 0;
    }
}

fn variance_clamp_history(
    history: Vec3,
    fb: &FrameBuffer,
    x: usize,
    y: usize,
    w: usize,
    h: usize,
) -> Vec3 {
    let mut m1 = Vec3::ZERO;
    let mut m2 = Vec3::ZERO;
    let mut count = 0usize;

    let x0 = x.saturating_sub(1);
    let x1 = (x + 2).min(w);
    let y0 = y.saturating_sub(1);
    let y1 = (y + 2).min(h);

    for ny in y0..y1 {
        for nx in x0..x1 {
            let s = fb.color[ny * w + nx];
            m1 += s;
            m2 += Vec3::new(s.x * s.x, s.y * s.y, s.z * s.z);
            count += 1;
        }
    }

    let n = count as f64;
    let mean = m1 * (1.0 / n);
    let variance = m2 * (1.0 / n) - Vec3::new(mean.x * mean.x, mean.y * mean.y, mean.z * mean.z);
    let sigma = Vec3::new(variance.x.max(0.0).sqrt(), variance.y.max(0.0).sqrt(), variance.z.max(0.0).sqrt());
    let gamma = 1.25_f64;

    let lo = mean - sigma * gamma;
    let hi = mean + sigma * gamma;

    Vec3::new(
        history.x.max(lo.x).min(hi.x),
        history.y.max(lo.y).min(hi.y),
        history.z.max(lo.z).min(hi.z),
    )
}

fn halton_sequence(mut index: usize, base: usize) -> f64 {
    let mut result = 0.0_f64;
    let mut denom = 1.0_f64;
    while index > 0 {
        denom *= base as f64;
        result += (index % base) as f64 / denom;
        index /= base;
    }
    result
}

pub struct SpatialUpscaler {
    pub scale_factor: u32,
}

impl SpatialUpscaler {
    pub fn new(scale_factor: u32) -> Self {
        Self { scale_factor: scale_factor.max(1) }
    }

    pub fn upscale(&self, src: &FrameBuffer) -> FrameBuffer {
        let scale = self.scale_factor as usize;
        let dst_w = src.width * scale;
        let dst_h = src.height * scale;
        let len = dst_w * dst_h;
        let mut dst = FrameBuffer {
            width: dst_w,
            height: dst_h,
            color: vec![Vec3::ZERO; len],
            alpha: vec![1.0; len],
            depth: vec![f64::INFINITY; len],
            sample_count: vec![1; len],
        };

        for sy in 0..src.height {
            for sx in 0..src.width {
                let src_idx = sy * src.width + sx;
                let c = src.color[src_idx];
                let d = src.depth[src_idx];
                for dy in 0..scale {
                    for dx in 0..scale {
                        let dst_x = sx * scale + dx;
                        let dst_y = sy * scale + dy;
                        let dst_idx = dst_y * dst_w + dst_x;
                        dst.color[dst_idx] = c;
                        dst.depth[dst_idx] = d;
                    }
                }
            }
        }

        dst
    }

    pub fn upscale_bilinear(&self, src: &FrameBuffer) -> FrameBuffer {
        let scale = self.scale_factor as usize;
        let dst_w = src.width * scale;
        let dst_h = src.height * scale;
        let len = dst_w * dst_h;
        let mut dst = FrameBuffer {
            width: dst_w,
            height: dst_h,
            color: vec![Vec3::ZERO; len],
            alpha: vec![1.0; len],
            depth: vec![f64::INFINITY; len],
            sample_count: vec![1; len],
        };

        for dy in 0..dst_h {
            for dx in 0..dst_w {
                let fx = dx as f64 / scale as f64;
                let fy = dy as f64 / scale as f64;
                let x0 = (fx as usize).min(src.width - 1);
                let y0 = (fy as usize).min(src.height - 1);
                let x1 = (x0 + 1).min(src.width - 1);
                let y1 = (y0 + 1).min(src.height - 1);
                let tx = fx - x0 as f64;
                let ty = fy - y0 as f64;

                let c00 = src.color[y0 * src.width + x0];
                let c10 = src.color[y0 * src.width + x1];
                let c01 = src.color[y1 * src.width + x0];
                let c11 = src.color[y1 * src.width + x1];

                let color = c00 * ((1.0 - tx) * (1.0 - ty))
                    + c10 * (tx * (1.0 - ty))
                    + c01 * ((1.0 - tx) * ty)
                    + c11 * (tx * ty);

                dst.color[dy * dst_w + dx] = color;
                dst.depth[dy * dst_w + dx] = src.depth[y0 * src.width + x0];
            }
        }

        dst
    }
}
