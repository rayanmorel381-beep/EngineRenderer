#[derive(Clone, Debug, PartialEq)]
pub enum GiMode {
    None,
    Baked,
    Dynamic,
    Hybrid,
}

impl GiMode {
    pub fn label(&self) -> &'static str {
        match self { Self::None => "Aucun", Self::Baked => "Baked", Self::Dynamic => "Dynamique", Self::Hybrid => "Hybride" }
    }
    pub const ALL: [GiMode; 4] = [GiMode::None, GiMode::Baked, GiMode::Dynamic, GiMode::Hybrid];
}

impl Default for GiMode { fn default() -> Self { Self::Dynamic } }

#[derive(Clone, Debug)]
pub struct LightProbe {
    pub name: String,
    pub position: [f64; 3],
    pub radius: f64,
    pub intensity: f64,
    pub enabled: bool,
}

impl LightProbe {
    pub fn new(name: impl Into<String>, position: [f64; 3]) -> Self {
        Self { name: name.into(), position, radius: 5.0, intensity: 1.0, enabled: true }
    }
}

#[derive(Clone, Debug)]
pub struct IrradianceVolume {
    pub name: String,
    pub bounds_min: [f64; 3],
    pub bounds_max: [f64; 3],
    pub probe_spacing: f64,
    pub intensity: f64,
    pub enabled: bool,
}

impl IrradianceVolume {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            bounds_min: [-5.0, 0.0, -5.0],
            bounds_max: [5.0, 5.0, 5.0],
            probe_spacing: 1.0,
            intensity: 1.0,
            enabled: true,
        }
    }
}

#[derive(Clone, Debug)]
pub struct GiSettings {
    pub mode: GiMode,
    pub bounces: usize,
    pub samples_per_probe: usize,
    pub ambient_occlusion: bool,
    pub ao_radius: f64,
    pub sky_light_intensity: f64,
    pub emission_intensity_scale: f64,
    pub probes: Vec<LightProbe>,
    pub volumes: Vec<IrradianceVolume>,
}

impl Default for GiSettings {
    fn default() -> Self {
        Self {
            mode: GiMode::Dynamic,
            bounces: 3,
            samples_per_probe: 256,
            ambient_occlusion: true,
            ao_radius: 0.5,
            sky_light_intensity: 1.0,
            emission_intensity_scale: 1.0,
            probes: Vec::new(),
            volumes: Vec::new(),
        }
    }
}

impl GiSettings {
    pub fn new() -> Self { Self::default() }
    pub fn add_probe(&mut self, name: impl Into<String>, position: [f64; 3]) { self.probes.push(LightProbe::new(name, position)); }
    pub fn add_volume(&mut self, name: impl Into<String>) { self.volumes.push(IrradianceVolume::new(name)); }
}
