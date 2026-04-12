use crate::core::engine::scene::graph::SceneGraph;

fn detail_score(scene_radius: f64, node_count: usize, luminous_nodes: usize) -> f64 {
    let base = (node_count as f64).ln().max(1.0);
    let luminous_ratio = luminous_nodes as f64 / node_count.max(1) as f64;
    let radius_factor = (scene_radius / 10.0).clamp(0.5, 3.0);
    (base * (1.0 + luminous_ratio) * radius_factor).clamp(0.1, 5.0)
}

fn camera_distance_scale(detail: f64) -> f64 {
    (1.0 + detail * 0.15).clamp(0.8, 2.0)
}

#[derive(Debug, Clone, Copy)]
pub struct AiDirective {
    pub quality_bias: f64,
    pub camera_distance_scale: f64,
    pub exposure_bias: f64,
}

#[derive(Debug, Clone, Copy)]
pub struct AiManager {
    aggressiveness: f64,
}

impl AiManager {
    pub fn new(aggressiveness: f64) -> Self {
        Self {
            aggressiveness: aggressiveness.clamp(0.25, 2.0),
        }
    }

    pub fn analyze(&self, graph: &SceneGraph, detail_scale: f64) -> AiDirective {
        let detail = detail_score(
            graph.scene_radius(),
            graph.node_count(),
            graph.luminous_node_count(),
        ) * detail_scale.clamp(0.75, 2.5);

        AiDirective {
            quality_bias: (0.82 + detail * 0.22 * self.aggressiveness).clamp(0.7, 1.25),
            camera_distance_scale: camera_distance_scale(detail),
            exposure_bias: (1.0 + detail * 0.12).clamp(0.95, 1.18),
        }
    }
}
