use super::rigidbody::RigidBody;
use crate::core::engine::rendering::raytracing::Vec3;

#[derive(Debug, Clone)]
pub struct Wheel {
    pub offset: Vec3,
    pub radius: f64,
    pub suspension_rest: f64,
    pub stiffness: f64,
    pub damping: f64,
    pub contact_point: Option<Vec3>,
    pub compression: f64,
    pub angular_velocity: f64,
}

impl Wheel {
    pub fn new(
        offset: Vec3,
        radius: f64,
        suspension_rest: f64,
        stiffness: f64,
        damping: f64,
    ) -> Self {
        Self {
            offset,
            radius,
            suspension_rest,
            stiffness,
            damping,
            contact_point: None,
            compression: 0.0,
            angular_velocity: 0.0,
        }
    }

    pub fn suspension_force(&self) -> f64 {
        let velocity_component = 0.0_f64;
        self.stiffness * self.compression - self.damping * velocity_component
    }
}

pub struct Vehicle {
    pub body: RigidBody,
    pub wheels: Vec<Wheel>,
    pub engine_torque: f64,
    pub brake_torque: f64,
    pub steering_angle: f64,
    pub throttle: f64,
    pub brake: f64,
}

impl Vehicle {
    pub fn new(body: RigidBody, wheels: Vec<Wheel>) -> Self {
        Self {
            body,
            wheels,
            engine_torque: 200.0,
            brake_torque: 500.0,
            steering_angle: 0.0,
            throttle: 0.0,
            brake: 0.0,
        }
    }

    pub fn apply_throttle(&mut self, t: f64) {
        self.throttle = t.clamp(0.0, 1.0);
    }

    pub fn apply_brake(&mut self, b: f64) {
        self.brake = b.clamp(0.0, 1.0);
    }

    pub fn set_steering(&mut self, angle: f64) {
        let max_steer = std::f64::consts::PI / 6.0;
        self.steering_angle = angle.clamp(-max_steer, max_steer);
    }

    pub fn step(&mut self, dt: f64, gravity: Vec3) {
        if self.body.is_static {
            return;
        }

        let ground_y = 0.0_f64;
        let total_suspension_force = {
            let mut total = 0.0_f64;
            for wheel in &mut self.wheels {
                let wheel_world_y = self.body.position.y + wheel.offset.y - wheel.radius;
                let ground_dist = wheel_world_y - ground_y;
                if ground_dist < wheel.suspension_rest {
                    wheel.compression = (wheel.suspension_rest - ground_dist).max(0.0);
                    let f = wheel.stiffness * wheel.compression;
                    total += f;
                    wheel.contact_point = Some(Vec3::new(
                        self.body.position.x + wheel.offset.x,
                        ground_y,
                        self.body.position.z + wheel.offset.z,
                    ));
                } else {
                    wheel.compression = 0.0;
                    wheel.contact_point = None;
                }
            }
            total
        };

        let gravity_force = gravity * self.body.mass;
        let suspension_vec = Vec3::new(0.0, total_suspension_force, 0.0);

        let drive_wheels: usize = self
            .wheels
            .iter()
            .filter(|w| w.contact_point.is_some())
            .count();
        let drive_force = if drive_wheels > 0 {
            let torque = self.engine_torque * self.throttle;
            let wheel_radius = self.wheels.first().map(|w| w.radius).unwrap_or(0.3);
            let force_per_wheel = torque / wheel_radius / drive_wheels as f64;
            let heading = Vec3::new(self.steering_angle.sin(), 0.0, self.steering_angle.cos());
            heading * force_per_wheel
        } else {
            Vec3::ZERO
        };

        let brake_force = if self.brake > 0.0 {
            let speed = self.body.velocity.length();
            let brake_mag = self.brake_torque * self.brake;
            if speed > 1e-4 {
                self.body.velocity * -(brake_mag / speed).min(speed)
            } else {
                Vec3::ZERO
            }
        } else {
            Vec3::ZERO
        };

        let total_force = gravity_force + suspension_vec + drive_force;
        let accel = total_force * self.body.inv_mass + brake_force;

        self.body.velocity += accel * dt;
        self.body.position += self.body.velocity * dt;

        let damping = 0.98_f64;
        self.body.velocity = self.body.velocity * damping;

        for wheel in &mut self.wheels {
            if wheel.contact_point.is_some() {
                let wheel_speed = self.body.velocity.length() / wheel.radius.max(1e-6);
                wheel.angular_velocity = wheel_speed;
            } else {
                wheel.angular_velocity *= 0.95;
            }
        }
    }

    pub fn wheel_count(&self) -> usize {
        self.wheels.len()
    }

    pub fn speed(&self) -> f64 {
        self.body.velocity.length()
    }
}
