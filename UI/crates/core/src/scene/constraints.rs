use crate::scene::ObjectId;

#[derive(Clone, Debug, PartialEq)]
pub enum JointKind {
    Fixed,
    Revolute,
    Prismatic,
    Spherical,
    Distance,
}

impl JointKind {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Fixed => "Fixe",
            Self::Revolute => "Pivot",
            Self::Prismatic => "Glissière",
            Self::Spherical => "Sphérique",
            Self::Distance => "Distance",
        }
    }
    pub const ALL: [JointKind; 5] = [
        JointKind::Fixed, JointKind::Revolute, JointKind::Prismatic,
        JointKind::Spherical, JointKind::Distance,
    ];
}

#[derive(Clone, Debug)]
pub struct Joint {
    pub id: u64,
    pub name: String,
    pub kind: JointKind,
    pub body_a: ObjectId,
    pub body_b: ObjectId,
    pub anchor_a: [f64; 3],
    pub anchor_b: [f64; 3],
    pub axis: [f64; 3],
    pub lower_limit: f64,
    pub upper_limit: f64,
    pub break_force: f64,
    pub break_torque: f64,
    pub collision_between_bodies: bool,
    pub enabled: bool,
}

impl Joint {
    pub fn new(id: u64, kind: JointKind, body_a: ObjectId, body_b: ObjectId) -> Self {
        Self {
            id,
            name: format!("Joint_{id}"),
            kind,
            body_a,
            body_b,
            anchor_a: [0.0; 3],
            anchor_b: [0.0; 3],
            axis: [0.0, 1.0, 0.0],
            lower_limit: -90.0,
            upper_limit: 90.0,
            break_force: f64::INFINITY,
            break_torque: f64::INFINITY,
            collision_between_bodies: false,
            enabled: true,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Spring {
    pub id: u64,
    pub name: String,
    pub body_a: ObjectId,
    pub body_b: ObjectId,
    pub anchor_a: [f64; 3],
    pub anchor_b: [f64; 3],
    pub rest_length: f64,
    pub stiffness: f64,
    pub damping: f64,
    pub enabled: bool,
}

impl Spring {
    pub fn new(id: u64, body_a: ObjectId, body_b: ObjectId) -> Self {
        Self {
            id,
            name: format!("Spring_{id}"),
            body_a,
            body_b,
            anchor_a: [0.0; 3],
            anchor_b: [0.0; 3],
            rest_length: 1.0,
            stiffness: 100.0,
            damping: 5.0,
            enabled: true,
        }
    }

    pub fn force(&self, pos_a: [f64; 3], pos_b: [f64; 3], vel_a: [f64; 3], vel_b: [f64; 3]) -> [f64; 3] {
        let dx = [pos_b[0] - pos_a[0], pos_b[1] - pos_a[1], pos_b[2] - pos_a[2]];
        let dist = (dx[0]*dx[0] + dx[1]*dx[1] + dx[2]*dx[2]).sqrt().max(1e-9);
        let dir = [dx[0]/dist, dx[1]/dist, dx[2]/dist];
        let spring_f = self.stiffness * (dist - self.rest_length);
        let rel_vel = (vel_b[0]-vel_a[0])*dir[0] + (vel_b[1]-vel_a[1])*dir[1] + (vel_b[2]-vel_a[2])*dir[2];
        let damp_f = self.damping * rel_vel;
        let f = spring_f + damp_f;
        [dir[0]*f, dir[1]*f, dir[2]*f]
    }
}

#[derive(Clone, Debug)]
pub struct RagdollBone {
    pub name: String,
    pub body_id: ObjectId,
    pub joint_id: Option<u64>,
    pub parent_bone: Option<usize>,
    pub collider_radius: f64,
    pub collider_length: f64,
}

#[derive(Clone, Debug, Default)]
pub struct Ragdoll {
    pub bones: Vec<RagdollBone>,
    pub active: bool,
    pub blend_weight: f64,
}

impl Ragdoll {
    pub fn new() -> Self { Self { bones: Vec::new(), active: false, blend_weight: 0.0 } }
    pub fn activate(&mut self) { self.active = true; self.blend_weight = 1.0; }
    pub fn deactivate(&mut self) { self.active = false; self.blend_weight = 0.0; }
}

#[derive(Clone, Debug, Default)]
pub struct ConstraintWorld {
    pub joints: Vec<Joint>,
    pub springs: Vec<Spring>,
    pub next_id: u64,
}

impl ConstraintWorld {
    pub fn new() -> Self { Self::default() }

    pub fn add_joint(&mut self, kind: JointKind, body_a: ObjectId, body_b: ObjectId) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        self.joints.push(Joint::new(id, kind, body_a, body_b));
        id
    }

    pub fn add_spring(&mut self, body_a: ObjectId, body_b: ObjectId) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        self.springs.push(Spring::new(id, body_a, body_b));
        id
    }

    pub fn remove_joint(&mut self, id: u64) { self.joints.retain(|j| j.id != id); }
    pub fn remove_spring(&mut self, id: u64) { self.springs.retain(|s| s.id != id); }
}
