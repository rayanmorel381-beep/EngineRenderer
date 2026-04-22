use crate::api::scenes::SceneDescriptor;
use crate::api::types::CameraDesc;
use super::timeline::{Timeline, Lerp};

/// Camera state sampled for a single animation time.
#[derive(Debug, Clone)]
pub struct CameraFrame {
    /// Camera eye position.
    pub eye:         [f64; 3],
    /// Camera look-at target.
    pub target:      [f64; 3],
    /// Vertical field of view in degrees.
    pub fov_degrees: f64,
    /// Lens aperture value.
    pub aperture:    f64,
}

impl Default for CameraFrame {
    fn default() -> Self {
        let d = CameraDesc::default();
        Self { eye: d.eye, target: d.target, fov_degrees: d.fov_degrees, aperture: d.aperture }
    }
}

impl Lerp for CameraFrame {
    fn lerp(&self, other: &Self, t: f64) -> Self {
        Self {
            eye:         self.eye.lerp(&other.eye, t),
            target:      self.target.lerp(&other.target, t),
            fov_degrees: self.fov_degrees.lerp(&other.fov_degrees, t),
            aperture:    self.aperture.lerp(&other.aperture, t),
        }
    }
}

/// Directional sun state sampled for a single animation time.
#[derive(Debug, Clone)]
pub struct SunFrame {
    /// Sun direction vector.
    pub direction: [f64; 3],
    /// Sun light color.
    pub color:     [f64; 3],
    /// Sun intensity multiplier.
    pub intensity: f64,
}

impl Lerp for SunFrame {
    fn lerp(&self, other: &Self, t: f64) -> Self {
        Self {
            direction: self.direction.lerp(&other.direction, t),
            color:     self.color.lerp(&other.color, t),
            intensity: self.intensity.lerp(&other.intensity, t),
        }
    }
}

/// Sky gradient state sampled for a single animation time.
#[derive(Debug, Clone)]
pub struct SkyFrame {
    /// Sky color at zenith.
    pub top:    [f64; 3],
    /// Sky color at horizon.
    pub bottom: [f64; 3],
}

impl Lerp for SkyFrame {
    fn lerp(&self, other: &Self, t: f64) -> Self {
        Self {
            top:    self.top.lerp(&other.top, t),
            bottom: self.bottom.lerp(&other.bottom, t),
        }
    }
}

/// Animation clip with optional timelines for scene channels.
#[derive(Debug, Clone)]
pub struct AnimationClip {
    /// Clip duration in seconds.
    pub duration_secs: f64,
    /// Frames per second.
    pub fps:           f64,
    /// Optional animated camera timeline.
    pub camera:        Option<Timeline<CameraFrame>>,
    /// Optional animated sun timeline.
    pub sun:           Option<Timeline<SunFrame>>,
    /// Optional animated sky timeline.
    pub sky:           Option<Timeline<SkyFrame>>,
    /// Optional animated exposure timeline.
    pub exposure:      Option<Timeline<f64>>,
}

impl AnimationClip {
    /// Creates a new animation clip.
    pub fn new(duration_secs: f64, fps: f64) -> Self {
        Self {
            duration_secs,
            fps,
            camera:   None,
            sun:      None,
            sky:      None,
            exposure: None,
        }
    }

    /// Returns the number of frames in the clip.
    pub fn frame_count(&self) -> usize {
        (self.duration_secs * self.fps).ceil() as usize
    }

    /// Returns the timestamp of a frame index in seconds.
    pub fn time_for_frame(&self, frame: usize) -> f64 {
        frame as f64 / self.fps
    }

    /// Evaluates animated channels over a base scene at a given time.
    pub fn evaluate(&self, base: &SceneDescriptor, time: f64) -> SceneDescriptor {
        let mut scene = base.clone();

        if let Some(cam_tl) = &self.camera
            && let Some(cf) = cam_tl.sample(time) {
            scene.camera.eye         = cf.eye;
            scene.camera.target      = cf.target;
            scene.camera.fov_degrees = cf.fov_degrees;
            scene.camera.aperture    = cf.aperture;
        }
        if let Some(sun_tl) = &self.sun
            && let Some(sf) = sun_tl.sample(time) {
            scene.sun_direction = sf.direction;
            scene.sun_color     = sf.color;
            scene.sun_intensity = sf.intensity;
        }
        if let Some(sky_tl) = &self.sky
            && let Some(sk) = sky_tl.sample(time) {
            scene.sky_top    = sk.top;
            scene.sky_bottom = sk.bottom;
        }
        if let Some(exp_tl) = &self.exposure
            && let Some(e) = exp_tl.sample(time) {
            scene.exposure = e;
        }

        scene
    }
}
