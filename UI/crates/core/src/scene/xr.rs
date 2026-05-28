#[derive(Clone, Debug, PartialEq)]
pub enum XrMode { Disabled, VR, AR, MixedReality }

impl XrMode {
    pub fn label(&self) -> &'static str {
        match self { Self::Disabled => "Désactivé", Self::VR => "VR", Self::AR => "AR", Self::MixedReality => "Mixed Reality" }
    }
    pub const ALL: [XrMode; 4] = [XrMode::Disabled, XrMode::VR, XrMode::AR, XrMode::MixedReality];
}

impl Default for XrMode { fn default() -> Self { Self::Disabled } }

#[derive(Clone, Debug, PartialEq)]
pub enum XrTrackingOrigin { Floor, Eye, Stage }

impl XrTrackingOrigin {
    pub fn label(&self) -> &'static str {
        match self { Self::Floor => "Sol", Self::Eye => "Yeux", Self::Stage => "Scène" }
    }
    pub const ALL: [XrTrackingOrigin; 3] = [XrTrackingOrigin::Floor, XrTrackingOrigin::Eye, XrTrackingOrigin::Stage];
}

impl Default for XrTrackingOrigin { fn default() -> Self { Self::Floor } }

#[derive(Clone, Debug)]
pub struct XrHandData {
    pub position: [f64; 3],
    pub rotation: [f64; 4],
    pub grip: f64,
    pub trigger: f64,
    pub thumbstick: [f64; 2],
    pub haptic_amplitude: f64,
}

impl Default for XrHandData {
    fn default() -> Self {
        Self { position: [0.0; 3], rotation: [0.0, 0.0, 0.0, 1.0], grip: 0.0, trigger: 0.0, thumbstick: [0.0; 2], haptic_amplitude: 0.0 }
    }
}

#[derive(Clone, Debug)]
pub struct XrSettings {
    pub mode: XrMode,
    pub tracking_origin: XrTrackingOrigin,
    pub render_scale: f64,
    pub foveation_level: u8,
    pub refresh_rate: f64,
    pub ipd_mm: f64,
    pub left_hand: XrHandData,
    pub right_hand: XrHandData,
    pub hand_tracking: bool,
    pub eye_tracking: bool,
    pub passthrough: bool,
    pub fov_h_deg: f64,
    pub fov_v_deg: f64,
}

impl Default for XrSettings {
    fn default() -> Self {
        Self {
            mode: XrMode::Disabled,
            tracking_origin: XrTrackingOrigin::Floor,
            render_scale: 1.0,
            foveation_level: 0,
            refresh_rate: 90.0,
            ipd_mm: 64.0,
            left_hand: XrHandData::default(),
            right_hand: XrHandData::default(),
            hand_tracking: false,
            eye_tracking: false,
            passthrough: false,
            fov_h_deg: 90.0,
            fov_v_deg: 90.0,
        }
    }
}

impl XrSettings {
    pub fn new() -> Self { Self::default() }
}
