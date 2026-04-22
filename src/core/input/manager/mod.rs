
use crate::core::coremanager::input_manager::{FrameInput, InputManager};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Input mode used by the input driver.
pub enum InputMode {
    /// Uses generated cinematic input.
    Cinematic,
    /// Uses manual input mode.
    Manual,
}

#[derive(Debug, Clone, Copy)]
/// High-level input driver wrapper.
pub struct InputDriver {
    manager: InputManager,
    mode: InputMode,
}

impl InputDriver {
    /// Creates a new input driver for the selected mode.
    pub fn new(mode: InputMode) -> Self {
        let cinematic = matches!(mode, InputMode::Cinematic);
        Self {
            manager: InputManager::new(cinematic),
            mode,
        }
    }

    /// Creates a cinematic input driver.
    pub fn cinematic() -> Self {
        Self::new(InputMode::Cinematic)
    }

    /// Creates a manual input driver.
    pub fn manual() -> Self {
        Self::new(InputMode::Manual)
    }

    /// Samples frame input at a given time.
    pub fn sample(&self, time: f64) -> FrameInput {
        self.manager.sample_cinematic_input(time)
    }

    /// Returns the currently configured input mode.
    pub fn mode(&self) -> InputMode {
        self.mode
    }
}
