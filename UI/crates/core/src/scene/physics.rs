#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ColliderShape {
    Box,
    Sphere,
    Capsule,
}

impl Default for ColliderShape {
    fn default() -> Self { Self::Box }
}

#[derive(Clone, Debug)]
pub struct Collider {
    pub shape: ColliderShape,
    pub half_extents: [f64; 3],
    pub is_trigger: bool,
}

impl Default for Collider {
    fn default() -> Self {
        Self {
            shape: ColliderShape::Box,
            half_extents: [0.5, 0.5, 0.5],
            is_trigger: false,
        }
    }
}

#[derive(Clone, Debug)]
pub struct PhysicsBody {
    pub mass: f64,
    pub velocity: [f64; 3],
    pub angular_velocity: [f64; 3],
    pub friction: f64,
    pub restitution: f64,
    pub linear_damping: f64,
    pub angular_damping: f64,
    pub is_static: bool,
    pub use_gravity: bool,
    pub collider: Collider,
}

impl Default for PhysicsBody {
    fn default() -> Self {
        Self {
            mass: 1.0,
            velocity: [0.0; 3],
            angular_velocity: [0.0; 3],
            friction: 0.5,
            restitution: 0.3,
            linear_damping: 0.05,
            angular_damping: 0.05,
            is_static: false,
            use_gravity: true,
            collider: Collider::default(),
        }
    }
}

const GRAVITY: f64 = -9.81;

pub fn step_physics(body: &mut PhysicsBody, position: &mut [f64; 3], dt: f64) {
    if body.is_static { return; }
    if body.use_gravity {
        body.velocity[1] += GRAVITY * dt;
    }
    position[0] += body.velocity[0] * dt;
    position[1] += body.velocity[1] * dt;
    position[2] += body.velocity[2] * dt;
    let ld = 1.0 - body.linear_damping.clamp(0.0, 1.0) * dt;
    body.velocity[0] *= ld;
    body.velocity[1] *= ld;
    body.velocity[2] *= ld;
    let ad = 1.0 - body.angular_damping.clamp(0.0, 1.0) * dt;
    body.angular_velocity[0] *= ad;
    body.angular_velocity[1] *= ad;
    body.angular_velocity[2] *= ad;
    if position[1] < 0.0 {
        position[1] = 0.0;
        body.velocity[1] = -body.velocity[1] * body.restitution;
    }
}
