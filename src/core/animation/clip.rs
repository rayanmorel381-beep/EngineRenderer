use crate::api::scene_descriptor::SceneDescriptor;
use crate::api::types::CameraDesc;
use super::timeline::{Timeline, Lerp};

#[derive(Debug, Clone)]
pub struct CameraFrame {
    pub eye:         [f64; 3],
    pub target:      [f64; 3],
    pub fov_degrees: f64,
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

#[derive(Debug, Clone)]
pub struct SunFrame {
    pub direction: [f64; 3],
    pub color:     [f64; 3],
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

#[derive(Debug, Clone)]
pub struct SkyFrame {
    pub top:    [f64; 3],
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

#[derive(Debug, Clone)]
pub struct AnimationClip {
    pub duration_secs: f64,
    pub fps:           f64,
    pub camera:        Option<Timeline<CameraFrame>>,
    pub sun:           Option<Timeline<SunFrame>>,
    pub sky:           Option<Timeline<SkyFrame>>,
    pub exposure:      Option<Timeline<f64>>,
}

impl AnimationClip {
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

    pub fn frame_count(&self) -> usize {
        (self.duration_secs * self.fps).ceil() as usize
    }

    pub fn time_for_frame(&self, frame: usize) -> f64 {
        frame as f64 / self.fps
    }

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
