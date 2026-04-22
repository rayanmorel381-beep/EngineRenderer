/// Interpolation mode used between timeline keyframes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Interpolation {
    #[default]
    /// Linear interpolation.
    Linear,
    /// Step interpolation.
    Step,
    /// Smoothstep interpolation.
    SmoothStep,
    /// Cubic Hermite-like smooth interpolation.
    CubicHermite,
    /// Quadratic ease-in interpolation.
    EaseInQuad,
    /// Quadratic ease-out interpolation.
    EaseOutQuad,
    /// Quadratic ease-in/out interpolation.
    EaseInOutQuad,
    /// Cubic ease-in interpolation.
    EaseInCubic,
    /// Cubic ease-out interpolation.
    EaseOutCubic,
    /// Cubic ease-in/out interpolation.
    EaseInOutCubic,
    /// Sine ease-in interpolation.
    EaseInSine,
    /// Sine ease-out interpolation.
    EaseOutSine,
    /// Sine ease-in/out interpolation.
    EaseInOutSine,
    /// Exponential ease-in interpolation.
    EaseInExpo,
    /// Exponential ease-out interpolation.
    EaseOutExpo,
    /// Back ease-in interpolation.
    EaseInBack,
    /// Back ease-out interpolation.
    EaseOutBack,
    /// Bounce ease-out interpolation.
    BounceOut,
}

/// Time/value keyframe used in timelines.
#[derive(Debug, Clone)]
pub struct Keyframe<T> {
    /// Keyframe timestamp in seconds.
    pub time:  f64,
    /// Keyframe value.
    pub value: T,
}

/// Trait for linearly interpolating values.
pub trait Lerp: Clone {
    /// Returns the interpolated value at normalized factor t.
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

/// Generic timeline of keyframes.
#[derive(Debug, Clone, Default)]
pub struct Timeline<T: Lerp> {
    /// Interpolation mode used between keyframes.
    pub interpolation: Interpolation,
    keyframes:         Vec<Keyframe<T>>,
}

impl<T: Lerp> Timeline<T> {
    /// Creates an empty timeline with the requested interpolation mode.
    pub fn new(interpolation: Interpolation) -> Self {
        Self { interpolation, keyframes: Vec::new() }
    }

    /// Inserts a keyframe while keeping chronological order.
    pub fn add(&mut self, time: f64, value: T) {
        let idx = self.keyframes.partition_point(|k| k.time <= time);
        self.keyframes.insert(idx, Keyframe { time, value });
    }

    /// Returns true when no keyframes are present.
    pub fn is_empty(&self) -> bool {
        self.keyframes.is_empty()
    }

    /// Samples the timeline at a given time.
    pub fn sample(&self, time: f64) -> Option<T> {
        if self.keyframes.is_empty() {
            return None;
        }
        if self.keyframes.len() == 1 || time <= self.keyframes[0].time {
            return Some(self.keyframes[0].value.clone());
        }
        let last = &self.keyframes[self.keyframes.len() - 1];
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
