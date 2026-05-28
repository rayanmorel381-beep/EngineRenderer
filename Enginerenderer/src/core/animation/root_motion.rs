use super::state_machine::SkeletalClip;
use crate::core::engine::rendering::mesh::skinning::Mat4;
use crate::core::engine::rendering::raytracing::Vec3;

#[derive(Debug, Clone, Copy)]
pub struct RootMotionDelta {
    pub translation: Vec3,
    pub rotation: [f64; 4],
}

impl RootMotionDelta {
    pub fn identity() -> Self {
        Self {
            translation: Vec3::ZERO,
            rotation: [0.0, 0.0, 0.0, 1.0],
        }
    }
}

pub fn extract_root_motion(
    clip: &SkeletalClip,
    from_time: f64,
    to_time: f64,
    root_bone: usize,
) -> RootMotionDelta {
    let pose_from = clip.sample_bone(root_bone, from_time);
    let pose_to = clip.sample_bone(root_bone, to_time);

    let trans_from = Vec3::new(
        pose_from.cols[3][0],
        pose_from.cols[3][1],
        pose_from.cols[3][2],
    );
    let trans_to = Vec3::new(pose_to.cols[3][0], pose_to.cols[3][1], pose_to.cols[3][2]);
    let delta_translation = trans_to - trans_from;

    let quat_from = mat4_to_quat(&pose_from);
    let quat_to = mat4_to_quat(&pose_to);
    let delta_rotation = quat_relative(quat_from, quat_to);

    RootMotionDelta {
        translation: delta_translation,
        rotation: delta_rotation,
    }
}

pub fn apply_root_motion(delta: &RootMotionDelta, body_pos: &mut Vec3, body_rot: &mut [f64; 4]) {
    *body_pos += delta.translation;
    *body_rot = quat_mul(*body_rot, delta.rotation);
    *body_rot = quat_normalize(*body_rot);
}

fn mat4_to_quat(m: &Mat4) -> [f64; 4] {
    let trace = m.cols[0][0] + m.cols[1][1] + m.cols[2][2];
    if trace > 0.0 {
        let s = 0.5 / (trace + 1.0).sqrt();
        [
            (m.cols[1][2] - m.cols[2][1]) * s,
            (m.cols[2][0] - m.cols[0][2]) * s,
            (m.cols[0][1] - m.cols[1][0]) * s,
            0.25 / s,
        ]
    } else if m.cols[0][0] > m.cols[1][1] && m.cols[0][0] > m.cols[2][2] {
        let s = 2.0 * (1.0 + m.cols[0][0] - m.cols[1][1] - m.cols[2][2]).sqrt();
        [
            0.25 * s,
            (m.cols[0][1] + m.cols[1][0]) / s,
            (m.cols[2][0] + m.cols[0][2]) / s,
            (m.cols[1][2] - m.cols[2][1]) / s,
        ]
    } else if m.cols[1][1] > m.cols[2][2] {
        let s = 2.0 * (1.0 + m.cols[1][1] - m.cols[0][0] - m.cols[2][2]).sqrt();
        [
            (m.cols[0][1] + m.cols[1][0]) / s,
            0.25 * s,
            (m.cols[1][2] + m.cols[2][1]) / s,
            (m.cols[2][0] - m.cols[0][2]) / s,
        ]
    } else {
        let s = 2.0 * (1.0 + m.cols[2][2] - m.cols[0][0] - m.cols[1][1]).sqrt();
        [
            (m.cols[2][0] + m.cols[0][2]) / s,
            (m.cols[1][2] + m.cols[2][1]) / s,
            0.25 * s,
            (m.cols[0][1] - m.cols[1][0]) / s,
        ]
    }
}

pub fn quat_mul(a: [f64; 4], b: [f64; 4]) -> [f64; 4] {
    [
        a[3] * b[0] + a[0] * b[3] + a[1] * b[2] - a[2] * b[1],
        a[3] * b[1] - a[0] * b[2] + a[1] * b[3] + a[2] * b[0],
        a[3] * b[2] + a[0] * b[1] - a[1] * b[0] + a[2] * b[3],
        a[3] * b[3] - a[0] * b[0] - a[1] * b[1] - a[2] * b[2],
    ]
}

pub fn quat_normalize(q: [f64; 4]) -> [f64; 4] {
    let len = (q[0] * q[0] + q[1] * q[1] + q[2] * q[2] + q[3] * q[3]).sqrt();
    if len < f64::EPSILON {
        return [0.0, 0.0, 0.0, 1.0];
    }
    [q[0] / len, q[1] / len, q[2] / len, q[3] / len]
}

pub fn quat_slerp(a: [f64; 4], b: [f64; 4], t: f64) -> [f64; 4] {
    let mut dot = a[0] * b[0] + a[1] * b[1] + a[2] * b[2] + a[3] * b[3];
    let b_adj = if dot < 0.0 {
        dot = -dot;
        [-b[0], -b[1], -b[2], -b[3]]
    } else {
        b
    };
    if dot > 0.9995 {
        return quat_normalize([
            a[0] + (b_adj[0] - a[0]) * t,
            a[1] + (b_adj[1] - a[1]) * t,
            a[2] + (b_adj[2] - a[2]) * t,
            a[3] + (b_adj[3] - a[3]) * t,
        ]);
    }
    let theta = dot.acos();
    let sin_theta = theta.sin();
    let wa = ((1.0 - t) * theta).sin() / sin_theta;
    let wb = (t * theta).sin() / sin_theta;
    quat_normalize([
        wa * a[0] + wb * b_adj[0],
        wa * a[1] + wb * b_adj[1],
        wa * a[2] + wb * b_adj[2],
        wa * a[3] + wb * b_adj[3],
    ])
}

fn quat_conjugate(q: [f64; 4]) -> [f64; 4] {
    [-q[0], -q[1], -q[2], q[3]]
}

fn quat_relative(from: [f64; 4], to: [f64; 4]) -> [f64; 4] {
    quat_mul(quat_conjugate(from), to)
}
