#[derive(Clone, Debug, PartialEq)]
pub enum AudioRolloff {
    Linear,
    Logarithmic,
    None,
}

impl Default for AudioRolloff {
    fn default() -> Self { Self::Logarithmic }
}

impl AudioRolloff {
    pub fn label(&self) -> &'static str {
        match self { Self::Linear => "Linéaire", Self::Logarithmic => "Logarithmique", Self::None => "Aucun" }
    }
    pub const ALL: [AudioRolloff; 3] = [AudioRolloff::Linear, AudioRolloff::Logarithmic, AudioRolloff::None];
}

#[derive(Clone, Debug)]
pub struct AudioSource {
    pub clip_name: String,
    pub volume: f64,
    pub pitch: f64,
    pub min_distance: f64,
    pub max_distance: f64,
    pub rolloff: AudioRolloff,
    pub looping: bool,
    pub play_on_awake: bool,
    pub spatial: bool,
    pub muted: bool,
    pub priority: u8,
    pub doppler_level: f64,
    pub reverb_zone_mix: f64,
}

impl Default for AudioSource {
    fn default() -> Self {
        Self {
            clip_name: String::new(),
            volume: 1.0,
            pitch: 1.0,
            min_distance: 1.0,
            max_distance: 50.0,
            rolloff: AudioRolloff::Logarithmic,
            looping: false,
            play_on_awake: true,
            spatial: true,
            muted: false,
            priority: 128,
            doppler_level: 1.0,
            reverb_zone_mix: 1.0,
        }
    }
}

impl AudioSource {
    pub fn new() -> Self { Self::default() }
}

#[derive(Clone, Debug)]
pub struct AudioBus {
    pub name: String,
    pub volume: f64,
    pub muted: bool,
    pub send_to: Option<usize>,
}

impl AudioBus {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into(), volume: 1.0, muted: false, send_to: None }
    }
}

#[derive(Clone, Debug, Default)]
pub struct AudioMixer {
    pub buses: Vec<AudioBus>,
    pub master_volume: f64,
}

impl AudioMixer {
    pub fn new() -> Self {
        let mut m = Self { buses: Vec::new(), master_volume: 1.0 };
        m.buses.push(AudioBus::new("Master"));
        m.buses.push(AudioBus::new("Music"));
        m.buses.push(AudioBus::new("SFX"));
        m.buses.push(AudioBus::new("Voice"));
        m
    }
}
