
// Re-export so `crate::core::config_manager::EngineConfig` resolves.
pub use crate::core::engine::config::EngineConfig;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigPreset {
    UltraHd,
    Production,
}

#[derive(Debug, Clone)]
pub struct ConfigManager {
    config: EngineConfig,
    dirty: bool,
}

impl ConfigManager {
    pub fn new(config: EngineConfig) -> Self {
        Self { config, dirty: false }
    }

    pub fn from_preset(preset: ConfigPreset) -> Self {
        match preset {
            ConfigPreset::UltraHd => Self::new(EngineConfig::ultra_hd_cpu()),
            ConfigPreset::Production => Self::new(EngineConfig::production_reference()),
        }
    }

    pub fn config(&self) -> &EngineConfig {
        &self.config
    }

    pub fn config_mut(&mut self) -> &mut EngineConfig {
        self.dirty = true;
        &mut self.config
    }

    pub fn set_resolution(&mut self, width: usize, height: usize) {
        self.config.width = width;
        self.config.height = height;
        self.dirty = true;
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    pub fn apply(&mut self) {
        self.dirty = false;
    }

    pub fn validate(&self) -> Result<(), &'static str> {
        if self.config.width == 0 || self.config.height == 0 {
            return Err("resolution must be non-zero");
        }
        if self.config.width > 7680 || self.config.height > 4320 {
            return Err("resolution exceeds 8K maximum");
        }
        Ok(())
    }
}
