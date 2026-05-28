use crate::core::engine::rendering::raytracing::Vec3;
use super::pipeline::{GBuffer, RasterVertex};

pub struct TileRasterizer {
    pub tile_size: usize,
}

fn transform_vertex(m: &[[f64; 4]; 4], p: Vec3) -> [f64; 4] {
    [
        m[0][0] * p.x + m[0][1] * p.y + m[0][2] * p.z + m[0][3],
        m[1][0] * p.x + m[1][1] * p.y + m[1][2] * p.z + m[1][3],
        m[2][0] * p.x + m[2][1] * p.y + m[2][2] * p.z + m[2][3],
        m[3][0] * p.x + m[3][1] * p.y + m[3][2] * p.z + m[3][3],
    ]
}

fn edge_fn(a: [f64; 2], b: [f64; 2], c: [f64; 2]) -> f64 {
    (c[0] - a[0]) * (b[1] - a[1]) - (c[1] - a[1]) * (b[0] - a[0])
}

struct ScreenTri {
    s: [[f64; 2]; 3],
    z: [f64; 3],
    n: [Vec3; 3],
    area: f64,
}

impl TileRasterizer {
    pub fn new(tile_size: usize) -> Self {
        Self { tile_size: tile_size.max(4) }
    }

    pub fn tile_count(&self, width: usize, height: usize) -> usize {
        let tx = width.div_ceil(self.tile_size);
        let ty = height.div_ceil(self.tile_size);
        tx * ty
    }

    pub fn rasterize_to_gbuffer(
        &self,
        vertices: &[RasterVertex],
        mvp: &[[f64; 4]; 4],
        width: usize,
        height: usize,
    ) -> GBuffer {
        let mut gbuffer = GBuffer::new(width, height);
        if width == 0 || height == 0 || vertices.len() < 3 {
            return gbuffer;
        }

        let tri_count = vertices.len() / 3;
        let tiles_x = width.div_ceil(self.tile_size);
        let tiles_y = height.div_ceil(self.tile_size);

        let screen_tris: Vec<ScreenTri> = (0..tri_count)
            .filter_map(|ti| {
                let v0 = vertices[ti * 3];
                let v1 = vertices[ti * 3 + 1];
                let v2 = vertices[ti * 3 + 2];

                let c0 = transform_vertex(mvp, v0.position);
                let c1 = transform_vertex(mvp, v1.position);
                let c2 = transform_vertex(mvp, v2.position);

                if c0[3] <= 0.0 || c1[3] <= 0.0 || c2[3] <= 0.0 {
                    return None;
                }

                let ndc = |c: [f64; 4]| -> [f64; 3] { [c[0] / c[3], c[1] / c[3], c[2] / c[3]] };
                let n0 = ndc(c0);
                let n1 = ndc(c1);
                let n2 = ndc(c2);

                let to_screen = |nx: f64, ny: f64| -> [f64; 2] {
                    [
                        (nx + 1.0) * 0.5 * width as f64,
                        (1.0 - ny) * 0.5 * height as f64,
                    ]
                };
                let s0 = to_screen(n0[0], n0[1]);
                let s1 = to_screen(n1[0], n1[1]);
                let s2 = to_screen(n2[0], n2[1]);

                let area = edge_fn(s0, s1, s2);
                if area.abs() < f64::EPSILON {
                    return None;
                }

                Some(ScreenTri {
                    s: [s0, s1, s2],
                    z: [n0[2], n1[2], n2[2]],
                    n: [v0.normal, v1.normal, v2.normal],
                    area,
                })
            })
            .collect();

        for ty in 0..tiles_y {
            for tx in 0..tiles_x {
                let tile_x0 = tx * self.tile_size;
                let tile_y0 = ty * self.tile_size;
                let tile_x1 = (tile_x0 + self.tile_size).min(width);
                let tile_y1 = (tile_y0 + self.tile_size).min(height);

                for tri in &screen_tris {
                    let [s0, s1, s2] = tri.s;
                    let tri_min_x = s0[0].min(s1[0]).min(s2[0]);
                    let tri_max_x = s0[0].max(s1[0]).max(s2[0]);
                    let tri_min_y = s0[1].min(s1[1]).min(s2[1]);
                    let tri_max_y = s0[1].max(s1[1]).max(s2[1]);

                    if tri_max_x < tile_x0 as f64
                        || tri_min_x >= tile_x1 as f64
                        || tri_max_y < tile_y0 as f64
                        || tri_min_y >= tile_y1 as f64
                    {
                        continue;
                    }

                    let px_min = (tri_min_x as usize).max(tile_x0);
                    let px_max = (tri_max_x.ceil() as usize + 1).min(tile_x1);
                    let py_min = (tri_min_y as usize).max(tile_y0);
                    let py_max = (tri_max_y.ceil() as usize + 1).min(tile_y1);

                    for py in py_min..py_max {
                        for px in px_min..px_max {
                            let sp = [px as f64 + 0.5, py as f64 + 0.5];
                            let w0 = edge_fn(s1, s2, sp) / tri.area;
                            let w1 = edge_fn(s2, s0, sp) / tri.area;
                            let w2 = edge_fn(s0, s1, sp) / tri.area;
                            if w0 < 0.0 || w1 < 0.0 || w2 < 0.0 {
                                continue;
                            }
                            let depth = w0 * tri.z[0] + w1 * tri.z[1] + w2 * tri.z[2];
                            let idx = py * width + px;
                            if depth >= gbuffer.depth[idx] {
                                continue;
                            }
                            gbuffer.depth[idx] = depth;
                            let nx = w0 * tri.n[0].x + w1 * tri.n[1].x + w2 * tri.n[2].x;
                            let ny = w0 * tri.n[0].y + w1 * tri.n[1].y + w2 * tri.n[2].y;
                            let nz = w0 * tri.n[0].z + w1 * tri.n[1].z + w2 * tri.n[2].z;
                            let len = (nx * nx + ny * ny + nz * nz).sqrt().max(f64::EPSILON);
                            gbuffer.normal[idx] = Vec3::new(nx / len, ny / len, nz / len);
                            gbuffer.albedo[idx] = [
                                nx / len * 0.5 + 0.5,
                                ny / len * 0.5 + 0.5,
                                nz / len * 0.5 + 0.5,
                                1.0,
                            ];
                        }
                    }
                }
            }
        }

        gbuffer
    }
}
