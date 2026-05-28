use crate::core::engine::rendering::raytracing::Vec3;
use super::rigidbody::{Collider, RigidBody};

#[derive(Debug, Clone)]
pub struct ClothParticle {
    pub position: Vec3,
    pub prev_position: Vec3,
    pub velocity: Vec3,
    pub inv_mass: f64,
    pub pinned: bool,
}

impl ClothParticle {
    pub fn new(position: Vec3, mass: f64) -> Self {
        let inv_mass = if mass > f64::EPSILON { 1.0 / mass } else { 0.0 };
        Self {
            position,
            prev_position: position,
            velocity: Vec3::ZERO,
            inv_mass,
            pinned: false,
        }
    }

    pub fn pinned(position: Vec3) -> Self {
        let mut p = Self::new(position, 1.0);
        p.pinned = true;
        p
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Spring {
    pub a: usize,
    pub b: usize,
    pub rest_length: f64,
    pub stiffness: f64,
    pub damping: f64,
}

impl Spring {
    pub fn new(a: usize, b: usize, rest_length: f64) -> Self {
        Self { a, b, rest_length, stiffness: 0.9, damping: 0.01 }
    }

    pub fn with_stiffness(mut self, s: f64) -> Self {
        self.stiffness = s;
        self
    }
}

pub struct ClothGrid {
    pub particles: Vec<ClothParticle>,
    pub springs: Vec<Spring>,
    pub width: usize,
    pub height: usize,
}

impl ClothGrid {
    pub fn new(cols: usize, rows: usize, spacing: f64, origin: Vec3) -> Self {
        let mut particles = Vec::with_capacity(cols * rows);
        for row in 0..rows {
            for col in 0..cols {
                let pos = origin
                    + Vec3::new(col as f64 * spacing, 0.0, row as f64 * spacing);
                let p = if row == 0 {
                    ClothParticle::pinned(pos)
                } else {
                    ClothParticle::new(pos, 1.0)
                };
                particles.push(p);
            }
        }

        let mut springs = Vec::new();
        for row in 0..rows {
            for col in 0..cols {
                let idx = row * cols + col;
                if col + 1 < cols {
                    let right = row * cols + col + 1;
                    let len = (particles[right].position - particles[idx].position).length();
                    springs.push(Spring::new(idx, right, len));
                }
                if row + 1 < rows {
                    let down = (row + 1) * cols + col;
                    let len = (particles[down].position - particles[idx].position).length();
                    springs.push(Spring::new(idx, down, len));
                }
                if col + 1 < cols && row + 1 < rows {
                    let diag = (row + 1) * cols + col + 1;
                    let len = (particles[diag].position - particles[idx].position).length();
                    springs.push(Spring::new(idx, diag, len).with_stiffness(0.5));
                }
                if col + 2 < cols {
                    let shear = row * cols + col + 2;
                    let len = (particles[shear].position - particles[idx].position).length();
                    springs.push(Spring::new(idx, shear, len).with_stiffness(0.3));
                }
                if row + 2 < rows {
                    let bend = (row + 2) * cols + col;
                    let len = (particles[bend].position - particles[idx].position).length();
                    springs.push(Spring::new(idx, bend, len).with_stiffness(0.3));
                }
            }
        }

        Self { particles, springs, width: cols, height: rows }
    }

    pub fn step(&mut self, dt: f64, gravity: Vec3, iterations: usize) {
        let damping = 1.0 - 0.02 * dt;
        for p in &mut self.particles {
            if p.pinned {
                continue;
            }
            let velocity = (p.position - p.prev_position) * damping;
            let new_pos = p.position + velocity + gravity * dt * dt;
            p.prev_position = p.position;
            p.position = new_pos;
            p.velocity = velocity * (1.0 / dt);
        }

        let springs: Vec<Spring> = self.springs.clone();
        for _ in 0..iterations {
            for spring in &springs {
                let pa = self.particles[spring.a].position;
                let pb = self.particles[spring.b].position;
                let delta = pb - pa;
                let dist = delta.length();
                if dist < f64::EPSILON {
                    continue;
                }
                let error = dist - spring.rest_length;
                let sum_inv = self.particles[spring.a].inv_mass
                    + self.particles[spring.b].inv_mass;
                if sum_inv < f64::EPSILON {
                    continue;
                }
                let correction = delta.normalize() * error * spring.stiffness;
                let wa = self.particles[spring.a].inv_mass / sum_inv;
                let wb = self.particles[spring.b].inv_mass / sum_inv;
                if !self.particles[spring.a].pinned {
                    self.particles[spring.a].position += correction * wa;
                }
                if !self.particles[spring.b].pinned {
                    self.particles[spring.b].position =
                        self.particles[spring.b].position - correction * wb;
                }
            }
        }
    }

    pub fn resolve_rigid_collisions(&mut self, bodies: &[RigidBody]) {
        for particle in &mut self.particles {
            if particle.pinned {
                continue;
            }
            for body in bodies {
                match body.collider {
                    Collider::Sphere { radius } => {
                        let delta = particle.position - body.position;
                        let dist = delta.length();
                        if dist < radius {
                            let push = if dist > 1e-9 {
                                delta * ((radius - dist) / dist)
                            } else {
                                Vec3::new(0.0, radius, 0.0)
                            };
                            particle.position += push;
                        }
                    }
                    Collider::Box { half_extents } => {
                        let local = particle.position - body.position;
                        let neg_half = Vec3::ZERO - half_extents;
                        let clamped = Vec3::new(
                            local.x.clamp(neg_half.x, half_extents.x),
                            local.y.clamp(neg_half.y, half_extents.y),
                            local.z.clamp(neg_half.z, half_extents.z),
                        );
                        if (local - clamped).length_squared() < f64::EPSILON {
                            let px = half_extents.x - local.x.abs();
                            let py = half_extents.y - local.y.abs();
                            let pz = half_extents.z - local.z.abs();
                            let (axis, depth) = if px <= py && px <= pz {
                                (Vec3::new(local.x.signum(), 0.0, 0.0), px)
                            } else if py <= pz {
                                (Vec3::new(0.0, local.y.signum(), 0.0), py)
                            } else {
                                (Vec3::new(0.0, 0.0, local.z.signum()), pz)
                            };
                            particle.position += axis * depth;
                        }
                    }
                }
            }
        }
    }

    pub fn particle_count(&self) -> usize {
        self.particles.len()
    }

    pub fn spring_count(&self) -> usize {
        self.springs.len()
    }

    pub fn average_velocity(&self) -> f64 {
        if self.particles.is_empty() {
            return 0.0;
        }
        let total: f64 = self.particles.iter().map(|p| p.velocity.length()).sum();
        total / self.particles.len() as f64
    }
}
