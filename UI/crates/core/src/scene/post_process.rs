#[derive(Clone, Debug)]
pub struct BloomSettings {
    pub enabled: bool,
    pub threshold: f64,
    pub intensity: f64,
    pub scatter: f64,
    pub clamp: f64,
    pub dirt_intensity: f64,
}

impl Default for BloomSettings {
    fn default() -> Self {
        Self { enabled: true, threshold: 0.9, intensity: 0.5, scatter: 0.7, clamp: 65472.0, dirt_intensity: 0.0 }
    }
}

#[derive(Clone, Debug)]
pub struct SsaoSettings {
    pub enabled: bool,
    pub intensity: f64,
    pub radius: f64,
    pub bias: f64,
    pub color: [f64; 4],
    pub sample_count: usize,
}

impl Default for SsaoSettings {
    fn default() -> Self {
        Self { enabled: true, intensity: 0.5, radius: 0.5, bias: 0.025, color: [0.0, 0.0, 0.0, 1.0], sample_count: 8 }
    }
}

#[derive(Clone, Debug)]
pub struct DepthOfFieldSettings {
    pub enabled: bool,
    pub focus_distance: f64,
    pub aperture: f64,
    pub focal_length: f64,
    pub blade_count: usize,
}

impl Default for DepthOfFieldSettings {
    fn default() -> Self {
        Self { enabled: false, focus_distance: 10.0, aperture: 5.6, focal_length: 50.0, blade_count: 5 }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum ToneMappingMode {
    None,
    Neutral,
    ACES,
    Reinhard,
    Filmic,
}

impl Default for ToneMappingMode {
    fn default() -> Self { Self::ACES }
}

impl ToneMappingMode {
    pub fn label(&self) -> &'static str {
        match self { Self::None => "Aucun", Self::Neutral => "Neutre", Self::ACES => "ACES", Self::Reinhard => "Reinhard", Self::Filmic => "Filmic" }
    }
    pub const ALL: [ToneMappingMode; 5] = [ToneMappingMode::None, ToneMappingMode::Neutral, ToneMappingMode::ACES, ToneMappingMode::Reinhard, ToneMappingMode::Filmic];
}

#[derive(Clone, Debug)]
pub struct ColorGrading {
    pub enabled: bool,
    pub exposure: f64,
    pub contrast: f64,
    pub brightness: f64,
    pub saturation: f64,
    pub hue_shift: f64,
    pub tone_mapping: ToneMappingMode,
}

impl Default for ColorGrading {
    fn default() -> Self {
        Self { enabled: true, exposure: 0.0, contrast: 0.0, brightness: 0.0, saturation: 0.0, hue_shift: 0.0, tone_mapping: ToneMappingMode::ACES }
    }
}

#[derive(Clone, Debug)]
pub struct VignetteSettings {
    pub enabled: bool,
    pub color: [f64; 4],
    pub center: [f64; 2],
    pub intensity: f64,
    pub smoothness: f64,
}

impl Default for VignetteSettings {
    fn default() -> Self {
        Self { enabled: false, color: [0.0, 0.0, 0.0, 1.0], center: [0.5, 0.5], intensity: 0.4, smoothness: 0.2 }
    }
}

#[derive(Clone, Debug, Default)]
pub struct PostProcessVolume {
    pub global: bool,
    pub priority: i32,
    pub bloom: BloomSettings,
    pub ssao: SsaoSettings,
    pub dof: DepthOfFieldSettings,
    pub color_grading: ColorGrading,
    pub vignette: VignetteSettings,
    pub ambient_occlusion_enabled: bool,
    pub motion_blur_enabled: bool,
    pub motion_blur_intensity: f64,
    pub chromatic_aberration_enabled: bool,
    pub chromatic_aberration_intensity: f64,
}

impl PostProcessVolume {
    pub fn new() -> Self { Self { global: true, priority: 0, motion_blur_intensity: 0.5, chromatic_aberration_intensity: 0.1, ..Self::default() } }
}
