//! Configuration management for the engine runtime.
//!
//! [`ConfigManager`] wraps [`EngineConfig`] with validation,
//! preset selection, and runtime resolution adjustment.

// Re-export so `crate::core::config_manager::EngineConfig` resolves.
pub use crate::core::engine::config::EngineConfig;

/// Preset identifiers for quick configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigPreset {
    /// Preset Ultra HD orienté rendu CPU.
    UltraHd,
    /// Preset référence production.
    Production,
}

/// Manages engine configuration with validation and preset support.
#[derive(Debug, Clone)]
pub struct ConfigManager {
    config: EngineConfig,
    dirty: bool,
}

impl ConfigManager {
    /// Crée un gestionnaire à partir d'une configuration explicite.
    pub fn new(config: EngineConfig) -> Self {
        Self { config, dirty: false }
    }

    /// Crée un gestionnaire à partir d'un preset prédéfini.
    pub fn from_preset(preset: ConfigPreset) -> Self {
        match preset {
            ConfigPreset::UltraHd => Self::new(EngineConfig::ultra_hd_cpu()),
            ConfigPreset::Production => Self::new(EngineConfig::production_reference()),
        }
    }

    /// Retourne une vue immuable de la configuration courante.
    pub fn config(&self) -> &EngineConfig {
        &self.config
    }

    /// Retourne une vue mutable de la configuration et marque l'état comme modifié.
    pub fn config_mut(&mut self) -> &mut EngineConfig {
        self.dirty = true;
        &mut self.config
    }

    /// Met à jour la résolution cible.
    pub fn set_resolution(&mut self, width: usize, height: usize) {
        self.config.width = width;
        self.config.height = height;
        self.dirty = true;
    }

    /// Indique si la configuration a été modifiée depuis le dernier `apply`.
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    /// Marque les modifications comme appliquées.
    pub fn apply(&mut self) {
        self.dirty = false;
    }

    /// Valide la cohérence de la configuration (résolution minimale et maximale).
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
