#[derive(Clone, Debug, PartialEq)]
pub enum EmitterShape {
    Point,
    Sphere,
    Box,
    Cone,
}

impl Default for EmitterShape {
    fn default() -> Self { Self::Point }
}

impl EmitterShape {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Point => "Point",
            Self::Sphere => "Sphere",
            Self::Box => "Box",
            Self::Cone => "Cone",
        }
    }
    pub const ALL: [EmitterShape; 4] = [EmitterShape::Point, EmitterShape::Sphere, EmitterShape::Box, EmitterShape::Cone];
}

#[derive(Clone, Debug, PartialEq)]
pub enum SimSpace {
    World,
    Local,
}

impl Default for SimSpace {
    fn default() -> Self { Self::World }
}

impl SimSpace {
    pub fn label(&self) -> &'static str {
        match self { Self::World => "World", Self::Local => "Local" }
    }
    pub const ALL: [SimSpace; 2] = [SimSpace::World, SimSpace::Local];
}

#[derive(Clone, Debug)]
pub struct Particle {
    pub position: [f64; 3],
    pub velocity: [f64; 3],
    pub age: f64,
    pub lifetime: f64,
    pub size: f64,
    pub color: [f64; 4],
}

impl Particle {
    pub fn alive(&self) -> bool { self.age < self.lifetime }
    pub fn normalized_age(&self) -> f64 {
        if self.lifetime <= 0.0 { 1.0 } else { (self.age / self.lifetime).clamp(0.0, 1.0) }
    }
}

#[derive(Clone, Debug)]
pub struct ParticleEmitter {
    pub name: String,
    pub enabled: bool,
    pub shape: EmitterShape,
    pub sim_space: SimSpace,
    pub max_particles: usize,
    pub emission_rate: f64,
    pub burst_count: usize,
    pub lifetime_min: f64,
    pub lifetime_max: f64,
    pub start_size_min: f64,
    pub start_size_max: f64,
    pub start_speed_min: f64,
    pub start_speed_max: f64,
    pub start_color: [f64; 4],
    pub end_color: [f64; 4],
    pub gravity_scale: f64,
    pub looping: bool,
    pub duration: f64,
    pub shape_radius: f64,
    pub shape_angle: f64,
    pub size_over_lifetime: bool,
    pub color_over_lifetime: bool,
    particles: Vec<Particle>,
    emission_accum: f64,
    time: f64,
    rng_state: u64,
}

impl Default for ParticleEmitter {
    fn default() -> Self {
        Self {
            name: "Particles".to_string(),
            enabled: true,
            shape: EmitterShape::Cone,
            sim_space: SimSpace::World,
            max_particles: 100,
            emission_rate: 10.0,
            burst_count: 0,
            lifetime_min: 1.0,
            lifetime_max: 2.0,
            start_size_min: 0.05,
            start_size_max: 0.15,
            start_speed_min: 1.0,
            start_speed_max: 3.0,
            start_color: [1.0, 0.5, 0.1, 1.0],
            end_color: [0.8, 0.1, 0.0, 0.0],
            gravity_scale: 0.0,
            looping: true,
            duration: 5.0,
            shape_radius: 0.5,
            shape_angle: 25.0,
            size_over_lifetime: true,
            color_over_lifetime: true,
            particles: Vec::new(),
            emission_accum: 0.0,
            time: 0.0,
            rng_state: 12345,
        }
    }
}

impl ParticleEmitter {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into(), ..Self::default() }
    }

    pub fn particle_count(&self) -> usize { self.particles.iter().filter(|p| p.alive()).count() }

    fn lcg_rand(&mut self) -> f64 {
        self.rng_state = self.rng_state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        let bits = ((self.rng_state >> 33) as u32) as f64;
        bits / 4294967295.0
    }

    fn rand_range(&mut self, min: f64, max: f64) -> f64 {
        min + self.lcg_rand() * (max - min)
    }

    fn spawn_particle(&mut self, origin: [f64; 3]) {
        let speed = self.rand_range(self.start_speed_min, self.start_speed_max);
        let lifetime = self.rand_range(self.lifetime_min, self.lifetime_max);
        let size = self.rand_range(self.start_size_min, self.start_size_max);

        let theta = self.rand_range(0.0, std::f64::consts::TAU);
        let angle_rad = (self.shape_angle * std::f64::consts::PI / 180.0).clamp(0.0, std::f64::consts::FRAC_PI_2);
        let phi = self.rand_range(0.0, angle_rad);
        let vx = phi.sin() * theta.cos() * speed;
        let vy = phi.cos() * speed;
        let vz = phi.sin() * theta.sin() * speed;

        let ox = match self.shape {
            EmitterShape::Sphere | EmitterShape::Box => self.rand_range(-self.shape_radius, self.shape_radius),
            _ => 0.0,
        };
        let oz = match self.shape {
            EmitterShape::Sphere | EmitterShape::Box => self.rand_range(-self.shape_radius, self.shape_radius),
            _ => 0.0,
        };

        let p = Particle {
            position: [origin[0] + ox, origin[1], origin[2] + oz],
            velocity: [vx, vy, vz],
            age: 0.0,
            lifetime,
            size,
            color: self.start_color,
        };
        if let Some(slot) = self.particles.iter_mut().find(|p| !p.alive()) {
            *slot = p;
        } else if self.particles.len() < self.max_particles {
            self.particles.push(p);
        }
    }

    pub fn step(&mut self, origin: [f64; 3], dt: f64) {
        if !self.enabled { return; }
        self.time += dt;

        let gravity = 9.81 * self.gravity_scale;
        for p in self.particles.iter_mut() {
            if !p.alive() { continue; }
            p.velocity[1] -= gravity * dt;
            p.position[0] += p.velocity[0] * dt;
            p.position[1] += p.velocity[1] * dt;
            p.position[2] += p.velocity[2] * dt;
            p.age += dt;
            if self.color_over_lifetime {
                let t = p.normalized_age();
                for ci in 0..4 {
                    p.color[ci] = self.start_color[ci] * (1.0 - t) + self.end_color[ci] * t;
                }
            }
            if self.size_over_lifetime {
                let t = p.normalized_age();
                p.size *= 1.0 - t * 0.5;
            }
        }

        if self.looping || self.time <= self.duration {
            self.emission_accum += self.emission_rate * dt;
            while self.emission_accum >= 1.0 {
                self.spawn_particle(origin);
                self.emission_accum -= 1.0;
            }
        }
    }

    pub fn particles(&self) -> &[Particle] { &self.particles }
}
