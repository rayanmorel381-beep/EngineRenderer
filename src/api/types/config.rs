#[derive(Debug, Clone, Default)]
pub struct RenderConfig {
    pub samples_per_pixel: Option<usize>,
    pub max_bounces: Option<u32>,
    pub passes: Option<usize>,
    pub tile_size: Option<usize>,
    pub gamma: Option<f64>,
    pub tone_mapping: Option<ToneMapping>,
}

#[derive(Debug, Clone, Copy)]
pub enum ToneMapping {
    None,
    Reinhard,
    AcesFilmic,
    Uncharted2,
    Custom {
        gamma: f64,
        exposure: f64,
    },
}
