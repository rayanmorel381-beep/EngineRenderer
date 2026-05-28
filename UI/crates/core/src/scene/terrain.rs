#[derive(Clone, Debug)]
pub struct TerrainLayer {
    pub name: String,
    pub color: [f64; 4],
    pub tiling: f64,
}

impl TerrainLayer {
    pub fn new(name: impl Into<String>, color: [f64; 4]) -> Self {
        Self { name: name.into(), color, tiling: 4.0 }
    }
}

#[derive(Clone, Debug)]
pub struct TerrainData {
    pub width: usize,
    pub height: usize,
    pub resolution: f64,
    pub height_scale: f64,
    pub heights: Vec<f32>,
    pub blend_weights: Vec<Vec<f32>>,
    pub layers: Vec<TerrainLayer>,
}

impl TerrainData {
    pub fn new(width: usize, height: usize, resolution: f64) -> Self {
        let count = width * height;
        let layers = vec![
            TerrainLayer::new("Grass", [0.22, 0.55, 0.20, 1.0]),
            TerrainLayer::new("Rock",  [0.45, 0.40, 0.35, 1.0]),
            TerrainLayer::new("Snow",  [0.90, 0.92, 0.95, 1.0]),
            TerrainLayer::new("Sand",  [0.85, 0.78, 0.55, 1.0]),
        ];
        let layer_count = layers.len();
        let mut blend_weights = vec![vec![0.0f32; count]; layer_count];
        for i in 0..count { blend_weights[0][i] = 1.0; }
        Self {
            width,
            height,
            resolution,
            height_scale: 20.0,
            heights: vec![0.0f32; count],
            blend_weights,
            layers,
        }
    }

    pub fn sample_height(&self, x: f64, z: f64) -> f64 {
        let gx = (x / self.resolution).clamp(0.0, self.width as f64 - 1.0) as usize;
        let gz = (z / self.resolution).clamp(0.0, self.height as f64 - 1.0) as usize;
        let idx = gz * self.width + gx;
        self.heights.get(idx).copied().unwrap_or(0.0) as f64 * self.height_scale
    }

    pub fn sculpt(&mut self, center_x: f64, center_z: f64, radius: f64, strength: f64, raise: bool) {
        let cx = (center_x / self.resolution) as i64;
        let cz = (center_z / self.resolution) as i64;
        let r = (radius / self.resolution).ceil() as i64;
        for gz in (cz - r)..=(cz + r) {
            for gx in (cx - r)..=(cx + r) {
                if gx < 0 || gz < 0 || gx >= self.width as i64 || gz >= self.height as i64 { continue; }
                let dx = (gx - cx) as f64;
                let dz = (gz - cz) as f64;
                let dist = (dx*dx + dz*dz).sqrt();
                let falloff = (1.0 - (dist / r as f64).clamp(0.0, 1.0)).powi(2);
                let delta = strength * falloff * 0.016;
                let idx = gz as usize * self.width + gx as usize;
                if let Some(h) = self.heights.get_mut(idx) {
                    if raise { *h = (*h + delta as f32).min(1.0); }
                    else { *h = (*h - delta as f32).max(0.0); }
                }
            }
        }
    }

    pub fn smooth(&mut self, center_x: f64, center_z: f64, radius: f64, strength: f64) {
        let cx = (center_x / self.resolution) as i64;
        let cz = (center_z / self.resolution) as i64;
        let r = (radius / self.resolution).ceil() as i64;
        let snapshot = self.heights.clone();
        for gz in (cz - r)..=(cz + r) {
            for gx in (cx - r)..=(cx + r) {
                if gx < 0 || gz < 0 || gx >= self.width as i64 || gz >= self.height as i64 { continue; }
                let dx = (gx - cx) as f64;
                let dz = (gz - cz) as f64;
                let dist = (dx*dx + dz*dz).sqrt();
                let falloff = (1.0 - (dist / r as f64).clamp(0.0, 1.0)).powi(2) * strength;
                let mut sum = 0.0f64;
                let mut cnt = 0;
                for nz in (gz - 1)..=(gz + 1) {
                    for nx in (gx - 1)..=(gx + 1) {
                        if nx >= 0 && nz >= 0 && nx < self.width as i64 && nz < self.height as i64 {
                            sum += snapshot[nz as usize * self.width + nx as usize] as f64;
                            cnt += 1;
                        }
                    }
                }
                let avg = sum / cnt as f64;
                let idx = gz as usize * self.width + gx as usize;
                if let Some(h) = self.heights.get_mut(idx) {
                    *h = (*h as f64 * (1.0 - falloff) + avg * falloff) as f32;
                }
            }
        }
    }

    pub fn paint_layer(&mut self, center_x: f64, center_z: f64, radius: f64, strength: f64, layer: usize) {
        if layer >= self.layers.len() { return; }
        let cx = (center_x / self.resolution) as i64;
        let cz = (center_z / self.resolution) as i64;
        let r = (radius / self.resolution).ceil() as i64;
        for gz in (cz - r)..=(cz + r) {
            for gx in (cx - r)..=(cx + r) {
                if gx < 0 || gz < 0 || gx >= self.width as i64 || gz >= self.height as i64 { continue; }
                let dx = (gx - cx) as f64;
                let dz = (gz - cz) as f64;
                let dist = (dx*dx + dz*dz).sqrt();
                let falloff = (1.0 - (dist / r as f64).clamp(0.0, 1.0)).powi(2) * strength * 0.016;
                let idx = gz as usize * self.width + gx as usize;
                for (li, lw) in self.blend_weights.iter_mut().enumerate() {
                    if let Some(w) = lw.get_mut(idx) {
                        if li == layer { *w = (*w + falloff as f32).min(1.0); }
                        else { *w = (*w * (1.0 - falloff as f32)).max(0.0); }
                    }
                }
            }
        }
    }
}
