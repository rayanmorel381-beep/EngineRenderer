fn vec3_sub(a: [f64; 3], b: [f64; 3]) -> [f64; 3] { [a[0]-b[0], a[1]-b[1], a[2]-b[2]] }
fn vec3_add(a: [f64; 3], b: [f64; 3]) -> [f64; 3] { [a[0]+b[0], a[1]+b[1], a[2]+b[2]] }
fn vec3_scale(v: [f64; 3], s: f64) -> [f64; 3] { [v[0]*s, v[1]*s, v[2]*s] }
fn vec3_len(v: [f64; 3]) -> f64 { (v[0]*v[0]+v[1]*v[1]+v[2]*v[2]).sqrt() }

fn hash_u64(mut x: u64) -> f64 {
    x ^= x >> 33;
    x = x.wrapping_mul(0xff51afd7ed558ccd);
    x ^= x >> 33;
    (x as f64) / (u64::MAX as f64)
}

#[derive(Clone, Debug, PartialEq)]
pub enum FractureMode {
    Voronoi,
    Radial,
    PlanarSlice,
}

impl FractureMode {
    pub fn label(&self) -> &'static str {
        match self { Self::Voronoi => "Voronoi", Self::Radial => "Radial", Self::PlanarSlice => "Tranche planaire" }
    }
    pub const ALL: [FractureMode; 3] = [FractureMode::Voronoi, FractureMode::Radial, FractureMode::PlanarSlice];
}

#[derive(Clone, Debug)]
pub struct DestructionChunk {
    pub id: u32,
    pub position: [f64; 3],
    pub velocity: [f64; 3],
    pub angular_velocity: [f64; 3],
    pub mass: f64,
    pub health: f64,
    pub active: bool,
    pub parent: Option<u32>,
    pub size: [f64; 3],
}

impl DestructionChunk {
    pub fn new(id: u32, position: [f64; 3], size: [f64; 3], mass: f64) -> Self {
        Self {
            id, position, velocity: [0.0; 3], angular_velocity: [0.0; 3],
            mass, health: 100.0, active: false, parent: None, size,
        }
    }

    pub fn apply_impulse(&mut self, impulse: [f64; 3]) {
        let inv_mass = 1.0 / self.mass.max(1e-6);
        self.velocity = vec3_add(self.velocity, vec3_scale(impulse, inv_mass));
    }

    pub fn tick(&mut self, dt: f64) {
        if !self.active { return; }
        self.velocity[1] -= 9.81 * dt;
        let drag = 0.02;
        self.velocity = vec3_scale(self.velocity, (1.0_f64 - drag).max(0.0_f64));
        self.position = vec3_add(self.position, vec3_scale(self.velocity, dt));
        if self.position[1] < 0.0 {
            self.position[1] = 0.0;
            self.velocity[1] = -self.velocity[1] * 0.3;
        }
    }
}

#[derive(Clone, Debug)]
pub struct DestructionBody {
    pub name: String,
    pub mode: FractureMode,
    pub chunk_count: usize,
    pub impact_threshold: f64,
    pub health: f64,
    pub chunks: Vec<DestructionChunk>,
    pub fractured: bool,
    pub debris_lifetime: f64,
    pub enabled: bool,
    next_id: u32,
}

impl Default for DestructionBody {
    fn default() -> Self {
        Self {
            name: "Destructible".to_string(),
            mode: FractureMode::Voronoi,
            chunk_count: 8,
            impact_threshold: 50.0,
            health: 100.0,
            chunks: Vec::new(),
            fractured: false,
            debris_lifetime: 10.0,
            enabled: true,
            next_id: 0,
        }
    }
}

impl DestructionBody {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into(), ..Self::default() }
    }

    pub fn fracture(&mut self, origin: [f64; 3], impulse: [f64; 3], seed: u64) {
        if self.fractured { return; }
        self.fractured = true;
        self.chunks.clear();
        match self.mode {
            FractureMode::Voronoi => self.fracture_voronoi(origin, impulse, seed),
            FractureMode::Radial => self.fracture_radial(origin, impulse, seed),
            FractureMode::PlanarSlice => self.fracture_planar(origin, impulse, seed),
        }
    }

    fn fracture_voronoi(&mut self, origin: [f64; 3], impulse: [f64; 3], seed: u64) {
        let n = self.chunk_count;
        for i in 0..n {
            let hx = hash_u64(seed.wrapping_add(i as u64 * 3));
            let hy = hash_u64(seed.wrapping_add(i as u64 * 3 + 1));
            let hz = hash_u64(seed.wrapping_add(i as u64 * 3 + 2));
            let pos = [origin[0] + (hx - 0.5) * 2.0, origin[1] + hy * 1.5, origin[2] + (hz - 0.5) * 2.0];
            let mass = 1.0 / n as f64;
            let size = [0.3 + hx * 0.3, 0.3 + hy * 0.3, 0.3 + hz * 0.3];
            let mut chunk = DestructionChunk::new(self.next_id, pos, size, mass);
            chunk.active = true;
            let dir = vec3_sub(pos, origin);
            let dist = vec3_len(dir).max(1e-6);
            let n_dir = vec3_scale(dir, 1.0 / dist);
            let imp_len = vec3_len(impulse);
            chunk.apply_impulse(vec3_scale(n_dir, imp_len * (1.0 + hx)));
            self.chunks.push(chunk);
            self.next_id += 1;
        }
    }

    fn fracture_radial(&mut self, origin: [f64; 3], impulse: [f64; 3], seed: u64) {
        let n = self.chunk_count;
        let imp_len = vec3_len(impulse);
        for i in 0..n {
            let angle = std::f64::consts::TAU * i as f64 / n as f64;
            let hr = hash_u64(seed.wrapping_add(i as u64));
            let r = 0.5 + hr * 1.0;
            let pos = [origin[0] + angle.cos() * r, origin[1] + hr * 0.5, origin[2] + angle.sin() * r];
            let mass = 1.0 / n as f64;
            let size = [0.25, 0.25, 0.25];
            let mut chunk = DestructionChunk::new(self.next_id, pos, size, mass);
            chunk.active = true;
            let dir = [angle.cos(), 0.3, angle.sin()];
            chunk.apply_impulse(vec3_scale(dir, imp_len));
            self.chunks.push(chunk);
            self.next_id += 1;
        }
    }

    fn fracture_planar(&mut self, origin: [f64; 3], impulse: [f64; 3], seed: u64) {
        let n = self.chunk_count.min(4);
        let imp_len = vec3_len(impulse);
        for i in 0..n {
            let layer = i as f64 - n as f64 * 0.5;
            let h = hash_u64(seed.wrapping_add(i as u64));
            let pos = [origin[0] + (h - 0.5), origin[1] + layer * 0.4, origin[2]];
            let mass = 1.0 / n as f64;
            let size = [1.5, 0.3, 1.5];
            let mut chunk = DestructionChunk::new(self.next_id, pos, size, mass);
            chunk.active = true;
            chunk.apply_impulse([0.0, imp_len * 0.5, 0.0]);
            self.chunks.push(chunk);
            self.next_id += 1;
        }
    }

    pub fn tick(&mut self, dt: f64) {
        if !self.fractured { return; }
        for chunk in &mut self.chunks { chunk.tick(dt); }
    }

    pub fn apply_damage(&mut self, damage: f64, impact_pos: [f64; 3], impulse: [f64; 3]) {
        self.health -= damage;
        if self.health <= 0.0 {
            self.fracture(impact_pos, impulse, 42);
        }
    }
}
