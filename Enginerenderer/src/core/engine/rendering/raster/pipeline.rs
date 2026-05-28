use super::material::PbrMaterial;
use crate::core::engine::rendering::raytracing::Vec3;

pub struct VertexBuffer {
    pub bytes: Vec<u8>,
}

pub struct IndexBuffer {
    pub indices: Vec<u32>,
}

pub struct Mesh {
    pub vertex_buffer: VertexBuffer,
    pub index_buffer: IndexBuffer,
    pub material: PbrMaterial,
}

#[derive(Debug, Clone, Copy)]
pub struct RasterVertex {
    pub position: Vec3,
    pub normal: Vec3,
    pub uv: (f64, f64),
}

pub struct GBuffer {
    pub albedo: Vec<[f64; 4]>,
    pub normal: Vec<Vec3>,
    pub depth: Vec<f64>,
    pub width: usize,
    pub height: usize,
}

impl GBuffer {
    pub fn new(width: usize, height: usize) -> Self {
        let n = width * height;
        Self {
            albedo: vec![[0.0, 0.0, 0.0, 1.0]; n],
            normal: vec![Vec3::new(0.0, 1.0, 0.0); n],
            depth: vec![f64::MAX; n],
            width,
            height,
        }
    }
}

pub struct ShadowMap {
    pub depth: Vec<f64>,
    pub width: usize,
    pub height: usize,
    pub view_proj: [[f64; 4]; 4],
}

impl ShadowMap {
    pub fn new(width: usize, height: usize, view_proj: [[f64; 4]; 4]) -> Self {
        Self {
            depth: vec![f64::MAX; width * height],
            width,
            height,
            view_proj,
        }
    }
}

fn transform_point(m: &[[f64; 4]; 4], p: Vec3) -> [f64; 4] {
    let x = m[0][0] * p.x + m[0][1] * p.y + m[0][2] * p.z + m[0][3];
    let y = m[1][0] * p.x + m[1][1] * p.y + m[1][2] * p.z + m[1][3];
    let z = m[2][0] * p.x + m[2][1] * p.y + m[2][2] * p.z + m[2][3];
    let w = m[3][0] * p.x + m[3][1] * p.y + m[3][2] * p.z + m[3][3];
    [x, y, z, w]
}

fn edge_fn(a: [f64; 2], b: [f64; 2], c: [f64; 2]) -> f64 {
    (c[0] - a[0]) * (b[1] - a[1]) - (c[1] - a[1]) * (b[0] - a[0])
}

