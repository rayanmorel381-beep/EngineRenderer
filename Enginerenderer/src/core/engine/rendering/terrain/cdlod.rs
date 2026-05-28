use crate::core::engine::rendering::raytracing::Vec3;

#[derive(Debug, Clone)]
pub struct HeightMap {
    pub data: Vec<f32>,
    pub width: usize,
    pub height: usize,
    pub world_scale: Vec3,
}

impl HeightMap {
    pub fn new(data: Vec<f32>, width: usize, height: usize, world_scale: Vec3) -> Self {
        Self {
            data,
            width,
            height,
            world_scale,
        }
    }

    pub fn flat(width: usize, height: usize, world_scale: Vec3) -> Self {
        Self {
            data: vec![0.0; width * height],
            width,
            height,
            world_scale,
        }
    }

    pub fn sample(&self, x: f64, z: f64) -> f64 {
        let nx =
            (x / self.world_scale.x * (self.width - 1) as f64).clamp(0.0, (self.width - 1) as f64);
        let nz = (z / self.world_scale.z * (self.height - 1) as f64)
            .clamp(0.0, (self.height - 1) as f64);
        let ix = nx as usize;
        let iz = nz as usize;
        let fx = nx - ix as f64;
        let fz = nz - iz as f64;
        let x1 = ix.min(self.width - 1);
        let z1 = iz.min(self.height - 1);
        let x2 = (ix + 1).min(self.width - 1);
        let z2 = (iz + 1).min(self.height - 1);
        let h00 = self.data[z1 * self.width + x1] as f64;
        let h10 = self.data[z1 * self.width + x2] as f64;
        let h01 = self.data[z2 * self.width + x1] as f64;
        let h11 = self.data[z2 * self.width + x2] as f64;
        let h = h00 * (1.0 - fx) * (1.0 - fz)
            + h10 * fx * (1.0 - fz)
            + h01 * (1.0 - fx) * fz
            + h11 * fx * fz;
        h * self.world_scale.y
    }

    pub fn normal_at(&self, x: f64, z: f64) -> Vec3 {
        let eps = self.world_scale.x / self.width as f64;
        let h_left = self.sample(x - eps, z);
        let h_right = self.sample(x + eps, z);
        let h_back = self.sample(x, z - eps);
        let h_front = self.sample(x, z + eps);
        Vec3::new(h_left - h_right, 2.0 * eps, h_back - h_front).normalize()
    }

    pub fn world_position(&self, x: f64, z: f64) -> Vec3 {
        Vec3::new(x, self.sample(x, z), z)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct TerrainPatch {
    pub center: Vec3,
    pub size: f64,
    pub lod_level: u32,
    pub morph_factor: f64,
}

#[derive(Debug)]
pub struct CdlodNode {
    pub bounds_min: Vec3,
    pub bounds_max: Vec3,
    pub lod_level: u32,
    pub children: Option<Box<[CdlodNode; 4]>>,
}

impl CdlodNode {
    pub fn build(heightmap: &HeightMap, min: Vec3, max: Vec3, level: u32, max_level: u32) -> Self {
        let level = level.min(max_level);
        if level == 0 {
            return Self {
                bounds_min: min,
                bounds_max: max,
                lod_level: 0,
                children: None,
            };
        }
        let mid_x = (min.x + max.x) * 0.5;
        let mid_z = (min.z + max.z) * 0.5;
        let mid_y_min = heightmap.sample(mid_x, mid_z) - 1.0;
        let mid_y_max = heightmap.sample(mid_x, mid_z) + 1.0;
        let child_level = level - 1;
        let children = [
            CdlodNode::build(
                heightmap,
                Vec3::new(min.x, mid_y_min, min.z),
                Vec3::new(mid_x, mid_y_max, mid_z),
                child_level,
                max_level,
            ),
            CdlodNode::build(
                heightmap,
                Vec3::new(mid_x, mid_y_min, min.z),
                Vec3::new(max.x, mid_y_max, mid_z),
                child_level,
                max_level,
            ),
            CdlodNode::build(
                heightmap,
                Vec3::new(min.x, mid_y_min, mid_z),
                Vec3::new(mid_x, mid_y_max, max.z),
                child_level,
                max_level,
            ),
            CdlodNode::build(
                heightmap,
                Vec3::new(mid_x, mid_y_min, mid_z),
                max,
                child_level,
                max_level,
            ),
        ];
        Self {
            bounds_min: min,
            bounds_max: max,
            lod_level: level,
            children: Some(Box::new(children)),
        }
    }

    pub fn select_patches(
        &self,
        camera_pos: Vec3,
        lod_ranges: &[f64],
        result: &mut Vec<TerrainPatch>,
    ) {
        let center = (self.bounds_min + self.bounds_max) * 0.5;
        let half_size = (self.bounds_max - self.bounds_min).x * 0.5;
        let dist = (center - camera_pos).length();
        let lod_range = lod_ranges
            .get(self.lod_level as usize)
            .copied()
            .unwrap_or(f64::MAX);
        let parent_range = if self.lod_level + 1 < lod_ranges.len() as u32 {
            lod_ranges[(self.lod_level + 1) as usize]
        } else {
            f64::MAX
        };

        if dist > lod_range {
            return;
        }

        if let Some(ref children) = self.children {
            let child_range = lod_ranges
                .get(self.lod_level.saturating_sub(1) as usize)
                .copied()
                .unwrap_or(0.0);
            if dist < child_range {
                for child in children.iter() {
                    child.select_patches(camera_pos, lod_ranges, result);
                }
                return;
            }
        }

        let morph_start = if parent_range < f64::MAX {
            (lod_range - (parent_range - lod_range) * 0.3).max(0.0)
        } else {
            lod_range * 0.7
        };
        let morph_factor = if dist > morph_start {
            ((dist - morph_start) / (lod_range - morph_start)).clamp(0.0, 1.0)
        } else {
            0.0
        };

        result.push(TerrainPatch {
            center,
            size: half_size * 2.0,
            lod_level: self.lod_level,
            morph_factor,
        });
    }
}

pub struct CdlodTerrain {
    pub heightmap: HeightMap,
    pub root: CdlodNode,
    pub lod_ranges: Vec<f64>,
}

impl CdlodTerrain {
    pub fn new(heightmap: HeightMap, max_lod: u32, base_lod_range: f64) -> Self {
        let w = heightmap.world_scale.x;
        let h_max = heightmap.world_scale.y;
        let d = heightmap.world_scale.z;
        let root = CdlodNode::build(
            &heightmap,
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(w, h_max, d),
            max_lod,
            max_lod,
        );
        let lod_ranges: Vec<f64> = (0..=max_lod as usize)
            .map(|i| base_lod_range * (1 << i) as f64)
            .collect();
        Self {
            heightmap,
            root,
            lod_ranges,
        }
    }

    pub fn select_patches(&self, camera_pos: Vec3) -> Vec<TerrainPatch> {
        let mut result = Vec::new();
        self.root
            .select_patches(camera_pos, &self.lod_ranges, &mut result);
        result
    }

    pub fn height_at(&self, x: f64, z: f64) -> f64 {
        self.heightmap.sample(x, z)
    }

    pub fn normal_at(&self, x: f64, z: f64) -> Vec3 {
        self.heightmap.normal_at(x, z)
    }
}
