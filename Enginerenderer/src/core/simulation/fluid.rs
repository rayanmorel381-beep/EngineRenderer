use crate::core::engine::rendering::raytracing::Vec3;

#[derive(Debug, Clone)]
pub struct FluidParticle {
    pub position: Vec3,
    pub velocity: Vec3,
    pub density: f64,
    pub pressure: f64,
    pub force: Vec3,
}

impl FluidParticle {
    pub fn new(position: Vec3) -> Self {
        Self { position, velocity: Vec3::ZERO, density: 0.0, pressure: 0.0, force: Vec3::ZERO }
    }
}

pub struct FluidSim {
    pub particles: Vec<FluidParticle>,
    pub h: f64,
    pub mass: f64,
    pub rest_density: f64,
    pub stiffness: f64,
    pub viscosity: f64,
    pub gravity: Vec3,
}

impl FluidSim {
    pub fn new(rest_density: f64, stiffness: f64, viscosity: f64, h: f64, mass: f64) -> Self {
        Self {
            particles: Vec::new(),
            h,
            mass,
            rest_density,
            stiffness,
            viscosity,
            gravity: Vec3::new(0.0, -9.81, 0.0),
        }
    }

    pub fn add_particle(&mut self, position: Vec3) {
        self.particles.push(FluidParticle::new(position));
    }

    pub fn particle_count(&self) -> usize {
        self.particles.len()
    }

    pub fn step(&mut self, dt: f64) {
        self.compute_density();
        self.compute_pressure();
        self.compute_forces();
        self.integrate(dt);
    }

    fn compute_density(&mut self) {
        let n = self.particles.len();
        let positions: Vec<Vec3> = self.particles.iter().map(|p| p.position).collect();
        for i in 0..n {
            let mut density = 0.0_f64;
            for j in 0..n {
                let r = positions[i] - positions[j];
                let r_sq = r.dot(r);
                if r_sq < self.h * self.h {
                    density += self.mass * w_poly6(r_sq, self.h);
                }
            }
            self.particles[i].density = density.max(1e-6);
        }
    }

    fn compute_pressure(&mut self) {
        for p in &mut self.particles {
            p.pressure = self.stiffness * (p.density / self.rest_density - 1.0).max(0.0);
        }
    }

    fn compute_forces(&mut self) {
        let n = self.particles.len();
        let positions: Vec<Vec3> = self.particles.iter().map(|p| p.position).collect();
        let velocities: Vec<Vec3> = self.particles.iter().map(|p| p.velocity).collect();
        let densities: Vec<f64> = self.particles.iter().map(|p| p.density).collect();
        let pressures: Vec<f64> = self.particles.iter().map(|p| p.pressure).collect();

        for i in 0..n {
            let mut force = self.gravity * self.mass;
            for j in 0..n {
                if i == j { continue; }
                let r = positions[i] - positions[j];
                let r_len = r.length();
                if r_len < self.h && r_len > 1e-8 {
                    let dir = r * (1.0 / r_len);
                    let pressure_term = -self.mass
                        * (pressures[i] / (densities[i] * densities[i])
                            + pressures[j] / (densities[j] * densities[j]))
                        * w_spiky_grad(r_len, self.h);
                    force += dir * pressure_term;

                    let visc_lap = w_viscosity_laplacian(r_len, self.h);
                    let visc_term = self.viscosity * self.mass / densities[j]
                        * (velocities[j] - velocities[i])
                        * visc_lap;
                    force += visc_term;
                }
            }
            self.particles[i].force = force;
        }
    }

    fn integrate(&mut self, dt: f64) {
        for p in &mut self.particles {
            let accel = p.force * (1.0 / p.density.max(1e-6));
            p.velocity += accel * dt;
            p.position += p.velocity * dt;
        }
    }
}

fn w_poly6(r_sq: f64, h: f64) -> f64 {
    let h2 = h * h;
    if r_sq > h2 { return 0.0; }
    let factor = 315.0 / (64.0 * std::f64::consts::PI * h.powi(9));
    let diff = h2 - r_sq;
    factor * diff * diff * diff
}

fn w_spiky_grad(r: f64, h: f64) -> f64 {
    if r > h { return 0.0; }
    let factor = -45.0 / (std::f64::consts::PI * h.powi(6));
    let diff = h - r;
    factor * diff * diff
}

fn w_viscosity_laplacian(r: f64, h: f64) -> f64 {
    if r > h { return 0.0; }
    45.0 / (std::f64::consts::PI * h.powi(6)) * (h - r)
}
