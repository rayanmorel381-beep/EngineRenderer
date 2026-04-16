use crate::api::scenes::SceneDescriptor;
use crate::api::types::CameraDesc;
use super::timeline::{Timeline, Lerp};

/// État caméra interpolable.
#[derive(Debug, Clone)]
pub struct CameraFrame {
    /// Position caméra.
    pub eye:         [f64; 3],
    /// Cible caméra.
    pub target:      [f64; 3],
    /// FOV vertical.
    pub fov_degrees: f64,
    /// Ouverture.
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

/// État solaire interpolable.
#[derive(Debug, Clone)]
pub struct SunFrame {
    /// Direction du soleil.
    pub direction: [f64; 3],
    /// Couleur solaire.
    pub color:     [f64; 3],
    /// Intensité solaire.
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

/// État de ciel interpolable.
#[derive(Debug, Clone)]
pub struct SkyFrame {
    /// Couleur du haut de ciel.
    pub top:    [f64; 3],
    /// Couleur du bas de ciel.
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

/// Clip d'animation regroupant les timelines de paramètres de scène.
#[derive(Debug, Clone)]
pub struct AnimationClip {
    /// Durée en secondes.
    pub duration_secs: f64,
    /// FPS cible.
    pub fps:           f64,
    /// Timeline caméra optionnelle.
    pub camera:        Option<Timeline<CameraFrame>>,
    /// Timeline solaire optionnelle.
    pub sun:           Option<Timeline<SunFrame>>,
    /// Timeline ciel optionnelle.
    pub sky:           Option<Timeline<SkyFrame>>,
    /// Timeline exposition optionnelle.
    pub exposure:      Option<Timeline<f64>>,
}

impl AnimationClip {
    /// Crée un clip vide.
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

    /// Retourne le nombre total de frames.
    pub fn frame_count(&self) -> usize {
        (self.duration_secs * self.fps).ceil() as usize
    }

    /// Retourne le temps associé à une frame.
    pub fn time_for_frame(&self, frame: usize) -> f64 {
        frame as f64 / self.fps
    }

    /// Évalue le clip sur une scène de base à l'instant `time`.
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
