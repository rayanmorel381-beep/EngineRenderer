
// Re-export so `crate::core::config_manager::EngineConfig` resolves.
pub use crate::core::engine::config::EngineConfig;

/// Preset configuration profiles exposed by the config manager.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigPreset {
    /// Ultra HD rendering profile.
    UltraHd,
    /// Production reference rendering profile.
    Production,
}

/// Mutable configuration wrapper with dirty-state tracking.
#[derive(Debug, Clone)]
pub struct ConfigManager {
    config: EngineConfig,
    dirty: bool,
}

impl ConfigManager {
    /// Creates a manager from an existing engine configuration.
    pub fn new(config: EngineConfig) -> Self {
        Self { config, dirty: false }
    }

    /// Creates a manager from a predefined preset.
    pub fn from_preset(preset: ConfigPreset) -> Self {
        match preset {
            ConfigPreset::UltraHd => Self::new(EngineConfig::ultra_hd_cpu()),
            ConfigPreset::Production => Self::new(EngineConfig::production_reference()),
        }
    }

    /// Returns an immutable view of the current configuration.
    pub fn config(&self) -> &EngineConfig {
        &self.config
    }

    /// Returns a mutable view and marks the config as dirty.
    pub fn config_mut(&mut self) -> &mut EngineConfig {
        self.dirty = true;
        &mut self.config
    }

    /// Sets output resolution and marks the config as dirty.
    pub fn set_resolution(&mut self, width: usize, height: usize) {
        self.config.width = width;
        self.config.height = height;
        self.dirty = true;
    }

    /// Indicates whether pending changes have not been applied.
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    /// Marks pending changes as applied.
    pub fn apply(&mut self) {
        self.dirty = false;
    }

    /// Validates the current configuration values.
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
