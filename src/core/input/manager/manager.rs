
use crate::core::coremanager::input_manager::{FrameInput, InputManager};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMode {
    Cinematic,
    Manual,
}

#[derive(Debug, Clone, Copy)]
pub struct InputDriver {
    manager: InputManager,
    mode: InputMode,
}

impl InputDriver {
    pub fn new(mode: InputMode) -> Self {
        let cinematic = matches!(mode, InputMode::Cinematic);
        Self {
            manager: InputManager::new(cinematic),
            mode,
        }
    }

    pub fn cinematic() -> Self {
        Self::new(InputMode::Cinematic)
    }

    pub fn manual() -> Self {
        Self::new(InputMode::Manual)
    }

    pub fn sample(&self, time: f64) -> FrameInput {
        self.manager.sample_cinematic_input(time)
    }

    pub fn mode(&self) -> InputMode {
        self.mode
    }
}

