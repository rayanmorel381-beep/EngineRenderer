use crate::core::engine::math::{Mat4, Vec4};
use crate::core::engine::rendering::raytracing::Vec3;
use super::pipeline::RasterVertex;

pub struct MsaaSample {
    pub x_offset: f32,
    pub y_offset: f32,
    pub weight: f32,
}

pub struct MsaaBuffer {
    pub samples: Vec<Vec<Vec4>>,
    pub resolved: Vec<Vec4>,
    pub sample_count: usize,
    pub width: usize,
    pub height: usize,
}

impl MsaaBuffer {
    pub fn new(width: usize, height: usize, sample_count: usize) -> Self {
        let n = width * height;
        Self {
            samples: vec![vec![Vec4::new(0.0, 0.0, 0.0, 1.0); n]; sample_count],
            resolved: vec![Vec4::new(0.0, 0.0, 0.0, 1.0); n],
            sample_count,
            width,
            height,
        }
    }

    pub fn resolve(&mut self) {
        let n = self.width * self.height;
        let inv = 1.0 / self.sample_count as f32;
        for i in 0..n {
            let mut r = 0.0_f32;
            let mut g = 0.0_f32;
            let mut b = 0.0_f32;
            let mut a = 0.0_f32;
            for s in 0..self.sample_count {
                r += self.samples[s][i].x;
                g += self.samples[s][i].y;
                b += self.samples[s][i].z;
                a += self.samples[s][i].w;
            }
            self.resolved[i] = Vec4::new(r * inv, g * inv, b * inv, a * inv);
        }
    }
}

pub struct MsaaRasterizer {
    pub view_matrix: Mat4,
    pub projection_matrix: Mat4,
    pub model_matrix: Mat4,
}

impl MsaaRasterizer {
    pub fn new() -> Self {
        Self {
            view_matrix: Mat4::IDENTITY,
            projection_matrix: Mat4::IDENTITY,
            model_matrix: Mat4::IDENTITY,
        }
    }

    pub fn render_msaa(&self, vertices: &[RasterVertex], sample_count: usize, width: usize, height: usize) -> MsaaBuffer {
        let mut msaa = MsaaBuffer::new(width, height, sample_count.max(1));
        let offsets = sample_offsets(sample_count.max(1));
        let mvp = self.projection_matrix * self.view_matrix * self.model_matrix;
        let tri_count = vertices.len() / 3;

        for ti in 0..tri_count {
            let v0 = vertices[ti * 3];
            let v1 = vertices[ti * 3 + 1];
            let v2 = vertices[ti * 3 + 2];
            let p0 = transform_clip(&mvp, v0.position);
            let p1 = transform_clip(&mvp, v1.position);
            let p2 = transform_clip(&mvp, v2.position);
            if p0[3] <= 0.0 || p1[3] <= 0.0 || p2[3] <= 0.0 { continue; }

            let ndc = |p: [f32; 4]| [p[0] / p[3], p[1] / p[3], p[2] / p[3]];
            let n0 = ndc(p0); let n1 = ndc(p1); let n2 = ndc(p2);
            let to_screen = |nx: f32, ny: f32| [(nx + 1.0) * 0.5 * width as f32, (1.0 - ny) * 0.5 * height as f32];
            let s0 = to_screen(n0[0], n0[1]);
            let s1 = to_screen(n1[0], n1[1]);
            let s2 = to_screen(n2[0], n2[1]);
            let area = edge_fn(s0, s1, s2);
            if area.abs() < f32::EPSILON { continue; }

            let min_x = s0[0].min(s1[0]).min(s2[0]).max(0.0) as usize;
            let max_x = (s0[0].max(s1[0]).max(s2[0]).ceil() as usize).min(width - 1);
            let min_y = s0[1].min(s1[1]).min(s2[1]).max(0.0) as usize;
            let max_y = (s0[1].max(s1[1]).max(s2[1]).ceil() as usize).min(height - 1);

            for py in min_y..=max_y {
                for px in min_x..=max_x {
                    let idx = py * width + px;
                    for (si, off) in offsets.iter().enumerate() {
                        let sp = [px as f32 + 0.5 + off.x_offset, py as f32 + 0.5 + off.y_offset];
                        let w0 = edge_fn(s1, s2, sp) / area;
                        let w1 = edge_fn(s2, s0, sp) / area;
                        let w2 = edge_fn(s0, s1, sp) / area;
                        if w0 < 0.0 || w1 < 0.0 || w2 < 0.0 { continue; }
                        let nx = (w0 * v0.normal.x as f32) + (w1 * v1.normal.x as f32) + (w2 * v2.normal.x as f32);
                        let ny = (w0 * v0.normal.y as f32) + (w1 * v1.normal.y as f32) + (w2 * v2.normal.y as f32);
                        let nz = (w0 * v0.normal.z as f32) + (w1 * v1.normal.z as f32) + (w2 * v2.normal.z as f32);
                        let shade = (nx + ny + nz) * 0.577_f32;
                        let lit = shade.max(0.0) * 0.8 + 0.2;
                        msaa.samples[si][idx] = Vec4::new(lit * off.weight, lit * off.weight, lit * off.weight, off.weight);
                    }
                }
            }
        }

        msaa.resolve();
        msaa
    }
}

