//! Public audio mixing API for crate consumers.
//!
//! Provides [`Mixer`] — a high-level facade to configure and run spatial
//! audio mixing.  Internally delegates to the engine's `AudioManager`.

use crate::core::coremanager::audio_manager::{AudioManager, AudioMix};
use crate::core::engine::scene::graph::SceneGraph;

/// High-level spatial audio mixer exposed to crate consumers.
#[derive(Debug, Clone)]
pub struct Mixer {
    manager: AudioManager,
    label: &'static str,
}

impl Mixer {
    /// Create a mixer with a custom base gain.
    pub fn new(base_gain: f64) -> Self {
        Self {
            manager: AudioManager::new(base_gain),
            label: "custom",
        }
    }

    /// Cinematic preset (gain = 0.85) — tuned for showcase renders.
    pub fn cinematic() -> Self {
        Self {
            manager: AudioManager::new(0.85),
            label: "cinematic",
        }
    }

    /// Preview preset (gain = 0.5) — quieter for draft renders.
    pub fn preview() -> Self {
        Self {
            manager: AudioManager::new(0.5),
            label: "preview",
        }
    }

    /// Produce an [`AudioMix`] for the current scene state.
    pub fn mix(
        &self,
        graph: &SceneGraph,
        camera_distance: f64,
        exposure_bias: f64,
    ) -> AudioMix {
        self.manager.mix_for_scene(graph, camera_distance, exposure_bias)
    }

    /// The preset label.
    pub fn label(&self) -> &str {
        self.label
    }
}

