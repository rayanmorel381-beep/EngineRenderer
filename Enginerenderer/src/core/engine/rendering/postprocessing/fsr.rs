use crate::core::engine::rendering::raytracing::Vec3;
use crate::core::engine::rendering::framebuffer::FrameBuffer;

#[derive(Debug, Clone, Copy)]
pub struct FsrConfig {
    pub input_width: usize,
    pub input_height: usize,
    pub output_width: usize,
    pub output_height: usize,
    pub sharpness: f64,
}

impl FsrConfig {
    pub fn new(input_width: usize, input_height: usize, output_width: usize, output_height: usize) -> Self {
        Self { input_width, input_height, output_width, output_height, sharpness: 0.75 }
    }

    pub fn scale_factor_x(&self) -> f64 { self.output_width as f64 / self.input_width as f64 }
    pub fn scale_factor_y(&self) -> f64 { self.output_height as f64 / self.input_height as f64 }
}

pub struct FsrPass;

impl FsrPass {
    pub fn upscale(src: &FrameBuffer, config: FsrConfig) -> FrameBuffer {
        let out_w = config.output_width;
        let out_h = config.output_height;
        let mut out = FrameBuffer {
            width: out_w,
            height: out_h,
            color: vec![Vec3::ZERO; out_w * out_h],
            alpha: vec![1.0; out_w * out_h],
            depth: vec![1.0; out_w * out_h],
            sample_count: vec![1; out_w * out_h],
        };

        for y in 0..out_h {
            for x in 0..out_w {
                let src_x = x as f64 / config.scale_factor_x();
                let src_y = y as f64 / config.scale_factor_y();
                let color = easu_sample(src, src_x, src_y);
                out.color[y * out_w + x] = color;
            }
        }

        rcas_pass(&out.clone(), &mut out, config.sharpness);
        out
    }
}

fn easu_sample(src: &FrameBuffer, fx: f64, fy: f64) -> Vec3 {
    let w = src.width;
    let h = src.height;
    let x0 = fx.floor() as i64;
    let y0 = fy.floor() as i64;
    let tx = fx - x0 as f64;
    let ty = fy - y0 as f64;

    let mut sum = Vec3::ZERO;
    let mut weight_total = 0.0_f64;

    for dy in -1i64..=2 {
        for dx in -1i64..=2 {
            let sx = (x0 + dx).clamp(0, w as i64 - 1) as usize;
            let sy = (y0 + dy).clamp(0, h as i64 - 1) as usize;
            let sample = src.color[sy * w + sx];
            let luma = luminance(sample);

            let kx = lanczos2(tx - dx as f64);
            let ky = lanczos2(ty - dy as f64);
            let edge_w = 1.0 + luma * 0.2;
            let w = kx * ky * edge_w;

            sum += sample * w;
            weight_total += w;
        }
    }

    if weight_total > 1e-6 { sum * (1.0 / weight_total) } else { Vec3::ZERO }
}

fn rcas_pass(src: &FrameBuffer, dst: &mut FrameBuffer, sharpness: f64) {
    let w = src.width;
    let h = src.height;
    let rcas_limit = 0.25 - 0.25 * (1.0 - sharpness.clamp(0.0, 1.0));

    for y in 0..h {
        for x in 0..w {
            let get = |dx: i64, dy: i64| -> Vec3 {
                let sx = (x as i64 + dx).clamp(0, w as i64 - 1) as usize;
                let sy = (y as i64 + dy).clamp(0, h as i64 - 1) as usize;
                src.color[sy * w + sx]
            };

            let e = get(0, 0);
            let n = get(0, -1);
            let s = get(0, 1);
            let ww = get(-1, 0);
            let ee = get(1, 0);

            let luma_e = luminance(e);
            let luma_n = luminance(n);
            let luma_s = luminance(s);
            let luma_w = luminance(ww);
            let luma_ee = luminance(ee);

            let luma_min = luma_e.min(luma_n).min(luma_s).min(luma_w).min(luma_ee);
            let luma_max = luma_e.max(luma_n).max(luma_s).max(luma_w).max(luma_ee);

            let amp = (luma_min / luma_max.max(1e-6) - 0.5).abs().min(1.0).sqrt();
            let rcas_w = -(amp * rcas_limit);
            let rcas_denom = 4.0 * rcas_w + 1.0;

            if rcas_denom.abs() < 1e-6 {
                dst.color[y * w + x] = e;
                continue;
            }

            let sharpened = (e + (n + s + ww + ee) * rcas_w) * (1.0 / rcas_denom);
            dst.color[y * w + x] = sharpened.clamp(0.0, 1e6);
        }
    }
}

fn lanczos2(x: f64) -> f64 {
    use std::f64::consts::PI;
    if x.abs() < 1e-6 { return 1.0; }
    if x.abs() >= 2.0 { return 0.0; }
    let px = PI * x;
    let px2 = PI * x * 0.5;
    px.sin() * px2.sin() / (px * px2)
}

fn luminance(c: Vec3) -> f64 {
    c.x * 0.2126 + c.y * 0.7152 + c.z * 0.0722
}