fn transform_clip(m: &Mat4, p: Vec3) -> [f32; 4] {
    let px = p.x as f32;
    let py = p.y as f32;
    let pz = p.z as f32;
    [
        m.m[0][0]*px + m.m[0][1]*py + m.m[0][2]*pz + m.m[0][3],
        m.m[1][0]*px + m.m[1][1]*py + m.m[1][2]*pz + m.m[1][3],
        m.m[2][0]*px + m.m[2][1]*py + m.m[2][2]*pz + m.m[2][3],
        m.m[3][0]*px + m.m[3][1]*py + m.m[3][2]*pz + m.m[3][3],
    ]
}

fn edge_fn(a: [f32; 2], b: [f32; 2], c: [f32; 2]) -> f32 {
    (c[0] - a[0]) * (b[1] - a[1]) - (c[1] - a[1]) * (b[0] - a[0])
}

fn sample_offsets(count: usize) -> Vec<MsaaSample> {
    match count {
        2 => vec![
            MsaaSample { x_offset: -0.25, y_offset: -0.25, weight: 0.5 },
            MsaaSample { x_offset:  0.25, y_offset:  0.25, weight: 0.5 },
        ],
        4 => vec![
            MsaaSample { x_offset: -0.125, y_offset: -0.375, weight: 0.25 },
            MsaaSample { x_offset:  0.375, y_offset: -0.125, weight: 0.25 },
            MsaaSample { x_offset: -0.375, y_offset:  0.125, weight: 0.25 },
            MsaaSample { x_offset:  0.125, y_offset:  0.375, weight: 0.25 },
        ],
        8 => vec![
            MsaaSample { x_offset:  0.0625, y_offset: -0.1875, weight: 0.125 },
            MsaaSample { x_offset: -0.0625, y_offset:  0.1875, weight: 0.125 },
            MsaaSample { x_offset:  0.3125, y_offset:  0.0625, weight: 0.125 },
            MsaaSample { x_offset: -0.1875, y_offset: -0.3125, weight: 0.125 },
            MsaaSample { x_offset: -0.3125, y_offset:  0.3125, weight: 0.125 },
            MsaaSample { x_offset:  0.1875, y_offset: -0.0625, weight: 0.125 },
            MsaaSample { x_offset: -0.4375, y_offset: -0.4375, weight: 0.125 },
            MsaaSample { x_offset:  0.4375, y_offset:  0.4375, weight: 0.125 },
        ],
        _ => vec![MsaaSample { x_offset: 0.0, y_offset: 0.0, weight: 1.0 }],
    }
}
