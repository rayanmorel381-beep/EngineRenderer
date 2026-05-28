use crate::core::engine::rendering::raytracing::Vec3;

#[derive(Debug, Clone, Copy)]
pub enum Collider {
    Sphere { radius: f64 },
    Box { half_extents: Vec3 },
}

impl Collider {
    pub fn aabb_half(&self) -> Vec3 {
        match self {
            Self::Sphere { radius } => Vec3::splat(*radius),
            Self::Box { half_extents } => *half_extents,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RigidBody {
    pub position: Vec3,
    pub velocity: Vec3,
    pub angular_velocity: Vec3,
    pub orientation: [f64; 4],
    pub mass: f64,
    pub inv_mass: f64,
    pub restitution: f64,
    pub friction: f64,
    pub collider: Collider,
    pub force_accumulator: Vec3,
    pub is_static: bool,
}

impl RigidBody {
    pub fn new(position: Vec3, mass: f64, collider: Collider) -> Self {
        let inv_mass = if mass > f64::EPSILON { 1.0 / mass } else { 0.0 };
        Self {
            position,
            velocity: Vec3::ZERO,
            angular_velocity: Vec3::ZERO,
            orientation: [0.0, 0.0, 0.0, 1.0],
            mass,
            inv_mass,
            restitution: 0.5,
            friction: 0.3,
            collider,
            force_accumulator: Vec3::ZERO,
            is_static: false,
        }
    }

    pub fn static_body(position: Vec3, collider: Collider) -> Self {
        let mut b = Self::new(position, 0.0, collider);
        b.is_static = true;
        b
    }

    pub fn with_restitution(mut self, r: f64) -> Self {
        self.restitution = r.clamp(0.0, 1.0);
        self
    }

    pub fn with_friction(mut self, f: f64) -> Self {
        self.friction = f.clamp(0.0, 1.0);
        self
    }

    pub fn apply_force(&mut self, force: Vec3) {
        self.force_accumulator += force;
    }

    pub fn apply_impulse(&mut self, impulse: Vec3) {
        if !self.is_static {
            self.velocity += impulse * self.inv_mass;
        }
    }

    pub fn integrate(&mut self, dt: f64) {
        if self.is_static {
            return;
        }
        let accel = self.force_accumulator * self.inv_mass;
        self.velocity += accel * dt;
        self.velocity = self.velocity * (1.0 - self.friction * dt).max(0.0);
        self.position += self.velocity * dt;
        self.angular_velocity = self.angular_velocity * (1.0 - 0.1 * dt).max(0.0);
        self.force_accumulator = Vec3::ZERO;
    }

    pub fn aabb_min(&self) -> Vec3 {
        self.position - self.collider.aabb_half()
    }

    pub fn aabb_max(&self) -> Vec3 {
        self.position + self.collider.aabb_half()
    }

    pub fn kinetic_energy(&self) -> f64 {
        0.5 * self.mass * self.velocity.length_squared()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ContactManifold {
    pub point: Vec3,
    pub normal: Vec3,
    pub depth: f64,
    pub body_a: usize,
    pub body_b: usize,
}

pub fn aabb_overlap(min_a: Vec3, max_a: Vec3, min_b: Vec3, max_b: Vec3) -> bool {
    min_a.x <= max_b.x
        && max_a.x >= min_b.x
        && min_a.y <= max_b.y
        && max_a.y >= min_b.y
        && min_a.z <= max_b.z
        && max_a.z >= min_b.z
}

fn clamp_vec(v: Vec3, lo: Vec3, hi: Vec3) -> Vec3 {
    Vec3::new(
        v.x.clamp(lo.x, hi.x),
        v.y.clamp(lo.y, hi.y),
        v.z.clamp(lo.z, hi.z),
    )
}

fn sphere_sphere_contact(a_pos: Vec3, ra: f64, b_pos: Vec3, rb: f64) -> Option<ContactManifold> {
    let delta = b_pos - a_pos;
    let dist = delta.length();
    let combined = ra + rb;
    if dist >= combined {
        return None;
    }
    let normal = if dist > 1e-9 {
        delta * (1.0 / dist)
    } else {
        Vec3::new(0.0, 1.0, 0.0)
    };
    Some(ContactManifold {
        point: a_pos + normal * ra,
        normal,
        depth: combined - dist,
        body_a: 0,
        body_b: 0,
    })
}

fn sphere_box_contact(
    sphere_pos: Vec3,
    sr: f64,
    box_pos: Vec3,
    half: Vec3,
) -> Option<ContactManifold> {
    let local = sphere_pos - box_pos;
    let closest = clamp_vec(local, Vec3::ZERO - half, half);
    let diff = local - closest;
    let dist2 = diff.length_squared();
    if dist2 >= sr * sr {
        return None;
    }
    let dist = dist2.sqrt();
    let normal = if dist > 1e-9 {
        diff * (1.0 / dist)
    } else {
        Vec3::new(0.0, 1.0, 0.0)
    };
    Some(ContactManifold {
        point: box_pos + closest,
        normal,
        depth: sr - dist,
        body_a: 0,
        body_b: 0,
    })
}

fn box_box_contact(a_pos: Vec3, ha: Vec3, b_pos: Vec3, hb: Vec3) -> Option<ContactManifold> {
    let overlap_x = (ha.x + hb.x) - (b_pos.x - a_pos.x).abs();
    let overlap_y = (ha.y + hb.y) - (b_pos.y - a_pos.y).abs();
    let overlap_z = (ha.z + hb.z) - (b_pos.z - a_pos.z).abs();
    if overlap_x <= 0.0 || overlap_y <= 0.0 || overlap_z <= 0.0 {
        return None;
    }
    let (axis, depth) = if overlap_x <= overlap_y && overlap_x <= overlap_z {
        (0usize, overlap_x)
    } else if overlap_y <= overlap_z {
        (1, overlap_y)
    } else {
        (2, overlap_z)
    };
    let sign = if b_pos.axis(axis) >= a_pos.axis(axis) {
        1.0
    } else {
        -1.0
    };
    let normal = match axis {
        0 => Vec3::new(sign, 0.0, 0.0),
        1 => Vec3::new(0.0, sign, 0.0),
        _ => Vec3::new(0.0, 0.0, sign),
    };
    Some(ContactManifold {
        point: a_pos + normal * ha.axis(axis),
        normal,
        depth,
        body_a: 0,
        body_b: 0,
    })
}

pub fn narrow_phase(a: &RigidBody, b: &RigidBody) -> Option<ContactManifold> {
    match (&a.collider, &b.collider) {
        (Collider::Sphere { radius: ra }, Collider::Sphere { radius: rb }) => {
            sphere_sphere_contact(a.position, *ra, b.position, *rb)
        }
        (Collider::Sphere { radius: r }, Collider::Box { half_extents: h }) => {
            sphere_box_contact(a.position, *r, b.position, *h)
        }
        (Collider::Box { half_extents: h }, Collider::Sphere { radius: r }) => {
            sphere_box_contact(b.position, *r, a.position, *h).map(|mut c| {
                c.normal = Vec3::ZERO - c.normal;
                c
            })
        }
        (Collider::Box { half_extents: ha }, Collider::Box { half_extents: hb }) => {
            box_box_contact(a.position, *ha, b.position, *hb)
        }
    }
}

pub fn resolve_contact(a: &mut RigidBody, b: &mut RigidBody, contact: &ContactManifold) {
    let rel_vel = b.velocity - a.velocity;
    let vel_along_normal = rel_vel.dot(contact.normal);
    if vel_along_normal > 0.0 {
        return;
    }
    let restitution = a.restitution.min(b.restitution);
    let sum_inv = a.inv_mass + b.inv_mass;
    if sum_inv < f64::EPSILON {
        return;
    }
    let j = -(1.0 + restitution) * vel_along_normal / sum_inv;
    let impulse = contact.normal * j;
    if !a.is_static {
        a.velocity = a.velocity - impulse * a.inv_mass;
    }
    if !b.is_static {
        b.velocity += impulse * b.inv_mass;
    }
    let slop = 0.01;
    let correction_pct = 0.2;
    let correction =
        contact.normal * ((contact.depth - slop).max(0.0) / sum_inv * correction_pct);
    if !a.is_static {
        a.position = a.position - correction * a.inv_mass;
    }
    if !b.is_static {
        b.position += correction * b.inv_mass;
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Joint {
    Distance {
        body_a: usize,
        body_b: usize,
        rest_length: f64,
        stiffness: f64,
    },
    Hinge {
        body_a: usize,
        body_b: usize,
        axis: Vec3,
        min_angle: f64,
        max_angle: f64,
    },
}

impl Joint {
    pub fn solve(&self, bodies: &mut [RigidBody], dt: f64) {
        match self {
            Self::Distance {
                body_a,
                body_b,
                rest_length,
                stiffness,
            } => {
                let delta = bodies[*body_b].position - bodies[*body_a].position;
                let dist = delta.length();
                if dist < 1e-9 {
                    return;
                }
                let error = dist - rest_length;
                let sum_inv = bodies[*body_a].inv_mass + bodies[*body_b].inv_mass;
                if sum_inv < f64::EPSILON {
                    return;
                }
                let correction = delta.normalize() * error * (*stiffness) * dt;
                let wa = bodies[*body_a].inv_mass / sum_inv;
                let wb = bodies[*body_b].inv_mass / sum_inv;
                if !bodies[*body_a].is_static {
                    bodies[*body_a].position += correction * wa;
                }
                if !bodies[*body_b].is_static {
                    bodies[*body_b].position = bodies[*body_b].position - correction * wb;
                }
            }
            Self::Hinge {
                body_a,
                body_b,
                axis,
                min_angle,
                max_angle,
            } => {
                let delta = bodies[*body_b].position - bodies[*body_a].position;
                let along = *axis * delta.dot(*axis);
                let perp = delta - along;
                let angle = perp.length().atan2(delta.dot(*axis));
                let clamped = angle.clamp(*min_angle, *max_angle);
                if (clamped - angle).abs() < 1e-6 {
                    return;
                }
                let perp_norm = if perp.length() > 1e-9 {
                    perp.normalize()
                } else {
                    Vec3::new(1.0, 0.0, 0.0)
                };
                let correction = perp_norm.cross(*axis).normalize() * ((clamped - angle) * 0.5 * dt);
                if !bodies[*body_a].is_static {
                    bodies[*body_a].position = bodies[*body_a].position - correction;
                }
                if !bodies[*body_b].is_static {
                    bodies[*body_b].position += correction;
                }
            }
        }
    }
}

pub struct PhysicsWorld {
    pub bodies: Vec<RigidBody>,
    pub joints: Vec<Joint>,
    pub gravity: Vec3,
    pub solver_iterations: usize,
    pub fluid_sim: Option<crate::core::simulation::fluid::FluidSim>,
    pub cloth_grids: Vec<crate::core::simulation::cloth::ClothGrid>,
    pub fracture_bodies: Vec<crate::core::simulation::fracture::FractureBody>,
    pub vehicles: Vec<crate::core::simulation::vehicle::Vehicle>,
}

impl PhysicsWorld {
    pub fn new() -> Self {
        Self {
            bodies: Vec::new(),
            joints: Vec::new(),
            gravity: Vec3::new(0.0, -9.81, 0.0),
            solver_iterations: 8,
            fluid_sim: None,
            cloth_grids: Vec::new(),
            fracture_bodies: Vec::new(),
            vehicles: Vec::new(),
        }
    }

    pub fn add_body(&mut self, body: RigidBody) -> usize {
        let idx = self.bodies.len();
        self.bodies.push(body);
        idx
    }

    pub fn add_joint(&mut self, joint: Joint) {
        self.joints.push(joint);
    }

    pub fn step(&mut self, dt: f64) {
        for body in &mut self.bodies {
            if !body.is_static {
                body.apply_force(self.gravity * body.mass);
            }
        }

        for body in &mut self.bodies {
            body.integrate(dt);
        }

        let cell_size = self.bodies.iter()
            .map(|b| b.collider.aabb_half().length() * 2.0)
            .fold(1.0_f64, |a, b| a.max(b));
        let mut hash = crate::core::simulation::broadphase::SpatialHash::new(cell_size);
        for (i, body) in self.bodies.iter().enumerate() {
            hash.insert(i, body.aabb_min(), body.aabb_max());
        }
        let origin_cell = crate::core::simulation::broadphase::cell_coord(Vec3::ZERO, cell_size);
        crate::runtime_log!(
            "broadphase: cell_count={} origin_cell={:?}",
            hash.cell_count(),
            origin_cell,
        );
        let pairs = hash.query_pairs();
        let mut contacts: Vec<ContactManifold> = Vec::new();
        for (i, j) in pairs {
            if !aabb_overlap(
                self.bodies[i].aabb_min(),
                self.bodies[i].aabb_max(),
                self.bodies[j].aabb_min(),
                self.bodies[j].aabb_max(),
            ) {
                continue;
            }
            if let Some(mut contact) = narrow_phase(&self.bodies[i], &self.bodies[j]) {
                contact.body_a = i;
                contact.body_b = j;
                contacts.push(contact);
            }
        }

        for contact in contacts {
            let (left, right) = self.bodies.split_at_mut(contact.body_b);
            resolve_contact(&mut left[contact.body_a], &mut right[0], &contact);
        }

        let joints: Vec<Joint> = self.joints.clone();
        for _ in 0..self.solver_iterations {
            for joint in &joints {
                joint.solve(&mut self.bodies, dt);
            }
        }

        if let Some(ref mut fluid) = self.fluid_sim {
            fluid.step(dt);
        }

        let gravity = self.gravity;
        let iterations = self.solver_iterations;
        for cloth in &mut self.cloth_grids {
            cloth.step(dt, gravity, iterations);
            cloth.resolve_rigid_collisions(&self.bodies);
        }

        for vehicle in &mut self.vehicles {
            vehicle.step(dt, gravity);
        }

        let total_fractured: usize = self.fracture_bodies.iter()
            .filter(|fb| fb.is_fully_fractured())
            .count();
        if total_fractured > 0 {
            crate::runtime_log!("fracture: fully_fractured={}", total_fractured);
        }
    }

    pub fn body_count(&self) -> usize {
        self.bodies.len()
    }

    pub fn joint_count(&self) -> usize {
        self.joints.len()
    }

    pub fn total_kinetic_energy(&self) -> f64 {
        self.bodies.iter().map(|b| b.kinetic_energy()).sum()
    }
}

impl Default for PhysicsWorld {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for PhysicsWorld {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PhysicsWorld")
            .field("body_count", &self.bodies.len())
            .finish()
    }
}
