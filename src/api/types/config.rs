/// Optional render configuration overrides.
#[derive(Debug, Clone, Default)]
pub struct RenderConfig {
    /// Samples per pixel override.
    pub samples_per_pixel: Option<usize>,
    /// Maximum ray bounce count override.
    pub max_bounces: Option<u32>,
    /// Number of accumulation passes override.
    pub passes: Option<usize>,
    /// Tile size override for tiled rendering.
    pub tile_size: Option<usize>,
    /// Gamma correction override.
    pub gamma: Option<f64>,
    /// Tone mapping override.
    pub tone_mapping: Option<ToneMapping>,
}

/// Tone mapping operator.
#[derive(Debug, Clone, Copy)]
pub enum ToneMapping {
    /// No tone mapping.
    None,
    /// Reinhard operator.
    Reinhard,
    /// ACES filmic approximation.
    AcesFilmic,
    /// Uncharted 2 filmic curve.
    Uncharted2,
    /// Custom tone mapping parameters.
    Custom {
        /// Output gamma.
        gamma: f64,
        /// Exposure multiplier.
        exposure: f64,
    },
}
