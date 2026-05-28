#[derive(Clone, Debug)]
pub struct Bone {
    pub name: String,
    pub parent: Option<usize>,
    pub bind_position: [f64; 3],
    pub bind_rotation: [f64; 4],
    pub bind_scale: [f64; 3],
    pub local_position: [f64; 3],
    pub local_rotation: [f64; 4],
    pub local_scale: [f64; 3],
}

impl Bone {
    pub fn new(name: impl Into<String>, parent: Option<usize>) -> Self {
        Self {
            name: name.into(),
            parent,
            bind_position: [0.0, 0.0, 0.0],
            bind_rotation: [0.0, 0.0, 0.0, 1.0],
            bind_scale: [1.0, 1.0, 1.0],
            local_position: [0.0, 0.0, 0.0],
            local_rotation: [0.0, 0.0, 0.0, 1.0],
            local_scale: [1.0, 1.0, 1.0],
        }
    }
}

#[derive(Clone, Debug)]
pub struct BoneKeyframe {
    pub time: f64,
    pub position: [f64; 3],
    pub rotation: [f64; 4],
    pub scale: [f64; 3],
}

#[derive(Clone, Debug)]
pub struct BoneTrack {
    pub bone_index: usize,
    pub keyframes: Vec<BoneKeyframe>,
}

impl BoneTrack {
    pub fn new(bone_index: usize) -> Self {
        Self { bone_index, keyframes: Vec::new() }
    }

    pub fn sample(&self, time: f64) -> Option<BoneKeyframe> {
        if self.keyframes.is_empty() { return None; }
        let last = self.keyframes.last().unwrap();
        if time >= last.time { return Some(last.clone()); }
        let first = &self.keyframes[0];
        if time <= first.time { return Some(first.clone()); }
        let i = self.keyframes.partition_point(|k| k.time <= time).saturating_sub(1);
        let a = &self.keyframes[i];
        let b = &self.keyframes[i + 1];
        let t = if (b.time - a.time).abs() < 1e-10 { 0.0 } else { (time - a.time) / (b.time - a.time) };
        Some(BoneKeyframe {
            time,
            position: lerp3(a.position, b.position, t),
            rotation: slerp(a.rotation, b.rotation, t),
            scale: lerp3(a.scale, b.scale, t),
        })
    }
}

fn lerp3(a: [f64; 3], b: [f64; 3], t: f64) -> [f64; 3] {
    [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t, a[2] + (b[2] - a[2]) * t]
}

fn slerp(a: [f64; 4], b: [f64; 4], t: f64) -> [f64; 4] {
    let dot = a[0]*b[0] + a[1]*b[1] + a[2]*b[2] + a[3]*b[3];
    let (b, dot) = if dot < 0.0 { ([-b[0], -b[1], -b[2], -b[3]], -dot) } else { (b, dot) };
    if dot > 0.9995 {
        let r = [a[0]+(b[0]-a[0])*t, a[1]+(b[1]-a[1])*t, a[2]+(b[2]-a[2])*t, a[3]+(b[3]-a[3])*t];
        return normalize4(r);
    }
    let theta = dot.acos();
    let st = theta.sin();
    let s0 = ((1.0 - t) * theta).sin() / st;
    let s1 = (t * theta).sin() / st;
    [a[0]*s0+b[0]*s1, a[1]*s0+b[1]*s1, a[2]*s0+b[2]*s1, a[3]*s0+b[3]*s1]
}

fn normalize4(q: [f64; 4]) -> [f64; 4] {
    let len = (q[0]*q[0]+q[1]*q[1]+q[2]*q[2]+q[3]*q[3]).sqrt();
    if len < 1e-10 { return [0.0,0.0,0.0,1.0]; }
    [q[0]/len, q[1]/len, q[2]/len, q[3]/len]
}

#[derive(Clone, Debug)]
pub struct SkeletalClip {
    pub name: String,
    pub duration: f64,
    pub tracks: Vec<BoneTrack>,
}

impl SkeletalClip {
    pub fn new(name: impl Into<String>, duration: f64) -> Self {
        Self { name: name.into(), duration, tracks: Vec::new() }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum TransitionCondition {
    TimeElapsed(f64),
    Always,
}

#[derive(Clone, Debug)]
pub struct AnimTransition {
    pub from: usize,
    pub to: usize,
    pub condition: TransitionCondition,
    pub blend_duration: f64,
}

#[derive(Clone, Debug)]
pub struct AnimStateMachine {
    pub states: Vec<String>,
    pub transitions: Vec<AnimTransition>,
    pub current_state: usize,
    pub time_in_state: f64,
    pub blend_time: f64,
    pub blend_from: usize,
    pub blend_weight: f64,
}

impl AnimStateMachine {
    pub fn new() -> Self {
        Self {
            states: vec!["Idle".to_string(), "Walk".to_string(), "Run".to_string(), "Jump".to_string()],
            transitions: Vec::new(),
            current_state: 0,
            time_in_state: 0.0,
            blend_time: 0.0,
            blend_from: 0,
            blend_weight: 1.0,
        }
    }

    pub fn advance(&mut self, dt: f64) {
        self.time_in_state += dt;
        if self.blend_weight < 1.0 {
            self.blend_weight = (self.blend_weight + dt / self.blend_time.max(0.01)).min(1.0);
        }
        for i in 0..self.transitions.len() {
            let t = &self.transitions[i];
            if t.from != self.current_state { continue; }
            let triggered = match t.condition {
                TransitionCondition::TimeElapsed(threshold) => self.time_in_state >= threshold,
                TransitionCondition::Always => true,
            };
            if triggered {
                let to = t.to;
                let bd = t.blend_duration;
                self.blend_from = self.current_state;
                self.current_state = to;
                self.time_in_state = 0.0;
                self.blend_weight = 0.0;
                self.blend_time = bd;
                break;
            }
        }
    }

    pub fn trigger_transition_to(&mut self, state_index: usize, blend_duration: f64) {
        self.blend_from = self.current_state;
        self.current_state = state_index;
        self.time_in_state = 0.0;
        self.blend_weight = 0.0;
        self.blend_time = blend_duration;
    }
}

impl Default for AnimStateMachine {
    fn default() -> Self { Self::new() }
}

#[derive(Clone, Debug)]
pub struct Skeleton {
    pub bones: Vec<Bone>,
    pub clips: Vec<SkeletalClip>,
    pub state_machine: AnimStateMachine,
    pub current_time: f64,
    pub playing: bool,
}

impl Skeleton {
    pub fn new() -> Self {
        Self {
            bones: Vec::new(),
            clips: Vec::new(),
            state_machine: AnimStateMachine::new(),
            current_time: 0.0,
            playing: false,
        }
    }

    pub fn advance(&mut self, dt: f64) {
        if !self.playing { return; }
        self.state_machine.advance(dt);
        self.current_time += dt;
        let state = self.state_machine.current_state;
        if let Some(clip) = self.clips.get(state) {
            if clip.duration > 0.0 {
                self.current_time %= clip.duration;
            }
        }
        self.apply_pose();
    }

    pub fn apply_pose(&mut self) {
        let state = self.state_machine.current_state;
        let time = self.current_time;
        if let Some(clip) = self.clips.get(state) {
            for track in &clip.tracks {
                if let Some(kf) = track.sample(time) {
                    if let Some(bone) = self.bones.get_mut(track.bone_index) {
                        bone.local_position = kf.position;
                        bone.local_rotation = kf.rotation;
                        bone.local_scale = kf.scale;
                    }
                }
            }
        }
    }
}

impl Default for Skeleton {
    fn default() -> Self { Self::new() }
}
