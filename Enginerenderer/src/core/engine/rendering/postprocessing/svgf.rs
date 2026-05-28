use crate::core::engine::rendering::raytracing::Vec3;
use crate::core::engine::rendering::framebuffer::FrameBuffer;
use super::blur::{gaussian_weights, horizontal_blur, vertical_blur};

pub struct SvgfDenoiser {
    pub history_color: Vec<Vec3>,
    pub history_moments: Vec<[f64; 2]>,
    pub history_depth: Vec<f64>,
    pub history_normal: Vec<Vec3>,
    pub width: usize,
    pub height: usize,
    pub alpha_color: f64,
    pub alpha_moments: f64,
    pub atrous_iterations: usize,
}

pub struct SvgfInput<'a> {
    pub depth: &'a [f64],
    pub normals: &'a [Vec3],
    pub motion_x: &'a [f64],
    pub motion_y: &'a [f64],
}

impl SvgfDenoiser {
    pub fn new(width: usize, height: usize) -> Self {
        let n = width * height;
        Self {
            history_color: vec![Vec3::ZERO; n],
            history_moments: vec![[0.0; 2]; n],
            history_depth: vec![f64::INFINITY; n],
            history_normal: vec![Vec3::ZERO; n],
            width,
            height,
            alpha_color: 0.1,
            alpha_moments: 0.1,
            atrous_iterations: 4,
        }
    }

    pub fn reset(&mut self) {
        let n = self.width * self.height;
        self.history_color = vec![Vec3::ZERO; n];
        self.history_moments = vec![[0.0; 2]; n];
        self.history_depth = vec![f64::INFINITY; n];
        self.history_normal = vec![Vec3::ZERO; n];
    }

    fn luminance(c: Vec3) -> f64 {
        0.2126 * c.x + 0.7152 * c.y + 0.0722 * c.z
    }

    fn reproject(
        &self,
        current_color: &[Vec3],
        input: &SvgfInput<'_>,
    ) -> (Vec<Vec3>, Vec<[f64; 2]>, Vec<bool>) {
        let w = self.width;
        let h = self.height;
        let n = w * h;
        let mut reprojected = vec![Vec3::ZERO; n];
        let mut rep_moments = vec![[0.0f64; 2]; n];
        let mut valid = vec![false; n];

        for y in 0..h {
            for x in 0..w {
                let idx = y * w + x;
                let prev_x = x as f64 - input.motion_x[idx];
                let prev_y = y as f64 - input.motion_y[idx];
                if prev_x < 0.0
                    || prev_x >= (w - 1) as f64
                    || prev_y < 0.0
                    || prev_y >= (h - 1) as f64
                {
                    continue;
                }
                let fx = prev_x as usize;
                let fy = prev_y as usize;
                let tx = prev_x.fract();
                let ty = prev_y.fract();

                let prev_depth = self.history_depth[fy * w + fx];
                let cur_depth = input.depth[idx];
                if (prev_depth - cur_depth).abs() > 0.1 * cur_depth.abs().max(0.001) {
                    continue;
                }
                let prev_n = self.history_normal[fy * w + fx];
                if prev_n.dot(input.normals[idx]) < 0.9 {
                    continue;
                }

                let c00 = self.history_color[fy * w + fx];
                let c10 = if fx + 1 < w { self.history_color[fy * w + fx + 1] } else { c00 };
                let c01 = if fy + 1 < h { self.history_color[(fy + 1) * w + fx] } else { c00 };
                let c11 = if fx + 1 < w && fy + 1 < h {
                    self.history_color[(fy + 1) * w + fx + 1]
                } else {
                    c00
                };

                reprojected[idx] = c00 * ((1.0 - tx) * (1.0 - ty))
                    + c10 * (tx * (1.0 - ty))
                    + c01 * ((1.0 - tx) * ty)
                    + c11 * (tx * ty);

                rep_moments[idx] = self.history_moments[fy * w + fx];
                valid[idx] = true;
            }
        }

        let _ = current_color;
        (reprojected, rep_moments, valid)
    }

