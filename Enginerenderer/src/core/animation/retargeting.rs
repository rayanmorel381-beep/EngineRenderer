use crate::core::engine::rendering::mesh::skinning::Mat4;
use super::root_motion::{quat_mul, quat_normalize, quat_slerp};

#[derive(Debug, Clone)]
pub struct BoneMapping {
    pub source_bone: usize,
    pub target_bone: usize,
    pub rotation_offset: [f64; 4],
}

impl BoneMapping {
    pub fn new(source_bone: usize, target_bone: usize) -> Self {
        Self { source_bone, target_bone, rotation_offset: [0.0, 0.0, 0.0, 1.0] }
    }

    pub fn with_offset(mut self, rotation_offset: [f64; 4]) -> Self {
        self.rotation_offset = quat_normalize(rotation_offset);
        self
    }
}

pub struct RetargetConfig {
    pub mappings: Vec<BoneMapping>,
    pub scale_factor: f64,
}

impl RetargetConfig {
    pub fn new(mappings: Vec<BoneMapping>, scale_factor: f64) -> Self {
        Self { mappings, scale_factor }
    }

    pub fn remap_pose(&self, source_pose: &[Mat4], target_bone_count: usize) -> Vec<Mat4> {
        let mut result = vec![Mat4::identity(); target_bone_count];
        for mapping in &self.mappings {
            if mapping.source_bone >= source_pose.len() { continue; }
            if mapping.target_bone >= target_bone_count { continue; }
            let src = source_pose[mapping.source_bone];
            let rotated = apply_rotation_offset(src, mapping.rotation_offset);
            let mut scaled = rotated;
            scaled.cols[3][0] *= self.scale_factor;
            scaled.cols[3][1] *= self.scale_factor;
            scaled.cols[3][2] *= self.scale_factor;
            result[mapping.target_bone] = scaled;
        }
        result
    }

    pub fn remap_pose_blend(&self, source_pose: &[Mat4], target_pose: &[Mat4], blend_alpha: f64) -> Vec<Mat4> {
        let retargeted = self.remap_pose(source_pose, target_pose.len());
        retargeted.into_iter().zip(target_pose.iter()).map(|(retarget, base)| {
            blend_mat4(*base, retarget, blend_alpha)
        }).collect()
    }
}

fn apply_rotation_offset(m: Mat4, quat: [f64; 4]) -> Mat4 {
    let src_quat = mat4_to_quat(&m);
    let result_quat = quat_mul(src_quat, quat);
    let result_quat = quat_normalize(result_quat);
    let mut out = quat_to_mat4(result_quat);
    out.cols[3] = m.cols[3];
    out
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
    } else {
        [0.0, 0.0, 0.0, 1.0]
    }
}

fn quat_to_mat4(q: [f64; 4]) -> Mat4 {
    let [x, y, z, w] = q;
    let mut m = Mat4::identity();
    m.cols[0][0] = 1.0 - 2.0*(y*y + z*z);
    m.cols[0][1] = 2.0*(x*y + z*w);
    m.cols[0][2] = 2.0*(x*z - y*w);
    m.cols[1][0] = 2.0*(x*y - z*w);
    m.cols[1][1] = 1.0 - 2.0*(x*x + z*z);
    m.cols[1][2] = 2.0*(y*z + x*w);
    m.cols[2][0] = 2.0*(x*z + y*w);
    m.cols[2][1] = 2.0*(y*z - x*w);
    m.cols[2][2] = 1.0 - 2.0*(x*x + y*y);
    m
}

fn blend_mat4(a: Mat4, b: Mat4, t: f64) -> Mat4 {
    let qa = mat4_to_quat(&a);
    let qb = mat4_to_quat(&b);
    let qr = quat_slerp(qa, qb, t);
    let mut out = quat_to_mat4(qr);
    for i in 0..4 {
        out.cols[3][i] = a.cols[3][i] + (b.cols[3][i] - a.cols[3][i]) * t;
    }
    out
}
