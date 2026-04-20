
use crate::core::coremanager::audio_manager::{AudioManager, AudioMix};
use crate::core::engine::scene::graph::SceneGraph;

#[derive(Debug, Clone)]
pub struct Mixer {
    manager: AudioManager,
    label: &'static str,
}

impl Mixer {
    pub fn new(base_gain: f64) -> Self {
        Self {
            manager: AudioManager::new(base_gain),
            label: "custom",
        }
    }

    pub fn cinematic() -> Self {
        Self {
            manager: AudioManager::new(0.85),
            label: "cinematic",
        }
    }

    pub fn preview() -> Self {
        Self {
            manager: AudioManager::new(0.5),
            label: "preview",
        }
    }

    pub fn mix(
        &self,
        graph: &SceneGraph,
        camera_distance: f64,
        exposure_bias: f64,
    ) -> AudioMix {
        self.manager.mix_for_scene(graph, camera_distance, exposure_bias)
    }

    pub fn label(&self) -> &str {
        self.label
    }
}

