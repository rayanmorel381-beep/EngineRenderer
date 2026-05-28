#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Easing {
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
    Step,
}

impl Default for Easing {
    fn default() -> Self { Self::Linear }
}

impl Easing {
    pub fn apply(self, t: f64) -> f64 {
        match self {
            Self::Linear    => t,
            Self::EaseIn    => t * t,
            Self::EaseOut   => t * (2.0 - t),
            Self::EaseInOut => if t < 0.5 { 2.0*t*t } else { -1.0 + (4.0 - 2.0*t)*t },
            Self::Step      => if t < 1.0 { 0.0 } else { 1.0 },
        }
    }
}

#[derive(Clone, Debug)]
pub struct Keyframe {
    pub time: f64,
    pub value: f64,
    pub easing: Easing,
}

impl Keyframe {
    pub fn new(time: f64, value: f64) -> Self {
        Self { time, value, easing: Easing::default() }
    }
    pub fn with_easing(mut self, e: Easing) -> Self { self.easing = e; self }
}

#[derive(Clone, Debug, Default)]
pub struct AnimTrack {
    pub property: String,
    pub keyframes: Vec<Keyframe>,
}

impl AnimTrack {
    pub fn new(property: impl Into<String>) -> Self {
        Self { property: property.into(), keyframes: Vec::new() }
    }

    pub fn add_keyframe(&mut self, kf: Keyframe) {
        let pos = self.keyframes.partition_point(|k| k.time < kf.time);
        self.keyframes.insert(pos, kf);
    }

    pub fn sample(&self, time: f64) -> Option<f64> {
        if self.keyframes.is_empty() { return None; }
        if time <= self.keyframes[0].time { return Some(self.keyframes[0].value); }
        let last = &self.keyframes[self.keyframes.len()-1];
        if time >= last.time { return Some(last.value); }
        let i = self.keyframes.partition_point(|k| k.time <= time) - 1;
        let a = &self.keyframes[i];
        let b = &self.keyframes[i+1];
        let span = (b.time - a.time).max(f64::EPSILON);
        let t = ((time - a.time) / span).clamp(0.0, 1.0);
        Some(a.value + (b.value - a.value) * a.easing.apply(t))
    }
}

#[derive(Clone, Debug, Default)]
pub struct AnimationClip {
    pub name: String,
    pub duration: f64,
    pub looping: bool,
    pub tracks: Vec<AnimTrack>,
}

impl AnimationClip {
    pub fn new(name: impl Into<String>, duration: f64) -> Self {
        Self { name: name.into(), duration, looping: false, tracks: Vec::new() }
    }

    pub fn add_track(&mut self, track: AnimTrack) {
        self.tracks.push(track);
    }

    pub fn track_mut(&mut self, property: &str) -> Option<&mut AnimTrack> {
        self.tracks.iter_mut().find(|t| t.property == property)
    }
}

#[derive(Clone, Debug, Default)]
pub struct Animator {
    pub clips: Vec<AnimationClip>,
    pub active_clip: usize,
    pub time: f64,
    pub playing: bool,
}

impl Animator {
    pub fn new() -> Self { Self::default() }

    pub fn play(&mut self) { self.playing = true; }
    pub fn pause(&mut self) { self.playing = false; }
    pub fn stop(&mut self) { self.playing = false; self.time = 0.0; }

    pub fn active(&self) -> Option<&AnimationClip> {
        self.clips.get(self.active_clip)
    }

    pub fn active_mut(&mut self) -> Option<&mut AnimationClip> {
        self.clips.get_mut(self.active_clip)
    }

    pub fn advance(&mut self, dt: f64) {
        if !self.playing { return; }
        let Some(clip) = self.clips.get(self.active_clip) else { return };
        self.time += dt;
        if self.time > clip.duration {
            if clip.looping { self.time %= clip.duration.max(f64::EPSILON); }
            else { self.time = clip.duration; self.playing = false; }
        }
    }

    pub fn sample_property(&self, property: &str) -> Option<f64> {
        let clip = self.clips.get(self.active_clip)?;
        let track = clip.tracks.iter().find(|t| t.property == property)?;
        track.sample(self.time)
    }
}
