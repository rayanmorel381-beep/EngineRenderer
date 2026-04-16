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

/// Directives de rendu produites par l'analyse de scène.
#[derive(Debug, Clone, Copy)]
pub struct AiDirective {
    /// Biais de qualité appliqué au préréglage de rendu, dans `[0.7, 1.25]`.
    pub quality_bias: f64,
    /// Facteur d'échelle de la distance caméra–scène.
    pub camera_distance_scale: f64,
    /// Biais d'exposition additif.
    pub exposure_bias: f64,
}

/// Gestionnaire IA qui analyse la complexité d'une scène et produit des directives
/// de rendu adaptées (qualité, distance caméra, exposition).
#[derive(Debug, Clone, Copy)]
pub struct AiManager {
    aggressiveness: f64,
}

impl AiManager {
    /// Crée un `AiManager` avec le coefficient d'agressivité donné (clampé dans `[0.25, 2.0]`).
    pub fn new(aggressiveness: f64) -> Self {
        Self {
            aggressiveness: aggressiveness.clamp(0.25, 2.0),
        }
    }

    /// Analyse le graphe de scène et retourne des directives de rendu adaptées.
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
