#[derive(Clone, Debug)]
pub struct LodLevel {
    pub screen_relative_height: f64,
    pub culled: bool,
    pub transition_width: f64,
}

impl LodLevel {
    pub fn new(screen_relative_height: f64) -> Self {
        Self { screen_relative_height, culled: false, transition_width: 0.0 }
    }
}

#[derive(Clone, Debug)]
pub struct LodGroup {
    pub enabled: bool,
    pub fade_mode_cross_fade: bool,
    pub animate_cross_fading: bool,
    pub levels: Vec<LodLevel>,
}

impl Default for LodGroup {
    fn default() -> Self {
        Self {
            enabled: true,
            fade_mode_cross_fade: true,
            animate_cross_fading: false,
            levels: vec![
                LodLevel::new(0.60),
                LodLevel::new(0.30),
                LodLevel::new(0.10),
                LodLevel::new(0.03),
            ],
        }
    }
}

impl LodGroup {
    pub fn new() -> Self { Self::default() }

    pub fn active_level(&self, screen_relative_height: f64) -> usize {
        if !self.enabled { return 0; }
        self.levels.iter()
            .position(|lod| screen_relative_height >= lod.screen_relative_height)
            .unwrap_or(self.levels.len().saturating_sub(1))
    }

    pub fn culled(&self, screen_relative_height: f64) -> bool {
        if !self.enabled { return false; }
        self.levels.iter().all(|lod| screen_relative_height < lod.screen_relative_height)
    }
}
