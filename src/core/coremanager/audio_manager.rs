use crate::core::engine::scene::graph::SceneGraph;

#[derive(Debug, Clone, Copy)]
pub struct AudioMix {
    pub master_gain: f64,
    pub spatial_width: f64,
    pub reverb_send: f64,
}

#[derive(Debug, Clone, Copy)]
pub struct AudioManager {
    base_gain: f64,
}

impl AudioManager {
    pub fn new(base_gain: f64) -> Self {
        Self {
            base_gain: base_gain.clamp(0.1, 2.0),
        }
    }

    pub fn mix_for_scene(
        &self,
        graph: &SceneGraph,
        camera_distance: f64,
        exposure_bias: f64,
    ) -> AudioMix {
        let luminous_factor = graph.luminous_node_count().max(1) as f64;
        let distance_factor = (camera_distance / (graph.scene_radius() + 1.0)).clamp(0.5, 2.0);

        AudioMix {
            master_gain: self.base_gain * (0.85 + luminous_factor * 0.03) / distance_factor,
            spatial_width: (0.55 + graph.node_count() as f64 * 0.04).clamp(0.55, 1.0),
            reverb_send: (0.12 + exposure_bias * 0.18).clamp(0.1, 0.4),
        }
    }
}
