use crate::core::engine::rendering::raytracing::Vec3;
use crate::core::engine::rendering::framebuffer::FrameBuffer;

pub const MAX_PARTICLES: usize = 65_536;

#[derive(Debug, Clone, Copy)]
pub struct Particle {
    pub position: Vec3,
    pub velocity: Vec3,
    pub age: f64,
    pub lifetime: f64,
    pub size: f64,
    pub color: Vec3,
    pub alpha: f64,
}

impl Particle {
    fn normalized_age(&self) -> f64 {
        if self.lifetime > 0.0 { self.age / self.lifetime } else { 1.0 }
    }
}

#[derive(Debug, Clone)]
pub struct ColorGradient {
    pub stops: Vec<(f64, Vec3, f64)>,
}

impl ColorGradient {
    pub fn new(stops: Vec<(f64, Vec3, f64)>) -> Self {
        Self { stops }
    }

    pub fn constant(color: Vec3, alpha: f64) -> Self {
        Self { stops: vec![(0.0, color, alpha), (1.0, color, alpha)] }
    }

    pub fn sample(&self, t: f64) -> (Vec3, f64) {
        if self.stops.is_empty() { return (Vec3::ONE, 1.0); }
        if t <= self.stops[0].0 { return (self.stops[0].1, self.stops[0].2); }
        let last = self.stops.last().unwrap();
        if t >= last.0 { return (last.1, last.2); }
        let next = self.stops.partition_point(|s| s.0 <= t);
        let (t0, c0, a0) = self.stops[next - 1];
        let (t1, c1, a1) = self.stops[next];
        let f = (t - t0) / (t1 - t0);
        (c0 * (1.0 - f) + c1 * f, a0 * (1.0 - f) + a1 * f)
    }
}

#[derive(Debug, Clone)]
pub struct ParticleEmitter {
    pub position: Vec3,
    pub emission_rate: f64,
    pub burst_count: u32,
    pub lifetime_min: f64,
    pub lifetime_max: f64,
    pub initial_speed_min: f64,
    pub initial_speed_max: f64,
    pub spread_angle: f64,
    pub emission_direction: Vec3,
    pub gravity: Vec3,
    pub drag: f64,
    pub size_start: f64,
    pub size_end: f64,
    pub color_over_life: ColorGradient,
    pub simulation_space: SimulationSpace,
    pub emission_accumulator: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SimulationSpace {
    World,
    Local,
}

impl ParticleEmitter {
    pub fn new(position: Vec3) -> Self {
        Self {
            position,
            emission_rate: 100.0,
            burst_count: 0,
            lifetime_min: 1.0,
            lifetime_max: 2.0,
            initial_speed_min: 1.0,
            initial_speed_max: 3.0,
            spread_angle: 0.5,
            emission_direction: Vec3::new(0.0, 1.0, 0.0),
            gravity: Vec3::new(0.0, -9.81, 0.0),
            drag: 0.1,
            size_start: 0.1,
            size_end: 0.0,
            color_over_life: ColorGradient::constant(Vec3::ONE, 1.0),
            simulation_space: SimulationSpace::World,
            emission_accumulator: 0.0,
        }
    }
}

fn lcg_next(seed: &mut u32) -> f64 {
    *seed = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
    (*seed >> 8) as f64 / 16_777_216.0
}

fn cone_direction(base: Vec3, spread: f64, seed: &mut u32) -> Vec3 {
    use std::f64::consts::TAU;
    let cos_spread = (spread * 0.5).cos();
    let cos_theta = cos_spread + (1.0 - cos_spread) * lcg_next(seed);
    let sin_theta = (1.0 - cos_theta * cos_theta).max(0.0).sqrt();
    let phi = TAU * lcg_next(seed);
    let local = Vec3::new(sin_theta * phi.cos(), cos_theta, sin_theta * phi.sin());
    let base_n = base.normalize();
    let up = if base_n.y.abs() < 0.99 { Vec3::new(0.0, 1.0, 0.0) } else { Vec3::new(1.0, 0.0, 0.0) };
    let right = base_n.cross(up).normalize();
    let fwd = right.cross(base_n).normalize();
    right * local.x + base_n * local.y + fwd * local.z
}

pub struct GpuParticleSystem {
    pub particles: Vec<Particle>,
    pub emitters: Vec<ParticleEmitter>,
    pub max_particles: usize,
    pub frame_seed: u32,
}

impl GpuParticleSystem {
    pub fn new(max_particles: usize) -> Self {
        Self {
            particles: Vec::with_capacity(max_particles),
            emitters: Vec::new(),
            max_particles,
            frame_seed: 0xDEAD_BEEF,
        }
    }

