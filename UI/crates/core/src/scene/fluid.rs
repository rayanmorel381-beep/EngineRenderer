const SPH_H: f64 = 0.5;
const SPH_H2: f64 = SPH_H * SPH_H;
const SPH_H6: f64 = SPH_H * SPH_H * SPH_H * SPH_H * SPH_H * SPH_H;
const SPH_H9: f64 = SPH_H6 * SPH_H * SPH_H * SPH_H;
const SPH_POLY6: f64 = 315.0 / (64.0 * std::f64::consts::PI * SPH_H9);
const SPH_SPIKY_GRAD: f64 = -45.0 / (std::f64::consts::PI * SPH_H6);
const SPH_VISC: f64 = 45.0 / (std::f64::consts::PI * SPH_H6);

#[derive(Clone, Debug)]
pub struct FluidParticle {
    pub position: [f64; 3],
    pub velocity: [f64; 3],
    pub force: [f64; 3],
    pub density: f64,
    pub pressure: f64,
}

impl FluidParticle {
    pub fn new(pos: [f64; 3]) -> Self {
        Self { position: pos, velocity: [0.0; 3], force: [0.0; 3], density: 0.0, pressure: 0.0 }
    }
}

#[derive(Clone, Debug)]
pub struct FluidParams {
    pub rest_density: f64,
    pub gas_constant: f64,
    pub viscosity: f64,
    pub gravity: [f64; 3],
    pub particle_mass: f64,
    pub boundary_min: [f64; 3],
    pub boundary_max: [f64; 3],
    pub restitution: f64,
}

impl Default for FluidParams {
    fn default() -> Self {
        Self {
            rest_density: 1000.0,
            gas_constant: 2.0,
            viscosity: 0.1,
            gravity: [0.0, -9.81, 0.0],
            particle_mass: 0.02,
            boundary_min: [-2.0, 0.0, -2.0],
            boundary_max: [2.0, 4.0, 2.0],
            restitution: 0.3,
        }
    }
}

#[derive(Clone, Debug)]
pub struct FluidVolume {
    pub particles: Vec<FluidParticle>,
    pub params: FluidParams,
    pub enabled: bool,
}

impl FluidVolume {
    pub fn new() -> Self {
        Self { particles: Vec::new(), params: FluidParams::default(), enabled: true }
    }

    pub fn spawn_cube(&mut self, origin: [f64; 3], n: usize, spacing: f64) {
        let side = (n as f64).cbrt().ceil() as usize;
        let mut count = 0;
        'outer: for ix in 0..side {
            for iy in 0..side {
                for iz in 0..side {
                    if count >= n { break 'outer; }
                    let pos = [
                        origin[0] + ix as f64 * spacing,
                        origin[1] + iy as f64 * spacing,
                        origin[2] + iz as f64 * spacing,
                    ];
                    self.particles.push(FluidParticle::new(pos));
                    count += 1;
                }
            }
        }
    }

    pub fn step(&mut self, dt: f64) {
        let n = self.particles.len();
        let m = self.params.particle_mass;
        let rho0 = self.params.rest_density;
        let k = self.params.gas_constant;
        let mu = self.params.viscosity;
        for i in 0..n {
            let mut density = 0.0f64;
            for j in 0..n {
                let dx = [
                    self.particles[j].position[0] - self.particles[i].position[0],
                    self.particles[j].position[1] - self.particles[i].position[1],
                    self.particles[j].position[2] - self.particles[i].position[2],
                ];
                let r2 = dx[0]*dx[0]+dx[1]*dx[1]+dx[2]*dx[2];
                if r2 < SPH_H2 {
                    density += m * SPH_POLY6 * (SPH_H2 - r2).powi(3);
                }
            }
            self.particles[i].density = density.max(1e-9);
            self.particles[i].pressure = k * (density - rho0);
        }
        for i in 0..n {
            let mut fx = self.params.gravity[0] * m;
            let mut fy = self.params.gravity[1] * m;
            let mut fz = self.params.gravity[2] * m;
            for j in 0..n {
                if i == j { continue; }
                let dx = [
                    self.particles[j].position[0] - self.particles[i].position[0],
                    self.particles[j].position[1] - self.particles[i].position[1],
                    self.particles[j].position[2] - self.particles[i].position[2],
                ];
                let r2 = dx[0]*dx[0]+dx[1]*dx[1]+dx[2]*dx[2];
                if r2 < SPH_H2 && r2 > 1e-9 {
                    let r = r2.sqrt();
                    let dir = [dx[0]/r, dx[1]/r, dx[2]/r];
                    let pressure_f = -m * (self.particles[i].pressure + self.particles[j].pressure)
                        / (2.0 * self.particles[j].density)
                        * SPH_SPIKY_GRAD * (SPH_H - r).powi(2);
                    fx += pressure_f * dir[0];
                    fy += pressure_f * dir[1];
                    fz += pressure_f * dir[2];
                    let dv = [
                        self.particles[j].velocity[0] - self.particles[i].velocity[0],
                        self.particles[j].velocity[1] - self.particles[i].velocity[1],
                        self.particles[j].velocity[2] - self.particles[i].velocity[2],
                    ];
                    let visc_f = mu * m / self.particles[j].density * SPH_VISC * (SPH_H - r);
                    fx += visc_f * dv[0];
                    fy += visc_f * dv[1];
                    fz += visc_f * dv[2];
                }
            }
            self.particles[i].force = [fx, fy, fz];
        }
        let restitution = self.params.restitution;
        let bmin = self.params.boundary_min;
        let bmax = self.params.boundary_max;
        for p in self.particles.iter_mut() {
            let acc = [p.force[0]/m, p.force[1]/m, p.force[2]/m];
            for ax in 0..3 {
                p.velocity[ax] += acc[ax] * dt;
                p.position[ax] += p.velocity[ax] * dt;
                if p.position[ax] < bmin[ax] { p.position[ax] = bmin[ax]; p.velocity[ax] = p.velocity[ax].abs() * restitution; }
                if p.position[ax] > bmax[ax] { p.position[ax] = bmax[ax]; p.velocity[ax] = -p.velocity[ax].abs() * restitution; }
            }
        }
    }
}

impl Default for FluidVolume {
    fn default() -> Self { Self::new() }
}
