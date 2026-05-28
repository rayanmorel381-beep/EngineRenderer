fn hash_f64(mut x: u64) -> f64 {
    x = x.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
    x ^= x >> 33;
    x = x.wrapping_mul(0xff51afd7ed558ccd);
    x ^= x >> 33;
    (x as f64) / (u64::MAX as f64)
}

#[derive(Clone, Debug, PartialEq)]
pub enum FoliagePaintMode {
    Add,
    Remove,
    Paint,
}

impl FoliagePaintMode {
    pub fn label(&self) -> &'static str {
        match self { Self::Add => "Ajouter", Self::Remove => "Supprimer", Self::Paint => "Peindre" }
    }
    pub const ALL: [FoliagePaintMode; 3] = [FoliagePaintMode::Add, FoliagePaintMode::Remove, FoliagePaintMode::Paint];
}

#[derive(Clone, Debug)]
pub struct FoliageInstance {
    pub position: [f64; 3],
    pub rotation: [f64; 3],
    pub scale: [f64; 3],
    pub color_variation: f64,
}

impl FoliageInstance {
    pub fn new(position: [f64; 3]) -> Self {
        Self { position, rotation: [0.0, 0.0, 0.0], scale: [1.0, 1.0, 1.0], color_variation: 0.0 }
    }
}

#[derive(Clone, Debug)]
pub struct FoliageType {
    pub name: String,
    pub mesh_asset: Option<String>,
    pub density: f64,
    pub min_scale: f64,
    pub max_scale: f64,
    pub align_to_normal: bool,
    pub random_yaw: bool,
    pub cast_shadow: bool,
    pub cull_distance: f64,
    pub instances: Vec<FoliageInstance>,
    pub enabled: bool,
}

impl Default for FoliageType {
    fn default() -> Self {
        Self {
            name: "Herbe".to_string(),
            mesh_asset: None,
            density: 1.0,
            min_scale: 0.8,
            max_scale: 1.2,
            align_to_normal: true,
            random_yaw: true,
            cast_shadow: false,
            cull_distance: 100.0,
            instances: Vec::new(),
            enabled: true,
        }
    }
}

impl FoliageType {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into(), ..Self::default() }
    }

    pub fn scatter(&mut self, origin: [f64; 3], radius: f64, count: usize, seed: u64) {
        for i in 0..count {
            let hx = hash_f64(seed.wrapping_add(i as u64 * 2));
            let hz = hash_f64(seed.wrapping_add(i as u64 * 2 + 1));
            let angle = hx * std::f64::consts::TAU;
            let r = hz.sqrt() * radius;
            let pos = [origin[0] + angle.cos() * r, origin[1], origin[2] + angle.sin() * r];
            let hy = hash_f64(seed.wrapping_add(i as u64 * 3 + 2));
            let scale_v = self.min_scale + (self.max_scale - self.min_scale) * hy;
            let hr = hash_f64(seed.wrapping_add(i as u64 * 4 + 3));
            let rot_y = if self.random_yaw { hr * 360.0 } else { 0.0 };
            let hc = hash_f64(seed.wrapping_add(i as u64 * 5 + 4));
            let mut inst = FoliageInstance::new(pos);
            inst.scale = [scale_v, scale_v, scale_v];
            inst.rotation = [0.0, rot_y, 0.0];
            inst.color_variation = hc;
            self.instances.push(inst);
        }
    }
}

#[derive(Clone, Debug)]
pub struct FoliagePainter {
    pub foliage_types: Vec<FoliageType>,
    pub brush_radius: f64,
    pub brush_density: f64,
    pub brush_strength: f64,
    pub active_type: usize,
    pub mode: FoliagePaintMode,
    pub enabled: bool,
}

impl Default for FoliagePainter {
    fn default() -> Self {
        let mut p = Self {
            foliage_types: Vec::new(),
            brush_radius: 5.0,
            brush_density: 1.0,
            brush_strength: 1.0,
            active_type: 0,
            mode: FoliagePaintMode::Add,
            enabled: true,
        };
        p.foliage_types.push(FoliageType::new("Herbe"));
        p.foliage_types.push(FoliageType::new("Fleurs"));
        let mut tree = FoliageType::new("Arbres");
        tree.density = 0.1;
        tree.min_scale = 0.9;
        tree.max_scale = 1.5;
        tree.cast_shadow = true;
        tree.cull_distance = 500.0;
        p.foliage_types.push(tree);
        p
    }
}

impl FoliagePainter {
    pub fn new() -> Self { Self::default() }

    pub fn scatter_at(&mut self, pos: [f64; 3], seed: u64) {
        let idx = self.active_type.min(self.foliage_types.len().saturating_sub(1));
        if self.foliage_types.is_empty() { return; }
        let radius = self.brush_radius;
        let count = (self.brush_density * 10.0 * self.brush_strength) as usize;
        self.foliage_types[idx].scatter(pos, radius, count, seed);
    }

    pub fn erase_at(&mut self, pos: [f64; 3]) {
        let idx = self.active_type.min(self.foliage_types.len().saturating_sub(1));
        if self.foliage_types.is_empty() { return; }
        let r2 = self.brush_radius * self.brush_radius;
        self.foliage_types[idx].instances.retain(|inst| {
            let dx = inst.position[0] - pos[0];
            let dz = inst.position[2] - pos[2];
            dx * dx + dz * dz > r2
        });
    }

    pub fn total_instances(&self) -> usize {
        self.foliage_types.iter().map(|t| t.instances.len()).sum()
    }
}