    pub fn add_emitter(&mut self, emitter: ParticleEmitter) {
        self.emitters.push(emitter);
    }

    pub fn update(&mut self, dt: f64) {
        self.frame_seed = self.frame_seed.wrapping_mul(0x9E37_79B9).wrapping_add(0x6C62_272E);

        let (shared_gravity, shared_drag) = if self.emitters.is_empty() {
            (Vec3::new(0.0, -9.81, 0.0), 0.05)
        } else {
            let n = self.emitters.len() as f64;
            let g = self.emitters.iter().fold(Vec3::ZERO, |a, e| a + e.gravity) * (1.0 / n);
            let d = self.emitters.iter().map(|e| e.drag).sum::<f64>() / n;
            (g, d)
        };

        for particle in &mut self.particles {
            let effective_drag = (particle.age * particle.age * 0.5 * shared_drag).min(0.99);
            particle.velocity = particle.velocity * (1.0 - effective_drag) + shared_gravity * dt;
            particle.position += particle.velocity * dt;
            particle.age += dt;
            let t = particle.normalized_age();
            particle.size *= 1.0 - t * 0.5;
            particle.alpha = (1.0 - t).powi(2);
        }

        self.particles.retain(|p| p.age < p.lifetime);

        let local_space = SimulationSpace::Local;
        let mut seed = self.frame_seed;
        for emitter in &mut self.emitters {
            let speed_scale = if emitter.simulation_space == local_space { 0.9 } else { 1.0 };
            let min_size = emitter.size_end;
            emitter.emission_accumulator += emitter.emission_rate * dt;
            let to_emit = emitter.emission_accumulator as u32 + emitter.burst_count;
            emitter.emission_accumulator -= to_emit as f64;
            emitter.burst_count = 0;

            let available = self.max_particles.saturating_sub(self.particles.len());
            let count = (to_emit as usize).min(available);

            for _ in 0..count {
                let lifetime = emitter.lifetime_min + (emitter.lifetime_max - emitter.lifetime_min) * lcg_next(&mut seed);
                let speed = (emitter.initial_speed_min + (emitter.initial_speed_max - emitter.initial_speed_min) * lcg_next(&mut seed)) * speed_scale;
                let dir = cone_direction(emitter.emission_direction, emitter.spread_angle, &mut seed);
                let (color, alpha) = emitter.color_over_life.sample(0.0);
                self.particles.push(Particle {
                    position: emitter.position,
                    velocity: dir * speed,
                    age: 0.0,
                    lifetime,
                    size: emitter.size_start.max(min_size),
                    color,
                    alpha,
                });
            }
        }

        self.frame_seed = seed;
    }

    pub fn draw(&self, fb: &mut FrameBuffer, view_proj: &[[f64; 4]; 4]) {
        for particle in &self.particles {
            let (cx, cy, cz, cw) = project_point(particle.position, view_proj);
            if cw < 1e-6 || cz < 0.0 { continue; }
            let ndcx = cx / cw;
            let ndcy = cy / cw;
            let depth = cz / cw;
            let px = ((ndcx * 0.5 + 0.5) * fb.width as f64) as i64;
            let py = ((1.0 - (ndcy * 0.5 + 0.5)) * fb.height as f64) as i64;
            let radius = ((particle.size * fb.width as f64) / cw).max(1.0) as i64;

            for dy in -radius..=radius {
                for dx in -radius..=radius {
                    if dx*dx + dy*dy > radius*radius { continue; }
                    let sx = px + dx;
                    let sy = py + dy;
                    if sx < 0 || sx >= fb.width as i64 || sy < 0 || sy >= fb.height as i64 { continue; }
                    let idx = sy as usize * fb.width + sx as usize;
                    if depth >= fb.depth[idx] { continue; }
                    let a = particle.alpha;
                    fb.color[idx] = fb.color[idx] * (1.0 - a) + particle.color * a;
                    fb.alpha[idx] = (fb.alpha[idx] + a).min(1.0);
                }
            }
        }
    }
}

fn project_point(p: Vec3, m: &[[f64; 4]; 4]) -> (f64, f64, f64, f64) {
    let x = m[0][0]*p.x + m[1][0]*p.y + m[2][0]*p.z + m[3][0];
    let y = m[0][1]*p.x + m[1][1]*p.y + m[2][1]*p.z + m[3][1];
    let z = m[0][2]*p.x + m[1][2]*p.y + m[2][2]*p.z + m[3][2];
    let w = m[0][3]*p.x + m[1][3]*p.y + m[2][3]*p.z + m[3][3];
    (x, y, z, w)
}