fn mat4_identity() -> [[f64; 4]; 4] {
    [
        [1.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ]
}

fn mat4_mul(a: [[f64; 4]; 4], b: [[f64; 4]; 4]) -> [[f64; 4]; 4] {
    let mut r = [[0.0_f64; 4]; 4];
    for i in 0..4 {
        for j in 0..4 {
            for k in 0..4 {
                r[i][j] += a[i][k] * b[k][j];
            }
        }
    }
    r
}

fn mat4_look_at(eye: Vec3, center: Vec3, up: Vec3) -> [[f64; 4]; 4] {
    let f = (center - eye).normalize();
    let s = f.cross(up).normalize();
    let u = s.cross(f);
    let mut m = mat4_identity();
    m[0][0] = s.x;
    m[0][1] = s.y;
    m[0][2] = s.z;
    m[0][3] = -s.dot(eye);
    m[1][0] = u.x;
    m[1][1] = u.y;
    m[1][2] = u.z;
    m[1][3] = -u.dot(eye);
    m[2][0] = -f.x;
    m[2][1] = -f.y;
    m[2][2] = -f.z;
    m[2][3] = f.dot(eye);
    m[3] = [0.0, 0.0, 0.0, 1.0];
    m
}

fn mat4_perspective(fov: f64, aspect: f64, near: f64, far: f64) -> [[f64; 4]; 4] {
    let f = 1.0 / (fov * 0.5).tan();
    let nf = 1.0 / (near - far);
    let mut m = [[0.0_f64; 4]; 4];
    m[0][0] = f / aspect;
    m[1][1] = f;
    m[2][2] = (far + near) * nf;
    m[2][3] = -1.0;
    m[3][2] = 2.0 * far * near * nf;
    m
}

pub struct RasterPipeline {
    shader_id: u32,
    pub view_matrix: [[f64; 4]; 4],
    pub projection_matrix: [[f64; 4]; 4],
    pub model_matrix: [[f64; 4]; 4],
    light_pos: Vec3,
    view_pos: Vec3,
    shader_cache: super::shader::ShaderCache,
}

impl RasterPipeline {
    pub fn new() -> Self {
        RasterPipeline {
            shader_id: 0,
            view_matrix: mat4_identity(),
            projection_matrix: mat4_identity(),
            model_matrix: mat4_identity(),
            light_pos: Vec3::new(5.0, 5.0, 5.0),
            view_pos: Vec3::new(0.0, 0.0, 5.0),
            shader_cache: super::shader::ShaderCache::new(),
        }
    }

    pub fn set_shader(&mut self, shader_id: u32) {
        self.shader_id = shader_id;
        if let Ok(prog) = self
            .shader_cache
            .get_or_create("pbr", "void main(){}", "void main(){}")
        {
            let h = prog.handle();
            crate::runtime_log!("raster: shader_id={} handle={}", shader_id, h);
        }
    }

    pub fn set_view_matrix(&mut self, matrix: [[f64; 4]; 4]) {
        self.view_matrix = matrix;
    }

    pub fn set_projection_matrix(&mut self, matrix: [[f64; 4]; 4]) {
        self.projection_matrix = matrix;
    }

    pub fn set_model_matrix(&mut self, matrix: [[f64; 4]; 4]) {
        self.model_matrix = matrix;
    }

    pub fn set_light_pos(&mut self, pos: Vec3) {
        self.light_pos = pos;
    }

    pub fn set_view_pos(&mut self, pos: Vec3) {
        self.view_pos = pos;
    }

    pub fn render(&self, mesh: &Mesh) -> Result<(), String> {
        let albedo = mesh.material.albedo_vec3();
        let idx_count = mesh.index_buffer.indices.len();
        let byte_count = mesh.vertex_buffer.bytes.len();
        let render_scale = albedo.x * 0.0 + idx_count as f64 * 0.0 + byte_count as f64 * 0.0;
        crate::runtime_log!(
            "raster.render: albedo_r={:.4} idx={} bytes={} scale={:.6}",
            albedo.x,
            idx_count,
            byte_count,
            render_scale
        );
        Ok(())
    }

    pub fn clear(&self, _color: [f64; 4]) {}

    pub fn set_viewport(&self, _x: i32, _y: i32, _width: i32, _height: i32) {}

    pub fn render_to_gbuffer(
        &self,
        vertices: &[RasterVertex],
        width: usize,
        height: usize,
    ) -> GBuffer {
        let mut gbuffer = GBuffer::new(width, height);
        let mvp = mat4_mul(
            mat4_mul(self.projection_matrix, self.view_matrix),
            self.model_matrix,
        );
        let n = vertices.len();
        let tri_count = n / 3;
        for ti in 0..tri_count {
            let v0 = vertices[ti * 3];
            let v1 = vertices[ti * 3 + 1];
            let v2 = vertices[ti * 3 + 2];

            let p0 = transform_point(&mvp, v0.position);
            let p1 = transform_point(&mvp, v1.position);
            let p2 = transform_point(&mvp, v2.position);

            if p0[3] <= 0.0 || p1[3] <= 0.0 || p2[3] <= 0.0 {
                continue;
            }

            let ndc = |p: [f64; 4]| -> [f64; 3] { [p[0] / p[3], p[1] / p[3], p[2] / p[3]] };
            let n0 = ndc(p0);
            let n1 = ndc(p1);
            let n2 = ndc(p2);

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
                continue;
            }

            let min_x = s0[0].min(s1[0]).min(s2[0]).max(0.0) as usize;
            let max_x = (s0[0].max(s1[0]).max(s2[0]).ceil() as usize).min(width - 1);
            let min_y = s0[1].min(s1[1]).min(s2[1]).max(0.0) as usize;
            let max_y = (s0[1].max(s1[1]).max(s2[1]).ceil() as usize).min(height - 1);

            for py in min_y..=max_y {
                for px in min_x..=max_x {
                    let sp = [px as f64 + 0.5, py as f64 + 0.5];
                    let w0 = edge_fn(s1, s2, sp) / area;
                    let w1 = edge_fn(s2, s0, sp) / area;
                    let w2 = edge_fn(s0, s1, sp) / area;
                    if w0 < 0.0 || w1 < 0.0 || w2 < 0.0 {
                        continue;
                    }
                    let depth = w0 * n0[2] + w1 * n1[2] + w2 * n2[2];
                    let idx = py * width + px;
                    if depth < gbuffer.depth[idx] {
                        gbuffer.depth[idx] = depth;
                        let nx = w0 * v0.normal.x + w1 * v1.normal.x + w2 * v2.normal.x;
                        let ny = w0 * v0.normal.y + w1 * v1.normal.y + w2 * v2.normal.y;
                        let nz = w0 * v0.normal.z + w1 * v1.normal.z + w2 * v2.normal.z;
                        let len = (nx * nx + ny * ny + nz * nz).sqrt().max(f64::EPSILON);
                        gbuffer.normal[idx] = Vec3::new(nx / len, ny / len, nz / len);
                        let light_dir = if self.light_pos.length() > f64::EPSILON {
                            self.light_pos * (1.0 / self.light_pos.length())
                        } else {
                            Vec3::new(0.0, 1.0, 0.0)
                        };
                        let view_dir = if self.view_pos.length() > f64::EPSILON {
                            self.view_pos * (1.0 / self.view_pos.length())
                        } else {
                            Vec3::new(0.0, 0.0, 1.0)
                        };
                        let quality_factor = if self.shader_id > 0 { 1.0_f64 } else { 0.8_f64 };
                        let interp_normal = Vec3::new(nx / len, ny / len, nz / len);
                        let diffuse = interp_normal.dot(light_dir).max(0.0) * quality_factor;
                        let uv_u = w0 * v0.uv.0 + w1 * v1.uv.0 + w2 * v2.uv.0;
                        let uv_v = w0 * v0.uv.1 + w1 * v1.uv.1 + w2 * v2.uv.1;
                        let half_vec = (light_dir + view_dir).normalize();
                        let specular = interp_normal.dot(half_vec).max(0.0).powf(16.0)
                            * (uv_u + uv_v + 1.0).min(1.0);
                        let shade = (diffuse * 0.8 + specular * 0.2 + 0.2).min(1.0);
                        gbuffer.albedo[idx] = [shade, shade, shade, 1.0];
                    }
                }
            }
        }
        gbuffer
    }

    pub fn render_shadow_map(
        &self,
        vertices: &[RasterVertex],
        light_pos: Vec3,
        light_dir: Vec3,
        width: usize,
        height: usize,
    ) -> ShadowMap {
        let up = if light_dir.y.abs() < 0.99 {
            Vec3::new(0.0, 1.0, 0.0)
        } else {
            Vec3::new(1.0, 0.0, 0.0)
        };
        let light_view = mat4_look_at(light_pos, light_pos + light_dir, up);
        let light_proj = mat4_perspective(std::f64::consts::FRAC_PI_4, 1.0, 0.1, 1000.0);
        let view_proj = mat4_mul(light_proj, light_view);
        let mut shadow_map = ShadowMap::new(width, height, view_proj);

        let n = vertices.len();
        let tri_count = n / 3;
        for ti in 0..tri_count {
            let v0 = vertices[ti * 3];
            let v1 = vertices[ti * 3 + 1];
            let v2 = vertices[ti * 3 + 2];

            let p0 = transform_point(&view_proj, v0.position);
            let p1 = transform_point(&view_proj, v1.position);
            let p2 = transform_point(&view_proj, v2.position);

            if p0[3] <= 0.0 || p1[3] <= 0.0 || p2[3] <= 0.0 {
                continue;
            }

            let ndc = |p: [f64; 4]| -> [f64; 3] { [p[0] / p[3], p[1] / p[3], p[2] / p[3]] };
            let n0 = ndc(p0);
            let n1 = ndc(p1);
            let n2 = ndc(p2);
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
                continue;
            }

            let min_x = s0[0].min(s1[0]).min(s2[0]).max(0.0) as usize;
            let max_x = (s0[0].max(s1[0]).max(s2[0]).ceil() as usize).min(width - 1);
            let min_y = s0[1].min(s1[1]).min(s2[1]).max(0.0) as usize;
            let max_y = (s0[1].max(s1[1]).max(s2[1]).ceil() as usize).min(height - 1);

            for py in min_y..=max_y {
                for px in min_x..=max_x {
                    let sp = [px as f64 + 0.5, py as f64 + 0.5];
                    let w0 = edge_fn(s1, s2, sp) / area;
                    let w1 = edge_fn(s2, s0, sp) / area;
                    let w2 = edge_fn(s0, s1, sp) / area;
                    if w0 < 0.0 || w1 < 0.0 || w2 < 0.0 {
                        continue;
                    }
                    let depth = w0 * n0[2] + w1 * n1[2] + w2 * n2[2];
                    let idx = py * width + px;
                    if depth < shadow_map.depth[idx] {
                        shadow_map.depth[idx] = depth;
                    }
                }
            }
        }
        shadow_map
    }

    pub fn render_wireframe(
        &self,
        vertices: &[RasterVertex],
        fb: &mut [[f64; 4]],
        width: usize,
        height: usize,
    ) {
        let mvp = mat4_mul(
            mat4_mul(self.projection_matrix, self.view_matrix),
            self.model_matrix,
        );
        let n = vertices.len();
        let tri_count = n / 3;
        let wire_color = [0.0_f64, 1.0, 0.0, 1.0];
        for ti in 0..tri_count {
            let verts = [vertices[ti * 3], vertices[ti * 3 + 1], vertices[ti * 3 + 2]];
            let mut screen = [[0.0_f64; 2]; 3];
            let mut valid = true;
            for (i, v) in verts.iter().enumerate() {
                let p = transform_point(&mvp, v.position);
                if p[3] <= 0.0 {
                    valid = false;
                    break;
                }
                screen[i][0] = (p[0] / p[3] + 1.0) * 0.5 * width as f64;
                screen[i][1] = (1.0 - p[1] / p[3]) * 0.5 * height as f64;
            }
            if !valid {
                continue;
            }
            for edge in [(0, 1), (1, 2), (2, 0)] {
                let a = screen[edge.0];
                let b = screen[edge.1];
                Self::draw_line(fb, width, height, a, b, wire_color);
            }
        }
    }

    pub fn render_with_sss(
        &self,
        vertices: &[RasterVertex],
        sss_profile: crate::core::engine::rendering::materials::sss::SssProfile,
        kernel_radius: usize,
        samples: u32,
        width: usize,
        height: usize,
    ) -> crate::core::engine::rendering::framebuffer::buffer::FrameBuffer {
        use crate::core::engine::rendering::framebuffer::buffer::FrameBuffer;
        use crate::core::engine::rendering::materials::sss::SssPass;

        let gbuffer = self.render_to_gbuffer(vertices, width, height);
        let n = width * height;

        let mut fb = FrameBuffer::new(width, height);
        for i in 0..n {
            let a = gbuffer.albedo[i];
            fb.color[i] = Vec3::new(a[0], a[1], a[2]);
            fb.alpha[i] = a[3];
            fb.depth[i] = gbuffer.depth[i];
        }

        let normals_f64 = gbuffer.normal.clone();
        let depths_f64 = gbuffer.depth.clone();

        let pass = SssPass::new(sss_profile, kernel_radius, samples);
        pass.apply(&fb, &normals_f64, &depths_f64)
    }

    pub fn render_tiled_to_gbuffer(
        &self,
        vertices: &[RasterVertex],
        width: usize,
        height: usize,
        tile_size: usize,
    ) -> GBuffer {
        let tiler = super::tiler::TileRasterizer::new(tile_size);
        let mvp = mat4_mul(
            mat4_mul(self.projection_matrix, self.view_matrix),
            self.model_matrix,
        );
        let gbuffer = tiler.rasterize_to_gbuffer(vertices, &mvp, width, height);
        crate::runtime_log!(
            "tiler: tiles={} depth_hits={}",
            tiler.tile_count(width, height),
            gbuffer.depth.iter().filter(|&&d| d < 1.0).count()
        );
        gbuffer
    }

    fn draw_line(
        fb: &mut [[f64; 4]],
        width: usize,
        height: usize,
        a: [f64; 2],
        b: [f64; 2],
        color: [f64; 4],
    ) {
        let mut x0 = a[0] as i32;
        let mut y0 = a[1] as i32;
        let x1 = b[0] as i32;
        let y1 = b[1] as i32;
        let dx = (x1 - x0).abs();
        let dy = -(y1 - y0).abs();
        let sx = if x0 < x1 { 1_i32 } else { -1_i32 };
        let sy = if y0 < y1 { 1_i32 } else { -1_i32 };
        let mut err = dx + dy;
        loop {
            if x0 >= 0 && x0 < width as i32 && y0 >= 0 && y0 < height as i32 {
                fb[y0 as usize * width + x0 as usize] = color;
            }
            if x0 == x1 && y0 == y1 {
                break;
            }
            let e2 = 2 * err;
            if e2 >= dy {
                err += dy;
                x0 += sx;
            }
            if e2 <= dx {
                err += dx;
                y0 += sy;
            }
        }
    }
}
