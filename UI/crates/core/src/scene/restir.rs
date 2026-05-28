#[derive(Clone, Debug, PartialEq)]
pub enum RestirMode {
    Disabled,
    SpatialOnly,
    TemporalOnly,
    SpatioTemporal,
}

impl RestirMode {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Disabled => "Désactivé",
            Self::SpatialOnly => "Spatial",
            Self::TemporalOnly => "Temporel",
            Self::SpatioTemporal => "Spatio-Temporel",
        }
    }
    pub const ALL: [RestirMode; 4] = [
        RestirMode::Disabled,
        RestirMode::SpatialOnly,
        RestirMode::TemporalOnly,
        RestirMode::SpatioTemporal,
    ];
}

impl Default for RestirMode {
    fn default() -> Self { Self::SpatioTemporal }
}

#[derive(Clone, Debug)]
pub struct RestirSettings {
    pub mode: RestirMode,
    pub reservoir_size: usize,
    pub spatial_radius: f64,
    pub spatial_samples: usize,
    pub temporal_history_length: usize,
    pub m_cap: usize,
    pub bias_correction: bool,
    pub visibility_reuse: bool,
    pub candidate_lights: usize,
    pub jacobian_clamping: f64,
}

impl Default for RestirSettings {
    fn default() -> Self {
        Self {
            mode: RestirMode::SpatioTemporal,
            reservoir_size: 1,
            spatial_radius: 30.0,
            spatial_samples: 5,
            temporal_history_length: 20,
            m_cap: 20,
            bias_correction: true,
            visibility_reuse: true,
            candidate_lights: 32,
            jacobian_clamping: 10.0,
        }
    }
}

impl RestirSettings {
    pub fn new() -> Self { Self::default() }
}
