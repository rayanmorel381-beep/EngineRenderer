use crate::core::engine::rendering::raytracing::Vec3;
use crate::core::engine::rendering::terrain::cdlod::HeightMap;

#[derive(Debug, Clone, Copy)]
pub struct FoliageInstance {
    pub position: Vec3,
    pub scale: f64,
    pub rotation_y: f64,
    pub lod: u32,
    pub wind_phase: f64,
}

#[derive(Debug, Clone)]
pub struct FoliageLayer {
    pub density_map: Vec<f64>,
    pub map_width: usize,
    pub map_height: usize,
    pub wind_strength: f64,
    pub wind_frequency: f64,
    pub wind_direction: Vec3,
    pub scale_min: f64,
    pub scale_max: f64,
    pub max_slope: f64,
    pub lod_distances: Vec<f64>,
}

impl FoliageLayer {
    pub fn new(map_width: usize, map_height: usize) -> Self {
        Self {
            density_map: vec![1.0; map_width * map_height],
            map_width,
            map_height,
            wind_strength: 0.15,
            wind_frequency: 1.2,
            wind_direction: Vec3::new(1.0, 0.0, 0.3).normalize(),
            scale_min: 0.8,
            scale_max: 1.2,
            max_slope: 0.7,
            lod_distances: vec![20.0, 50.0, 100.0],
        }
    }

    pub fn density_at(&self, u: f64, v: f64) -> f64 {
        let px = (u * (self.map_width - 1) as f64).clamp(0.0, (self.map_width - 1) as f64) as usize;
        let pz =
            (v * (self.map_height - 1) as f64).clamp(0.0, (self.map_height - 1) as f64) as usize;
        self.density_map[pz * self.map_width + px]
    }
}

pub struct FoliageSystem;

impl FoliageSystem {
    pub fn scatter(
        terrain: &HeightMap,
        layer: &FoliageLayer,
        grid_spacing: f64,
        seed: u32,
    ) -> Vec<FoliageInstance> {
        let world_w = terrain.world_scale.x;
        let world_d = terrain.world_scale.z;
        let cols = (world_w / grid_spacing) as usize;
        let rows = (world_d / grid_spacing) as usize;
        let mut instances = Vec::with_capacity(cols * rows);
        let mut rng = seed;

        for row in 0..rows {
            for col in 0..cols {
                rng = rng.wrapping_mul(0x9E37_79B9).wrapping_add(0x6C62_272E);
                let jitter_x = lcg_f64(&mut rng) - 0.5;
                let jitter_z = lcg_f64(&mut rng) - 0.5;
                let world_x = (col as f64 + jitter_x) * grid_spacing;
                let world_z = (row as f64 + jitter_z) * grid_spacing;

                if world_x < 0.0 || world_x >= world_w || world_z < 0.0 || world_z >= world_d {
                    continue;
                }

                let u = world_x / world_w;
                let v = world_z / world_d;
                let density = layer.density_at(u, v);
                if lcg_f64(&mut rng) > density {
                    continue;
                }

                let normal = terrain.normal_at(world_x, world_z);
                if normal.y < layer.max_slope {
                    continue;
                }

                let world_y = terrain.sample(world_x, world_z);
                let scale =
                    layer.scale_min + (layer.scale_max - layer.scale_min) * lcg_f64(&mut rng);
                let rotation_y = lcg_f64(&mut rng) * std::f64::consts::TAU;
                let wind_phase = lcg_f64(&mut rng) * std::f64::consts::TAU;

                instances.push(FoliageInstance {
                    position: Vec3::new(world_x, world_y, world_z),
                    scale,
                    rotation_y,
                    lod: 0,
                    wind_phase,
                });
            }
        }
        instances
    }

    pub fn update_lod(instances: &mut [FoliageInstance], camera_pos: Vec3, lod_distances: &[f64]) {
        for inst in instances.iter_mut() {
            let dist = (inst.position - camera_pos).length();
            inst.lod = lod_distances
                .iter()
                .position(|&d| dist < d)
                .map(|i| i as u32)
                .unwrap_or(lod_distances.len() as u32);
        }
    }

    pub fn animate_wind(
        instances: &[FoliageInstance],
        layer: &FoliageLayer,
        time: f64,
    ) -> Vec<Vec3> {
        instances
            .iter()
            .map(|inst| {
                let phase = inst.wind_phase + time * layer.wind_frequency;
                let sin_val = (phase).sin();
                let gust = (phase * 0.37 + 1.0).sin() * 0.3;
                let amplitude = layer.wind_strength * (1.0 + gust) * inst.scale;
                layer.wind_direction * (sin_val * amplitude)
            })
            .collect()
    }
}

fn lcg_f64(seed: &mut u32) -> f64 {
    *seed = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
    (*seed >> 8) as f64 / 16_777_216.0
}
