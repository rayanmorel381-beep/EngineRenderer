fn hash_u64(seed: u64, idx: u64) -> f64 {
    let mut x = seed.wrapping_mul(6364136223846793005).wrapping_add(idx.wrapping_mul(1442695040888963407));
    x ^= x >> 33;
    x = x.wrapping_mul(0xff51afd7ed558ccd);
    x ^= x >> 33;
    (x as f64) / (u64::MAX as f64)
}

#[derive(Clone, Debug)]
pub struct GerstnerWave {
    pub amplitude: f64,
    pub wavelength: f64,
    pub speed: f64,
    pub direction: [f64; 2],
    pub steepness: f64,
    pub enabled: bool,
}

impl Default for GerstnerWave {
    fn default() -> Self {
        Self { amplitude: 0.5, wavelength: 8.0, speed: 1.0, direction: [1.0, 0.0], steepness: 0.5, enabled: true }
    }
}

impl GerstnerWave {
    pub fn displacement(&self, pos: [f64; 2], time: f64) -> [f64; 3] {
        let k = std::f64::consts::TAU / self.wavelength;
        let phase_bias = hash_u64((self.wavelength * 1000.0) as u64, (self.speed * 1000.0) as u64);
        let phase = k * (self.direction[0] * pos[0] + self.direction[1] * pos[1]) - self.speed * time + phase_bias;
        let q = self.steepness / (k * self.amplitude).max(1e-6);
        [
            q * self.amplitude * self.direction[0] * phase.cos(),
            self.amplitude * phase.sin(),
            q * self.amplitude * self.direction[1] * phase.cos(),
        ]
    }
}

#[derive(Clone, Debug)]
pub struct FloatingBody {
    pub position: [f64; 3],
    pub velocity: [f64; 3],
    pub mass: f64,
    pub volume: f64,
    pub drag: f64,
    pub angular_drag: f64,
    pub submerged_fraction: f64,
}

impl Default for FloatingBody {
    fn default() -> Self {
        Self { position: [0.0, 0.5, 0.0], velocity: [0.0; 3], mass: 1.0, volume: 0.5, drag: 0.5, angular_drag: 0.2, submerged_fraction: 0.0 }
    }
}

#[derive(Clone, Debug)]
pub struct WaterBody {
    pub name: String,
    pub waves: Vec<GerstnerWave>,
    pub water_level: f64,
    pub foam_threshold: f64,
    pub depth: f64,
    pub density: f64,
    pub surface_tension: f64,
    pub caustic_intensity: f64,
    pub color: [f64; 4],
    pub absorption: [f64; 3],
    pub enabled: bool,
}

impl Default for WaterBody {
    fn default() -> Self {
        let mut w = Self {
            name: "Eau".to_string(),
            waves: Vec::new(),
            water_level: 0.0,
            foam_threshold: 0.8,
            depth: 20.0,
            density: 1025.0,
            surface_tension: 0.072,
            caustic_intensity: 1.0,
            color: [0.05, 0.25, 0.45, 0.9],
            absorption: [0.45, 0.08, 0.02],
            enabled: true,
        };
        let mut wave1 = GerstnerWave::default();
        wave1.amplitude = 0.3;
        wave1.wavelength = 6.0;
        w.waves.push(wave1);
        let mut wave2 = GerstnerWave::default();
        wave2.amplitude = 0.15;
        wave2.wavelength = 3.5;
        wave2.direction = [0.7, 0.7];
        wave2.speed = 1.5;
        w.waves.push(wave2);
        let mut wave3 = GerstnerWave::default();
        wave3.amplitude = 0.08;
        wave3.wavelength = 1.5;
        wave3.direction = [-0.5, 0.866];
        wave3.speed = 2.0;
        w.waves.push(wave3);
        w
    }
}

impl WaterBody {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into(), ..Self::default() }
    }

    pub fn surface_height(&self, pos: [f64; 2], time: f64) -> f64 {
        let mut h = self.water_level;
        for wave in &self.waves {
            if wave.enabled { h += wave.displacement(pos, time)[1]; }
        }
        h
    }

    pub fn tick_buoyancy(&self, body: &mut FloatingBody, dt: f64) {
        let h = self.surface_height([body.position[0], body.position[2]], 0.0);
        let submerged = (h - body.position[1]).clamp(0.0, 1.0);
        body.submerged_fraction = submerged;
        let buoyancy = self.density * 9.81 * body.volume * submerged;
        let gravity = -9.81 * body.mass;
        let net = (buoyancy + gravity) / body.mass;
        body.velocity[1] += net * dt;
        body.velocity[1] *= 1.0 - body.drag * dt;
        body.position[1] += body.velocity[1] * dt;
    }
}
