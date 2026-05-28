use crate::core::engine::rendering::framebuffer::FrameBuffer;
use crate::core::engine::rendering::raytracing::Vec3;

#[derive(Debug, Clone, Copy)]
pub struct Decal {
    pub position: Vec3,
    pub normal: Vec3,
    pub tangent: Vec3,
    pub size: Vec3,
    pub albedo: Vec3,
    pub roughness: f64,
    pub metallic: f64,
    pub opacity: f64,
    pub normal_strength: f64,
}

impl Decal {
    pub fn new(position: Vec3, normal: Vec3, size: Vec3, albedo: Vec3) -> Self {
        let up = if normal.y.abs() < 0.99 {
            Vec3::new(0.0, 1.0, 0.0)
        } else {
            Vec3::new(1.0, 0.0, 0.0)
        };
        let tangent = normal.cross(up).normalize();
        Self {
            position,
            normal,
            tangent,
            size,
            albedo,
            roughness: 0.7,
            metallic: 0.0,
            opacity: 1.0,
            normal_strength: 1.0,
        }
    }

    fn bitangent(&self) -> Vec3 {
        self.tangent.cross(self.normal).normalize()
    }

    pub fn project(&self, world_pos: Vec3) -> Option<(f64, f64, f64)> {
        let local = world_pos - self.position;
        let u = local.dot(self.tangent) / self.size.x + 0.5;
        let v = local.dot(self.bitangent()) / self.size.z + 0.5;
        let depth = local.dot(self.normal).abs() / self.size.y;
        if !(0.0..=1.0).contains(&u) || !(0.0..=1.0).contains(&v) || depth > 1.0 {
            None
        } else {
            Some((u, v, depth))
        }
    }
}

pub struct DecalPass;

impl DecalPass {
    pub fn project_pixels(
        decal: &Decal,
        fb: &FrameBuffer,
        world_pos_fb: &[Vec3],
    ) -> Vec<(usize, f64, f64, f64)> {
        let mut hits = Vec::new();
        for (idx, &world_pos) in world_pos_fb.iter().enumerate() {
            if idx >= fb.width * fb.height {
                break;
            }
            if let Some((u, v, depth)) = decal.project(world_pos) {
                hits.push((idx, u, v, depth));
            }
        }
        hits
    }

    pub fn apply_all(decals: &[Decal], fb: &mut FrameBuffer, world_pos_fb: &[Vec3]) {
        for decal in decals {
            let hits = Self::project_pixels(decal, fb, world_pos_fb);
            for (idx, u, v, depth) in hits {
                let edge_fade = smoothstep(0.0, 0.15, u.min(1.0 - u))
                    * smoothstep(0.0, 0.15, v.min(1.0 - v))
                    * smoothstep(0.0, 0.2, 1.0 - depth);
                let alpha = decal.opacity * decal.normal_strength * edge_fade;
                if alpha < 1e-4 {
                    continue;
                }
                let roughness_mod = 1.0 - decal.roughness * 0.1;
                let metalness_tint = decal.metallic * 0.2;
                let blend_color = decal.albedo * roughness_mod * (1.0 - metalness_tint)
                    + fb.color[idx] * metalness_tint;
                fb.color[idx] = fb.color[idx] * (1.0 - alpha) + blend_color * alpha;
            }
        }
    }
}

fn smoothstep(edge0: f64, edge1: f64, x: f64) -> f64 {
    let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}
