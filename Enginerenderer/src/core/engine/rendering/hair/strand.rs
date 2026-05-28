use crate::core::engine::rendering::raytracing::Vec3;
use crate::core::engine::rendering::raytracing::hair_bsdf::HairMaterial;

#[derive(Debug, Clone)]
pub struct HairStrand {
    pub control_points: Vec<Vec3>,
    pub width_root: f64,
    pub width_tip: f64,
    pub material: HairMaterial,
}

impl HairStrand {
    pub fn new(control_points: Vec<Vec3>, width_root: f64, width_tip: f64) -> Self {
        Self {
            control_points,
            width_root,
            width_tip,
            material: HairMaterial::default(),
        }
    }

    pub fn segment_count(&self) -> usize {
        self.control_points.len().saturating_sub(1)
    }

    pub fn width_at(&self, t: f64) -> f64 {
        self.width_root + (self.width_tip - self.width_root) * t
    }

    pub fn tangent_at(&self, segment: usize) -> Vec3 {
        let a = self.control_points[segment];
        let b = self.control_points[segment + 1];
        let d = b - a;
        let len = d.length();
        if len > f64::EPSILON {
            d * (1.0 / len)
        } else {
            Vec3::new(0.0, 1.0, 0.0)
        }
    }

    pub fn catmull_rom(&self, t: f64) -> Vec3 {
        let n = self.control_points.len();
        if n < 2 {
            return self.control_points[0];
        }
        let seg_f = t * (n - 1) as f64;
        let i = (seg_f as usize).min(n - 2);
        let lt = seg_f - i as f64;
        let p0 = if i > 0 {
            self.control_points[i - 1]
        } else {
            self.control_points[0]
        };
        let p1 = self.control_points[i];
        let p2 = self.control_points[i + 1];
        let p3 = if i + 2 < n {
            self.control_points[i + 2]
        } else {
            self.control_points[n - 1]
        };
        let t2 = lt * lt;
        let t3 = t2 * lt;
        (p0 * (-0.5 * t3 + t2 - 0.5 * lt))
            + (p1 * (1.5 * t3 - 2.5 * t2 + 1.0))
            + (p2 * (-1.5 * t3 + 2.0 * t2 + 0.5 * lt))
            + (p3 * (0.5 * t3 - 0.5 * t2))
    }
}

#[derive(Debug, Clone)]
pub struct HairGroom {
    pub strands: Vec<HairStrand>,
    pub root_offset: Vec3,
}

impl HairGroom {
    pub fn new(strands: Vec<HairStrand>, root_offset: Vec3) -> Self {
        Self {
            strands,
            root_offset,
        }
    }

    pub fn strand_count(&self) -> usize {
        self.strands.len()
    }

    pub fn total_segments(&self) -> usize {
        self.strands.iter().map(|s| s.segment_count()).sum()
    }

    pub fn apply_gravity(&mut self, gravity: Vec3, stiffness: f64, dt: f64) {
        for strand in &mut self.strands {
            let n = strand.control_points.len();
            for i in 1..n {
                let t = i as f64 / (n - 1) as f64;
                strand.control_points[i] += gravity * (1.0 - stiffness) * t * dt;
            }
        }
    }

    pub fn rasterize_to_buffer(
        &self,
        fb: &mut [[f32; 4]],
        width: usize,
        height: usize,
        view_proj: &[[f32; 4]; 4],
    ) {
        for strand in &self.strands {
            let steps = strand.segment_count() * 4;
            if steps == 0 {
                continue;
            }
            for step in 0..steps {
                let t = step as f64 / steps as f64;
                let pos = strand.catmull_rom(t);
                let clip =
                    transform_vec(view_proj, [pos.x as f32, pos.y as f32, pos.z as f32, 1.0]);
                if clip[3] <= 0.0 {
                    continue;
                }
                let nx = clip[0] / clip[3];
                let ny = clip[1] / clip[3];
                let sx = ((nx + 1.0) * 0.5 * width as f32) as usize;
                let sy = ((1.0 - ny) * 0.5 * height as f32) as usize;
                let w = (strand.width_at(t) * 0.5) as usize + 1;
                let mat = strand.material;
                let color = [
                    mat.melanin as f32 * 0.5,
                    mat.melanin as f32 * 0.3,
                    (1.0 - mat.melanin_redness) as f32 * 0.1,
                    1.0,
                ];
                for dy in 0..w {
                    for dx in 0..w {
                        let px = sx.wrapping_add(dx).wrapping_sub(w / 2);
                        let py = sy.wrapping_add(dy).wrapping_sub(w / 2);
                        if px < width && py < height {
                            fb[py * width + px] = color;
                        }
                    }
                }
            }
        }
    }
}

fn transform_vec(m: &[[f32; 4]; 4], v: [f32; 4]) -> [f32; 4] {
    [
        m[0][0] * v[0] + m[0][1] * v[1] + m[0][2] * v[2] + m[0][3] * v[3],
        m[1][0] * v[0] + m[1][1] * v[1] + m[1][2] * v[2] + m[1][3] * v[3],
        m[2][0] * v[0] + m[2][1] * v[1] + m[2][2] * v[2] + m[2][3] * v[3],
        m[3][0] * v[0] + m[3][1] * v[1] + m[3][2] * v[2] + m[3][3] * v[3],
    ]
}
