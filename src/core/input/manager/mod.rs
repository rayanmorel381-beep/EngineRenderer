//! Public input management API for crate consumers.
//!
//! Provides [`InputDriver`] — a high-level input facade with preset modes.
//! Delegates to the engine's `InputManager` for cinematic input sampling.

use crate::core::coremanager::input_manager::{FrameInput, InputManager};

/// Preset input mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMode {
    /// Cinematic auto-pilot — camera orbits, exposure breathes.
    Cinematic,
    /// Manual — no automatic animation.
    Manual,
}

/// High-level input driver exposed to crate consumers.
#[derive(Debug, Clone, Copy)]
pub struct InputDriver {
    manager: InputManager,
    mode: InputMode,
}

impl InputDriver {
    /// Create an input driver with the given mode.
    pub fn new(mode: InputMode) -> Self {
        let cinematic = matches!(mode, InputMode::Cinematic);
        Self {
            manager: InputManager::new(cinematic),
            mode,
        }
    }

    /// Cinematic preset.
    pub fn cinematic() -> Self {
        Self::new(InputMode::Cinematic)
    }

    /// Manual preset.
    pub fn manual() -> Self {
        Self::new(InputMode::Manual)
    }

    /// Sample input for the given absolute time.
    pub fn sample(&self, time: f64) -> FrameInput {
        self.manager.sample_cinematic_input(time)
    }

    /// Current mode.
    pub fn mode(&self) -> InputMode {
        self.mode
    }
}
