/// Fine-grained render configuration — overrides presets.
/// Every field `None` means "use the preset default".
#[derive(Debug, Clone, Default)]
pub struct RenderConfig {
    /// Nombre d'échantillons par pixel.
    pub samples_per_pixel: Option<usize>,
    /// Nombre max de rebonds.
    pub max_bounces: Option<u32>,
    /// Nombre de passes.
    pub passes: Option<usize>,
    /// Taille des tuiles.
    pub tile_size: Option<usize>,
    /// Gamma de sortie.
    pub gamma: Option<f64>,
    /// Opérateur de tone-mapping.
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
    Custom {
        /// Gamma de sortie personnalisé.
        gamma: f64,
        /// Exposition globale personnalisée.
        exposure: f64,
    },
}
