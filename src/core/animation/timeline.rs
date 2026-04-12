#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Interpolation {
    #[default]
    Linear,
    Step,
    SmoothStep,
    CubicHermite,
    EaseInQuad,
    EaseOutQuad,
    EaseInOutQuad,
    EaseInCubic,
    EaseOutCubic,
    EaseInOutCubic,
    EaseInSine,
    EaseOutSine,
    EaseInOutSine,
    EaseInExpo,
    EaseOutExpo,
    EaseInBack,
    EaseOutBack,
    BounceOut,
}

#[derive(Debug, Clone)]
pub struct Keyframe<T> {
    pub time:  f64,
    pub value: T,
}

pub trait Lerp: Clone {
    fn lerp(&self, other: &Self, t: f64) -> Self;
}

impl Lerp for f64 {
    fn lerp(&self, other: &Self, t: f64) -> Self {
        self + (other - self) * t
    }
}

impl Lerp for [f64; 3] {
    fn lerp(&self, other: &Self, t: f64) -> Self {
        [
            self[0] + (other[0] - self[0]) * t,
            self[1] + (other[1] - self[1]) * t,
            self[2] + (other[2] - self[2]) * t,
        ]
    }
}

impl Lerp for [f64; 2] {
    fn lerp(&self, other: &Self, t: f64) -> Self {
        [
            self[0] + (other[0] - self[0]) * t,
            self[1] + (other[1] - self[1]) * t,
        ]
    }
}

#[derive(Debug, Clone, Default)]
pub struct Timeline<T: Lerp> {
    pub interpolation: Interpolation,
    keyframes:         Vec<Keyframe<T>>,
}

impl<T: Lerp> Timeline<T> {
    pub fn new(interpolation: Interpolation) -> Self {
        Self { interpolation, keyframes: Vec::new() }
    }

    pub fn add(&mut self, time: f64, value: T) {
        let idx = self.keyframes.partition_point(|k| k.time <= time);
        self.keyframes.insert(idx, Keyframe { time, value });
    }

    pub fn is_empty(&self) -> bool {
        self.keyframes.is_empty()
    }

    pub fn sample(&self, time: f64) -> Option<T> {
        if self.keyframes.is_empty() {
            return None;
        }
        if self.keyframes.len() == 1 || time <= self.keyframes[0].time {
            return Some(self.keyframes[0].value.clone());
        }
        let last = self.keyframes.last().unwrap();
        if time >= last.time {
            return Some(last.value.clone());
        }

        let next_idx = self.keyframes.partition_point(|k| k.time <= time);
        let prev_idx = next_idx - 1;
        let a = &self.keyframes[prev_idx];
        let b = &self.keyframes[next_idx];

        let raw_t = (time - a.time) / (b.time - a.time);
        let t = apply_interpolation(raw_t, self.interpolation);
        Some(a.value.lerp(&b.value, t))
    }
}

fn apply_interpolation(t: f64, mode: Interpolation) -> f64 {
    use super::easing;
    match mode {
        Interpolation::Linear        => easing::linear(t),
        Interpolation::Step          => if t < 1.0 { 0.0 } else { 1.0 },
        Interpolation::SmoothStep    => t * t * (3.0 - 2.0 * t),
        Interpolation::CubicHermite  => t * t * (3.0 - 2.0 * t),
        Interpolation::EaseInQuad    => easing::ease_in_quad(t),
        Interpolation::EaseOutQuad   => easing::ease_out_quad(t),
        Interpolation::EaseInOutQuad => easing::ease_in_out_quad(t),
        Interpolation::EaseInCubic   => easing::ease_in_cubic(t),
        Interpolation::EaseOutCubic  => easing::ease_out_cubic(t),
        Interpolation::EaseInOutCubic => easing::ease_in_out_cubic(t),
        Interpolation::EaseInSine    => easing::ease_in_sine(t),
        Interpolation::EaseOutSine   => easing::ease_out_sine(t),
        Interpolation::EaseInOutSine => easing::ease_in_out_sine(t),
        Interpolation::EaseInExpo    => easing::ease_in_expo(t),
        Interpolation::EaseOutExpo   => easing::ease_out_expo(t),
        Interpolation::EaseInBack    => easing::ease_in_back(t),
        Interpolation::EaseOutBack   => easing::ease_out_back(t),
        Interpolation::BounceOut     => easing::bounce_out(t),
    }
}
