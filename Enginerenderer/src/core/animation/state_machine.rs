use crate::core::engine::rendering::mesh::skinning::{Mat4, Skeleton};

#[derive(Debug, Clone)]
pub struct SkeletalClip {
    pub name: String,
    pub duration: f64,
    pub bone_tracks: Vec<Vec<(f64, Mat4)>>,
}

impl SkeletalClip {
    pub fn new(name: impl Into<String>, duration: f64, bone_count: usize) -> Self {
        Self {
            name: name.into(),
            duration,
            bone_tracks: vec![Vec::new(); bone_count],
        }
    }

    pub fn add_keyframe(&mut self, bone_idx: usize, time: f64, pose: Mat4) {
        if bone_idx < self.bone_tracks.len() {
            self.bone_tracks[bone_idx].push((time, pose));
            self.bone_tracks[bone_idx].sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
        }
    }

    pub fn sample_bone(&self, bone_idx: usize, time: f64) -> Mat4 {
        let track = &self.bone_tracks[bone_idx];
        if track.is_empty() {
            return Mat4::identity();
        }
        if time <= track[0].0 {
            return track[0].1;
        }
        if time >= track[track.len() - 1].0 {
            return track[track.len() - 1].1;
        }
        for i in 0..track.len() - 1 {
            if time >= track[i].0 && time <= track[i + 1].0 {
                let span = track[i + 1].0 - track[i].0;
                let t = if span > f64::EPSILON { (time - track[i].0) / span } else { 0.0 };
                return lerp_mat4(track[i].1, track[i + 1].1, t);
            }
        }
        track[track.len() - 1].1
    }

    pub fn sample_all_bones(&self, time: f64) -> Vec<Mat4> {
        (0..self.bone_tracks.len())
            .map(|i| self.sample_bone(i, time))
            .collect()
    }
}

fn lerp_mat4(a: Mat4, b: Mat4, t: f64) -> Mat4 {
    let mut m = Mat4::identity();
    for col in 0..4 {
        for row in 0..4 {
            m.cols[col][row] = a.cols[col][row] + (b.cols[col][row] - a.cols[col][row]) * t;
        }
    }
    m
}

pub fn blend_poses(a: &[Mat4], b: &[Mat4], t: f64) -> Vec<Mat4> {
    a.iter().zip(b.iter()).map(|(&ma, &mb)| lerp_mat4(ma, mb, t)).collect()
}

#[derive(Debug, Clone)]
pub struct BlendTree {
    pub clips: Vec<usize>,
    pub weights: Vec<f64>,
}

impl BlendTree {
    pub fn single(clip_idx: usize) -> Self {
        Self { clips: vec![clip_idx], weights: vec![1.0] }
    }

    pub fn blend2(clip_a: usize, clip_b: usize, t: f64) -> Self {
        Self { clips: vec![clip_a, clip_b], weights: vec![1.0 - t, t] }
    }

    pub fn evaluate(&self, clips: &[SkeletalClip], time: f64) -> Vec<Mat4> {
        if self.clips.is_empty() {
            return Vec::new();
        }
        let first_idx = self.clips[0];
        if first_idx >= clips.len() {
            return Vec::new();
        }
        let bone_count = clips[first_idx].bone_tracks.len();
        let mut result: Vec<Mat4> = vec![Mat4 { cols: [[0.0; 4]; 4] }; bone_count];
        let total_weight: f64 = self.weights.iter().sum();
        let norm = if total_weight > f64::EPSILON { 1.0 / total_weight } else { 1.0 };
        for (i, &clip_idx) in self.clips.iter().enumerate() {
            if clip_idx >= clips.len() {
                continue;
            }
            let w = self.weights[i] * norm;
            let pose = clips[clip_idx].sample_all_bones(time);
            for (r, p) in result.iter_mut().zip(pose.iter()) {
                for col in 0..4 {
                    for row in 0..4 {
                        r.cols[col][row] += p.cols[col][row] * w;
                    }
                }
            }
        }
        result
    }
}

#[derive(Debug, Clone)]
pub struct AnimTransition {
    pub target: String,
    pub blend_time: f64,
    pub condition: f64,
}

#[derive(Debug, Clone)]
pub struct AnimState {
    pub name: String,
    pub blend_tree: BlendTree,
    pub transitions: Vec<AnimTransition>,
    pub playback_speed: f64,
}

impl AnimState {
    pub fn new(name: impl Into<String>, blend_tree: BlendTree) -> Self {
        Self {
            name: name.into(),
            blend_tree,
            transitions: Vec::new(),
            playback_speed: 1.0,
        }
    }

    pub fn with_transition(mut self, transition: AnimTransition) -> Self {
        self.transitions.push(transition);
        self
    }
}

#[derive(Debug, Clone)]
pub struct AnimStateMachine {
    pub states: Vec<AnimState>,
    pub clips: Vec<SkeletalClip>,
    pub current_state: usize,
    pub parameter: f64,
    pub local_time: f64,
    pub blend_time: f64,
    pub blend_accum: f64,
    pub transitioning_to: Option<usize>,
}

impl AnimStateMachine {
    pub fn new(clips: Vec<SkeletalClip>) -> Self {
        Self {
            states: Vec::new(),
            clips,
            current_state: 0,
            parameter: 0.0,
            local_time: 0.0,
            blend_time: 0.0,
            blend_accum: 0.0,
            transitioning_to: None,
        }
    }

    pub fn add_state(&mut self, state: AnimState) -> usize {
        let idx = self.states.len();
        self.states.push(state);
        idx
    }

    pub fn set_parameter(&mut self, value: f64) {
        self.parameter = value;
    }

    pub fn tick(&mut self, dt: f64) -> Vec<Mat4> {
        if self.states.is_empty() {
            return Vec::new();
        }

        let speed = self.states[self.current_state].playback_speed;
        self.local_time += dt * speed;

        if self.transitioning_to.is_none() {
            let current_name = self.states[self.current_state].name.clone();
            let param = self.parameter;
            let transitions = self.states[self.current_state].transitions.clone();
            for trans in &transitions {
                if param >= trans.condition {
                    let target_idx = self.states.iter().position(|s| s.name == trans.target);
                    if let Some(idx) = target_idx {
                        if self.states[idx].name != current_name {
                            self.transitioning_to = Some(idx);
                            self.blend_time = trans.blend_time;
                            self.blend_accum = 0.0;
                        }
                    }
                }
            }
        }

        if let Some(next_idx) = self.transitioning_to {
            self.blend_accum += dt;
            let t = if self.blend_time > f64::EPSILON {
                (self.blend_accum / self.blend_time).clamp(0.0, 1.0)
            } else {
                1.0
            };
            let pose_a = self.states[self.current_state]
                .blend_tree
                .evaluate(&self.clips, self.local_time);
            let pose_b = self.states[next_idx]
                .blend_tree
                .evaluate(&self.clips, 0.0);
            if t >= 1.0 {
                self.current_state = next_idx;
                self.local_time = self.blend_accum;
                self.transitioning_to = None;
                self.blend_accum = 0.0;
            }
            if pose_a.is_empty() {
                return pose_b;
            }
            blend_poses(&pose_a, &pose_b, t)
        } else {
            self.states[self.current_state]
                .blend_tree
                .evaluate(&self.clips, self.local_time)
        }
    }

    pub fn apply_to_skeleton(&self, skeleton: &mut Skeleton, pose: &[Mat4]) {
        for (i, mat) in pose.iter().enumerate() {
            if i < skeleton.bones.len() {
                skeleton.bones[i].local_transform = *mat;
            }
        }
        skeleton.update_transforms();
    }
}
