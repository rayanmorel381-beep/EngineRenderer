use crate::core::engine::rendering::raytracing::Vec3;

pub const MAX_BONES_PER_VERTEX: usize = 4;
pub const MAX_SKELETON_BONES: usize = 256;

#[derive(Debug, Clone, Copy, Default)]
pub struct BoneWeight {
    pub indices: [u8; MAX_BONES_PER_VERTEX],
    pub weights: [f32; MAX_BONES_PER_VERTEX],
}

impl BoneWeight {
    pub fn normalize(&mut self) {
        let sum: f32 = self.weights.iter().sum();
        if sum > 1e-6 {
            for w in &mut self.weights { *w /= sum; }
        }
    }

    pub fn single(bone: u8) -> Self {
        Self {
            indices: [bone, 0, 0, 0],
            weights: [1.0, 0.0, 0.0, 0.0],
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Mat4 {
    pub cols: [[f64; 4]; 4],
}

impl Default for Mat4 {
    fn default() -> Self { Self::identity() }
}

impl Mat4 {
    pub fn identity() -> Self {
        let mut m = Self { cols: [[0.0; 4]; 4] };
        m.cols[0][0] = 1.0;
        m.cols[1][1] = 1.0;
        m.cols[2][2] = 1.0;
        m.cols[3][3] = 1.0;
        m
    }

    pub fn from_trs(translation: Vec3, rotation_quat: [f64; 4], scale: Vec3) -> Self {
        let [qx, qy, qz, qw] = rotation_quat;
        let x2 = qx * qx; let y2 = qy * qy; let z2 = qz * qz;
        let xy = qx * qy; let xz = qx * qz; let yz = qy * qz;
        let wx = qw * qx; let wy = qw * qy; let wz = qw * qz;
        let mut m = Self { cols: [[0.0; 4]; 4] };
        m.cols[0][0] = (1.0 - 2.0*(y2+z2)) * scale.x;
        m.cols[0][1] = (2.0*(xy+wz)) * scale.x;
        m.cols[0][2] = (2.0*(xz-wy)) * scale.x;
        m.cols[1][0] = (2.0*(xy-wz)) * scale.y;
        m.cols[1][1] = (1.0 - 2.0*(x2+z2)) * scale.y;
        m.cols[1][2] = (2.0*(yz+wx)) * scale.y;
        m.cols[2][0] = (2.0*(xz+wy)) * scale.z;
        m.cols[2][1] = (2.0*(yz-wx)) * scale.z;
        m.cols[2][2] = (1.0 - 2.0*(x2+y2)) * scale.z;
        m.cols[3][0] = translation.x;
        m.cols[3][1] = translation.y;
        m.cols[3][2] = translation.z;
        m.cols[3][3] = 1.0;
        m
    }

    pub fn transform_point(&self, p: Vec3) -> Vec3 {
        let c = &self.cols;
        Vec3::new(
            c[0][0]*p.x + c[1][0]*p.y + c[2][0]*p.z + c[3][0],
            c[0][1]*p.x + c[1][1]*p.y + c[2][1]*p.z + c[3][1],
            c[0][2]*p.x + c[1][2]*p.y + c[2][2]*p.z + c[3][2],
        )
    }

    pub fn transform_direction(&self, d: Vec3) -> Vec3 {
        let c = &self.cols;
        Vec3::new(
            c[0][0]*d.x + c[1][0]*d.y + c[2][0]*d.z,
            c[0][1]*d.x + c[1][1]*d.y + c[2][1]*d.z,
            c[0][2]*d.x + c[1][2]*d.y + c[2][2]*d.z,
        )
    }

    pub fn mul(&self, other: &Mat4) -> Mat4 {
        let mut result = Mat4 { cols: [[0.0; 4]; 4] };
        for i in 0..4 {
            for j in 0..4 {
                for k in 0..4 {
                    result.cols[i][j] += self.cols[k][j] * other.cols[i][k];
                }
            }
        }
        result
    }

    pub fn scale_mat4(s: f64) -> Self {
        let mut m = Self::identity();
        m.cols[0][0] = s;
        m.cols[1][1] = s;
        m.cols[2][2] = s;
        m
    }

    pub fn add_scaled(&self, other: &Mat4, scale: f64) -> Mat4 {
        let mut result = Mat4 { cols: [[0.0; 4]; 4] };
        for i in 0..4 {
            for j in 0..4 {
                result.cols[i][j] = self.cols[i][j] + other.cols[i][j] * scale;
            }
        }
        result
    }
}

#[derive(Debug, Clone)]
pub struct Bone {
    pub name: String,
    pub parent: Option<usize>,
    pub inverse_bind_pose: Mat4,
    pub local_transform: Mat4,
}

#[derive(Debug, Clone)]
pub struct Skeleton {
    pub bones: Vec<Bone>,
    pub world_transforms: Vec<Mat4>,
    pub skinning_matrices: Vec<Mat4>,
}

impl Skeleton {
    pub fn new(bones: Vec<Bone>) -> Self {
        let n = bones.len();
        Self {
            world_transforms: vec![Mat4::identity(); n],
            skinning_matrices: vec![Mat4::identity(); n],
            bones,
        }
    }

    pub fn update_transforms(&mut self) {
        let n = self.bones.len();
        for i in 0..n {
            let local = self.bones[i].local_transform;
            self.world_transforms[i] = match self.bones[i].parent {
                None => local,
                Some(parent_idx) => self.world_transforms[parent_idx].mul(&local),
            };
        }
        for i in 0..n {
            self.skinning_matrices[i] = self.world_transforms[i].mul(&self.bones[i].inverse_bind_pose);
        }
    }

    pub fn bone_count(&self) -> usize { self.bones.len() }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct SkinnedVertex {
    pub bind_position: Vec3,
    pub bind_normal: Vec3,
    pub bind_tangent: Vec3,
    pub bone_weights: BoneWeight,
}

#[derive(Debug, Clone)]
pub struct BlendShape {
    pub name: String,
    pub weight: f64,
    pub delta_positions: Vec<Vec3>,
    pub delta_normals: Vec<Vec3>,
}

#[derive(Debug, Clone)]
pub struct SkinnedMesh {
    pub bind_vertices: Vec<SkinnedVertex>,
    pub blend_shapes: Vec<BlendShape>,
    pub posed_positions: Vec<Vec3>,
    pub posed_normals: Vec<Vec3>,
    pub posed_tangents: Vec<Vec3>,
}

impl SkinnedMesh {
    pub fn new(bind_vertices: Vec<SkinnedVertex>) -> Self {
        let n = bind_vertices.len();
        Self {
            posed_positions: vec![Vec3::ZERO; n],
            posed_normals: vec![Vec3::ZERO; n],
            posed_tangents: vec![Vec3::ZERO; n],
            blend_shapes: Vec::new(),
            bind_vertices,
        }
    }

    pub fn add_blend_shape(&mut self, shape: BlendShape) {
        self.blend_shapes.push(shape);
    }

    pub fn skin(&mut self, skeleton: &Skeleton) {
        for (i, vert) in self.bind_vertices.iter().enumerate() {
            let mut blended_pos = vert.bind_position;
            let mut blended_normal = vert.bind_normal;
            let blended_tangent = vert.bind_tangent;

            for shape in &self.blend_shapes {
                if shape.weight.abs() < 1e-6 { continue; }
                if let Some(dp) = shape.delta_positions.get(i) {
                    blended_pos += *dp * shape.weight;
                }
                if let Some(dn) = shape.delta_normals.get(i) {
                    blended_normal += *dn * shape.weight;
                }
            }
            blended_normal = blended_normal.normalize();

            let bw = &vert.bone_weights;
            let mut out_pos = Vec3::ZERO;
            let mut out_norm = Vec3::ZERO;
            let mut out_tang = Vec3::ZERO;

            for k in 0..MAX_BONES_PER_VERTEX {
                let w = bw.weights[k] as f64;
                if w < 1e-6 { continue; }
                let bone_idx = bw.indices[k] as usize;
                if bone_idx >= skeleton.skinning_matrices.len() { continue; }
                let mat = &skeleton.skinning_matrices[bone_idx];
                out_pos += mat.transform_point(blended_pos) * w;
                out_norm += mat.transform_direction(blended_normal) * w;
                out_tang += mat.transform_direction(blended_tangent) * w;
            }

            self.posed_positions[i] = out_pos;
            self.posed_normals[i] = out_norm.normalize();
            self.posed_tangents[i] = out_tang.normalize();
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct BoneKeyframe {
    pub time: f64,
    pub translation: Vec3,
    pub rotation: [f64; 4],
    pub scale: Vec3,
}

#[derive(Debug, Clone)]
pub struct BoneTrack {
    pub bone_index: usize,
    pub keyframes: Vec<BoneKeyframe>,
}

impl BoneTrack {
    pub fn sample(&self, time: f64) -> Mat4 {
        if self.keyframes.is_empty() { return Mat4::identity(); }
        if self.keyframes.len() == 1 || time <= self.keyframes[0].time {
            let k = &self.keyframes[0];
            return Mat4::from_trs(k.translation, k.rotation, k.scale);
        }
        let last = &self.keyframes[self.keyframes.len() - 1];
        if time >= last.time {
            return Mat4::from_trs(last.translation, last.rotation, last.scale);
        }
        let next = self.keyframes.partition_point(|k| k.time <= time);
        let k0 = &self.keyframes[next - 1];
        let k1 = &self.keyframes[next];
        let t = (time - k0.time) / (k1.time - k0.time);
        let trans = lerp_vec3(k0.translation, k1.translation, t);
        let rot = slerp_quat(k0.rotation, k1.rotation, t);
        let scale = lerp_vec3(k0.scale, k1.scale, t);
        Mat4::from_trs(trans, rot, scale)
    }
}

#[derive(Debug, Clone)]
pub struct AnimationClip {
    pub name: String,
    pub duration: f64,
    pub tracks: Vec<BoneTrack>,
}

impl AnimationClip {
    pub fn apply(&self, skeleton: &mut Skeleton, time: f64) {
        let looped_time = time % self.duration.max(1e-6);
        for track in &self.tracks {
            if track.bone_index < skeleton.bones.len() {
                skeleton.bones[track.bone_index].local_transform = track.sample(looped_time);
            }
        }
        skeleton.update_transforms();
    }
}

fn lerp_vec3(a: Vec3, b: Vec3, t: f64) -> Vec3 {
    a * (1.0 - t) + b * t
}

fn slerp_quat(a: [f64; 4], b: [f64; 4], t: f64) -> [f64; 4] {
    let mut dot = a[0]*b[0] + a[1]*b[1] + a[2]*b[2] + a[3]*b[3];
    let mut b = b;
    if dot < 0.0 {
        b = [-b[0], -b[1], -b[2], -b[3]];
        dot = -dot;
    }
    if dot > 0.9995 {
        let r = [a[0]+(b[0]-a[0])*t, a[1]+(b[1]-a[1])*t, a[2]+(b[2]-a[2])*t, a[3]+(b[3]-a[3])*t];
        let len = (r[0]*r[0]+r[1]*r[1]+r[2]*r[2]+r[3]*r[3]).sqrt().max(1e-9);
        return [r[0]/len, r[1]/len, r[2]/len, r[3]/len];
    }
    let theta_0 = dot.acos();
    let theta = theta_0 * t;
    let (sin_theta, sin_theta_0) = (theta.sin(), theta_0.sin());
    let s0 = (theta_0 - theta).cos() - dot * sin_theta / sin_theta_0;
    let s1 = sin_theta / sin_theta_0;
    [a[0]*s0+b[0]*s1, a[1]*s0+b[1]*s1, a[2]*s0+b[2]*s1, a[3]*s0+b[3]*s1]
}
