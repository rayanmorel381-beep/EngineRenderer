/// Fine-grained render configuration — overrides presets.
/// Every field `None` means "use the preset default".
#[derive(Debug, Clone, Default)]
pub struct RenderConfig {
    pub samples_per_pixel: Option<usize>,
    pub max_bounces: Option<u32>,
    pub passes: Option<usize>,
    pub tile_size: Option<usize>,
    pub gamma: Option<f64>,
    pub tone_mapping: Option<ToneMapping>,
}

/// Tone-mapping operator.
#[derive(Debug, Clone, Copy)]
pub enum ToneMapping {
    /// No tone mapping (linear clamp).
    None,
    /// Reinhard global operator.
    Reinhard,
    /// ACES filmic curve.
    AcesFilmic,
    /// Uncharted 2 filmic.
    Uncharted2,
    /// Custom gamma + exposure.
    Custom { gamma: f64, exposure: f64 },
}
