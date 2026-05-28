use crate::core::engine::rendering::raytracing::Vec3;

#[derive(Debug, Clone)]
pub struct IkChain {
    pub joints: Vec<Vec3>,
    pub lengths: Vec<f64>,
    pub constraints: Vec<(f64, f64)>,
}

impl IkChain {
    pub fn new(joints: Vec<Vec3>) -> Self {
        let n = joints.len();
        let lengths: Vec<f64> = (0..n.saturating_sub(1))
            .map(|i| (joints[i + 1] - joints[i]).length())
            .collect();
        let constraints = vec![(0.0_f64, std::f64::consts::PI); lengths.len()];
        Self { joints, lengths, constraints }
    }

    pub fn with_constraint(mut self, segment_idx: usize, min_angle: f64, max_angle: f64) -> Self {
        if segment_idx < self.constraints.len() {
            self.constraints[segment_idx] = (min_angle, max_angle);
        }
        self
    }

    pub fn total_length(&self) -> f64 {
        self.lengths.iter().sum()
    }

    pub fn solve_fabrik(&mut self, target: Vec3, max_iter: u32, tol: f64) {
        let n = self.joints.len();
        if n < 2 {
            return;
        }
        let root = self.joints[0];
        let dist_to_target = (target - root).length();
        if dist_to_target >= self.total_length() {
            for i in 0..n - 1 {
                let dir = {
                    let d = target - self.joints[i];
                    let len = d.length();
                    if len > f64::EPSILON { d * (1.0 / len) } else { Vec3::new(0.0, 1.0, 0.0) }
                };
                self.joints[i + 1] = self.joints[i] + dir * self.lengths[i];
            }
            return;
        }
        for _ in 0..max_iter {
            self.joints[n - 1] = target;
            for i in (0..n - 1).rev() {
                let dir = {
                    let d = self.joints[i] - self.joints[i + 1];
                    let len = d.length();
                    if len > f64::EPSILON { d * (1.0 / len) } else { Vec3::new(0.0, 1.0, 0.0) }
                };
                self.joints[i] = self.joints[i + 1] + dir * self.lengths[i];
            }
            self.joints[0] = root;
            for i in 0..n - 1 {
                let dir = {
                    let d = self.joints[i + 1] - self.joints[i];
                    let len = d.length();
                    if len > f64::EPSILON { d * (1.0 / len) } else { Vec3::new(0.0, 1.0, 0.0) }
                };
                self.joints[i + 1] = self.joints[i] + dir * self.lengths[i];
            }
            let end_dist = (self.joints[n - 1] - target).length();
            if end_dist < tol {
                break;
            }
        }
    }

    pub fn end_effector(&self) -> Vec3 {
        self.joints.last().copied().unwrap_or(Vec3::new(0.0, 0.0, 0.0))
    }

    pub fn joint_count(&self) -> usize {
        self.joints.len()
    }
}
