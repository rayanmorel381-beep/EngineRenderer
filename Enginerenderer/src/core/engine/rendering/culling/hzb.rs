use crate::core::engine::math::{Mat4, Vec3, Vec4};

pub struct HierarchicalZBuffer {
    pub mips: Vec<Vec<f32>>,
    pub width: usize,
    pub height: usize,
    pub levels: usize,
}

impl HierarchicalZBuffer {
    pub fn build(depth_buffer: &[f32], width: usize, height: usize) -> Self {
        let levels = (width.max(height) as f32).log2().ceil() as usize + 1;
        let mut mips: Vec<Vec<f32>> = Vec::with_capacity(levels);
        mips.push(depth_buffer.to_vec());

        let mut cur_w = width;
        let mut cur_h = height;
        for _ in 1..levels {
            let next_w = (cur_w / 2).max(1);
            let next_h = (cur_h / 2).max(1);
            let prev = &mips[mips.len() - 1];
            let mut next = vec![f32::MAX; next_w * next_h];
            for y in 0..next_h {
                for x in 0..next_w {
                    let sx = x * 2;
                    let sy = y * 2;
                    let mut max_d = 0.0_f32;
                    for dy in 0..2 {
                        for dx in 0..2 {
                            let px = (sx + dx).min(cur_w - 1);
                            let py = (sy + dy).min(cur_h - 1);
                            let d = prev[py * cur_w + px];
                            if d > max_d { max_d = d; }
                        }
                    }
                    next[y * next_w + x] = max_d;
                }
            }
            mips.push(next);
            cur_w = next_w;
            cur_h = next_h;
        }

        Self { mips, width, height, levels }
    }

    pub fn is_occluded(&self, aabb_min: Vec3, aabb_max: Vec3, mvp: &Mat4) -> bool {
        let corners = [
            Vec3::new(aabb_min.x, aabb_min.y, aabb_min.z),
            Vec3::new(aabb_max.x, aabb_min.y, aabb_min.z),
            Vec3::new(aabb_min.x, aabb_max.y, aabb_min.z),
            Vec3::new(aabb_max.x, aabb_max.y, aabb_min.z),
            Vec3::new(aabb_min.x, aabb_min.y, aabb_max.z),
            Vec3::new(aabb_max.x, aabb_min.y, aabb_max.z),
            Vec3::new(aabb_min.x, aabb_max.y, aabb_max.z),
            Vec3::new(aabb_max.x, aabb_max.y, aabb_max.z),
        ];

        let mut screen_min_x = f32::MAX;
        let mut screen_min_y = f32::MAX;
        let mut screen_max_x = f32::MIN;
        let mut screen_max_y = f32::MIN;
        let mut closest_depth = f32::MAX;

        for c in &corners {
            let clip = mat4_transform(mvp, *c);
            if clip.w <= 0.0 { return false; }
            let nx = clip.x / clip.w;
            let ny = clip.y / clip.w;
            let nz = clip.z / clip.w;
            if nz < 0.0 { return false; }
            let sx = (nx + 1.0) * 0.5 * self.width as f32;
            let sy = (1.0 - ny) * 0.5 * self.height as f32;
            screen_min_x = screen_min_x.min(sx);
            screen_min_y = screen_min_y.min(sy);
            screen_max_x = screen_max_x.max(sx);
            screen_max_y = screen_max_y.max(sy);
            if nz < closest_depth { closest_depth = nz; }
        }

        if screen_max_x < 0.0 || screen_max_y < 0.0
            || screen_min_x >= self.width as f32 || screen_min_y >= self.height as f32
        {
            return true;
        }

        let screen_w = (screen_max_x - screen_min_x).max(1.0);
        let screen_h = (screen_max_y - screen_min_y).max(1.0);
        let level = (screen_w.max(screen_h).log2().floor() as usize).min(self.levels - 1);

        let scale = 1.0 / (1 << level) as f32;
        let lx0 = ((screen_min_x * scale).floor() as usize).min((self.width >> level).saturating_sub(1));
        let ly0 = ((screen_min_y * scale).floor() as usize).min((self.height >> level).saturating_sub(1));
        let lx1 = ((screen_max_x * scale).ceil() as usize).min((self.width >> level).saturating_sub(1));
        let ly1 = ((screen_max_y * scale).ceil() as usize).min((self.height >> level).saturating_sub(1));

        let mip_w = (self.width >> level).max(1);
        let mip = &self.mips[level];

        let mut max_hzb = 0.0_f32;
        for y in ly0..=ly1 {
            for x in lx0..=lx1 {
                let idx = y * mip_w + x;
                if idx < mip.len() && mip[idx] > max_hzb {
                    max_hzb = mip[idx];
                }
            }
        }

        closest_depth > max_hzb
    }

    pub fn sample_depth(&self, x: usize, y: usize, level: usize) -> f32 {
        let l = level.min(self.levels - 1);
        let mip_w = (self.width >> l).max(1);
        let mip_h = (self.height >> l).max(1);
        let lx = x.min(mip_w - 1);
        let ly = y.min(mip_h - 1);
        self.mips[l][ly * mip_w + lx]
    }
}

fn mat4_transform(m: &Mat4, p: Vec3) -> Vec4 {
    Vec4::new(
        m.m[0][0]*p.x + m.m[0][1]*p.y + m.m[0][2]*p.z + m.m[0][3],
        m.m[1][0]*p.x + m.m[1][1]*p.y + m.m[1][2]*p.z + m.m[1][3],
        m.m[2][0]*p.x + m.m[2][1]*p.y + m.m[2][2]*p.z + m.m[2][3],
        m.m[3][0]*p.x + m.m[3][1]*p.y + m.m[3][2]*p.z + m.m[3][3],
    )
}