    fn atrous_pass(
        color: &[Vec3],
        depth: &[f64],
        normals: &[Vec3],
        variance: &[f64],
        w: usize,
        h: usize,
        step: usize,
    ) -> Vec<Vec3> {
        const SIGMA_L: f64 = 4.0;
        const SIGMA_Z: f64 = 1.0;
        const SIGMA_N: f64 = 128.0;
        let kernel = [3.0_f64 / 8.0, 1.0 / 4.0, 1.0 / 16.0];

        let mut out = vec![Vec3::ZERO; w * h];
        for y in 0..h {
            for x in 0..w {
                let idx = y * w + x;
                let c_c = color[idx];
                let c_d = depth[idx];
                let c_n = normals[idx];
                let c_lum = Self::luminance(c_c);
                let std_dev = variance[idx].sqrt().max(1e-9);

                let mut sum_c = Vec3::ZERO;
                let mut sum_w = 0.0_f64;

                for oy in -2_i32..=2 {
                    for ox in -2_i32..=2 {
                        let nx = x as i32 + ox * step as i32;
                        let ny = y as i32 + oy * step as i32;
                        if nx < 0 || nx >= w as i32 || ny < 0 || ny >= h as i32 {
                            continue;
                        }
                        let nidx = ny as usize * w + nx as usize;
                        let kx = (ox.unsigned_abs() as usize).min(2);
                        let ky = (oy.unsigned_abs() as usize).min(2);
                        let k = kernel[kx] * kernel[ky];

                        let wz = (-(c_d - depth[nidx]).abs() / (SIGMA_Z + 1e-9)).exp();
                        let wn = c_n.dot(normals[nidx]).clamp(0.0, 1.0).powf(SIGMA_N);
                        let wl = (-(c_lum - Self::luminance(color[nidx])).abs()
                            / (SIGMA_L * std_dev + 1e-9))
                            .exp();
                        let w = k * wz * wn * wl;

                        sum_c += color[nidx] * w;
                        sum_w += w;
                    }
                }
                out[idx] = if sum_w > 1e-9 { sum_c * (1.0 / sum_w) } else { c_c };
            }
        }
        out
    }

    pub fn denoise(&mut self, fb: &mut FrameBuffer, input: &SvgfInput<'_>) {
        let w = self.width;
        let h = self.height;
        let n = w * h;

        let (reprojected, rep_moments, valid) = self.reproject(&fb.color, input);

        let mut integrated = vec![Vec3::ZERO; n];
        let mut int_moments = vec![[0.0f64; 2]; n];
        for idx in 0..n {
            let lum = Self::luminance(fb.color[idx]);
            let cur_m = [lum, lum * lum];
            if valid[idx] {
                integrated[idx] = reprojected[idx] * (1.0 - self.alpha_color)
                    + fb.color[idx] * self.alpha_color;
                int_moments[idx] = [
                    rep_moments[idx][0] * (1.0 - self.alpha_moments)
                        + cur_m[0] * self.alpha_moments,
                    rep_moments[idx][1] * (1.0 - self.alpha_moments)
                        + cur_m[1] * self.alpha_moments,
                ];
            } else {
                integrated[idx] = fb.color[idx];
                int_moments[idx] = cur_m;
            }
        }

        let variance: Vec<f64> = int_moments
            .iter()
            .map(|m| (m[1] - m[0] * m[0]).max(0.0))
            .collect();

        let weights = gaussian_weights(1, 1.0);
        let var_vec: Vec<Vec3> = variance.iter().map(|&v| Vec3::splat(v)).collect();
        let mut h_tmp = vec![Vec3::ZERO; n];
        let mut v_tmp = vec![Vec3::ZERO; n];
        horizontal_blur(&var_vec, &mut h_tmp, w, h, &weights);
        vertical_blur(&h_tmp, &mut v_tmp, w, h, &weights);
        let filtered_var: Vec<f64> = v_tmp.iter().map(|v| v.x).collect();

        let mut filtered = integrated.clone();
        for i in 0..self.atrous_iterations {
            filtered =
                Self::atrous_pass(&filtered, input.depth, input.normals, &filtered_var, w, h, 1 << i);
        }

        for (idx, pixel) in fb.color.iter_mut().enumerate() {
            *pixel = filtered[idx];
        }

        self.history_color = integrated;
        self.history_moments = int_moments;
        self.history_depth.clear();
        self.history_depth.extend_from_slice(input.depth);
        self.history_normal.clear();
        self.history_normal.extend_from_slice(input.normals);
    }
}
