
use crate::core::coremanager::audio_manager::{AudioManager, AudioMix};
use crate::core::engine::scene::graph::SceneGraph;

#[derive(Debug, Clone)]
/// High-level audio mixer facade for scene-driven mixes.
pub struct Mixer {
    manager: AudioManager,
    label: &'static str,
}

impl Mixer {
    /// Creates a custom mixer with a base gain.
    pub fn new(base_gain: f64) -> Self {
        Self {
            manager: AudioManager::new(base_gain),
            label: "custom",
        }
    }

    /// Creates the cinematic mixer profile.
    pub fn cinematic() -> Self {
        Self {
            manager: AudioManager::new(0.85),
            label: "cinematic",
        }
    }

    /// Creates the lightweight preview mixer profile.
    pub fn preview() -> Self {
        Self {
            manager: AudioManager::new(0.5),
            label: "preview",
        }
    }

    /// Computes an audio mix from scene and camera context.
    pub fn mix(
        &self,
        graph: &SceneGraph,
        camera_distance: f64,
        exposure_bias: f64,
    ) -> AudioMix {
        self.manager.mix_for_scene(graph, camera_distance, exposure_bias)
    }

    /// Returns the mixer profile label.
    pub fn label(&self) -> &str {
        self.label
    }
}

