use crate::core::engine::rendering::mesh::skinning::Skeleton;
use crate::core::engine::rendering::raytracing::Vec3;

#[derive(Debug, Clone)]
pub struct SpringBone {
    pub rest_position: Vec3,
    pub position: Vec3,
    pub velocity: Vec3,
    pub stiffness: f64,
    pub damping: f64,
    pub mass: f64,
}

impl SpringBone {
    pub fn new(rest_position: Vec3, stiffness: f64, damping: f64, mass: f64) -> Self {
        Self {
            rest_position,
            position: rest_position,
            velocity: Vec3::ZERO,
            stiffness,
            damping,
            mass,
        }
    }

    pub fn update(&mut self, parent_world_pos: Vec3, dt: f64) {
        let target = parent_world_pos + self.rest_position;
        let spring_force = (target - self.position) * self.stiffness;
        let damping_force = self.velocity * (-self.damping);
        let total_force = spring_force + damping_force;
        let accel = total_force * (1.0 / self.mass.max(f64::EPSILON));
        self.velocity += accel * dt;
        self.position += self.velocity * dt;
    }

    pub fn displacement(&self) -> Vec3 {
        self.position - self.rest_position
    }
}

#[derive(Debug, Clone)]
pub struct JiggleBone {
    pub spring: SpringBone,
    pub max_angle_rad: f64,
}

impl JiggleBone {
    pub fn new(
        rest_position: Vec3,
        stiffness: f64,
        damping: f64,
        mass: f64,
        max_angle_rad: f64,
    ) -> Self {
        Self {
            spring: SpringBone::new(rest_position, stiffness, damping, mass),
            max_angle_rad,
        }
    }

    pub fn update(&mut self, parent_world_pos: Vec3, dt: f64) {
        self.spring.update(parent_world_pos, dt);
        let disp = self.spring.displacement();
        let len = disp.length();
        if len > f64::EPSILON {
            let max_disp =
                self.max_angle_rad.tan() * self.spring.rest_position.length().max(f64::EPSILON);
            if len > max_disp {
                let clamped = disp * (max_disp / len);
                self.spring.position = self.spring.rest_position + clamped;
                self.spring.velocity = self.spring.velocity * 0.5;
            }
        }
    }

    pub fn apply_to_bone_position(&self, bone_idx: usize, skeleton: &mut Skeleton) {
        if bone_idx < skeleton.bones.len() {
            let disp = self.spring.displacement();
            skeleton.bones[bone_idx].local_transform.cols[3][0] += disp.x;
            skeleton.bones[bone_idx].local_transform.cols[3][1] += disp.y;
            skeleton.bones[bone_idx].local_transform.cols[3][2] += disp.z;
        }
    }
}

pub struct SecondaryMotionSystem {
    pub bones: Vec<(usize, JiggleBone)>,
}

impl SecondaryMotionSystem {
    pub fn new() -> Self {
        Self { bones: Vec::new() }
    }

    pub fn add_jiggle_bone(&mut self, bone_idx: usize, jiggle: JiggleBone) {
        self.bones.push((bone_idx, jiggle));
    }

    pub fn update(&mut self, skeleton: &mut Skeleton, dt: f64) {
        let world_transforms: Vec<_> = skeleton.world_transforms.clone();
        for (bone_idx, jiggle) in &mut self.bones {
            let parent_idx = skeleton.bones[*bone_idx].parent;
            let parent_pos = match parent_idx {
                Some(pi) => Vec3::new(
                    world_transforms[pi].cols[3][0],
                    world_transforms[pi].cols[3][1],
                    world_transforms[pi].cols[3][2],
                ),
                None => Vec3::ZERO,
            };
            jiggle.update(parent_pos, dt);
            jiggle.apply_to_bone_position(*bone_idx, skeleton);
        }
        skeleton.update_transforms();
    }

    pub fn bone_count(&self) -> usize {
        self.bones.len()
    }
}
