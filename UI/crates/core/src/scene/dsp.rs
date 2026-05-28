#[derive(Clone, Debug)]
pub struct EqBand {
    pub enabled: bool,
    pub frequency: f64,
    pub gain_db: f64,
    pub q: f64,
}

impl EqBand {
    pub fn new(frequency: f64) -> Self {
        Self { enabled: true, frequency, gain_db: 0.0, q: 1.0 }
    }
}

#[derive(Clone, Debug)]
pub struct ParametricEq {
    pub enabled: bool,
    pub bands: Vec<EqBand>,
}

impl Default for ParametricEq {
    fn default() -> Self {
        Self {
            enabled: false,
            bands: vec![
                EqBand::new(80.0),
                EqBand::new(250.0),
                EqBand::new(1000.0),
                EqBand::new(4000.0),
                EqBand::new(12000.0),
            ],
        }
    }
}

#[derive(Clone, Debug)]
pub struct Compressor {
    pub enabled: bool,
    pub threshold_db: f64,
    pub ratio: f64,
    pub attack_ms: f64,
    pub release_ms: f64,
    pub makeup_gain_db: f64,
    pub knee_db: f64,
}

impl Default for Compressor {
    fn default() -> Self {
        Self { enabled: false, threshold_db: -24.0, ratio: 4.0, attack_ms: 10.0, release_ms: 100.0, makeup_gain_db: 0.0, knee_db: 2.0 }
    }
}

#[derive(Clone, Debug)]
pub struct DelayEffect {
    pub enabled: bool,
    pub delay_ms: f64,
    pub feedback: f64,
    pub wet_mix: f64,
    pub sync_to_tempo: bool,
}

impl Default for DelayEffect {
    fn default() -> Self {
        Self { enabled: false, delay_ms: 250.0, feedback: 0.3, wet_mix: 0.3, sync_to_tempo: false }
    }
}

#[derive(Clone, Debug)]
pub struct ReverbEffect {
    pub enabled: bool,
    pub room_size: f64,
    pub damping: f64,
    pub wet_level: f64,
    pub dry_level: f64,
    pub width: f64,
    pub early_reflections: f64,
    pub pre_delay_ms: f64,
}

impl Default for ReverbEffect {
    fn default() -> Self {
        Self { enabled: false, room_size: 0.5, damping: 0.5, wet_level: 0.3, dry_level: 0.7, width: 1.0, early_reflections: 0.5, pre_delay_ms: 20.0 }
    }
}

#[derive(Clone, Debug, Default)]
pub struct DspChain {
    pub eq: ParametricEq,
    pub compressor: Compressor,
    pub delay: DelayEffect,
    pub reverb: ReverbEffect,
}

impl DspChain {
    pub fn new() -> Self { Self::default() }
}
