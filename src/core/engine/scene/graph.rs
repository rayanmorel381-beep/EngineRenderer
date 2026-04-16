use crate::core::engine::rendering::raytracing::Vec3;

use super::celestial::CelestialBodies;

/// Nœud simplifié du graphe de scène représentant un objet (position, rayon, émission).
#[derive(Debug, Clone, Copy)]
pub struct SceneNode {
    /// Position du nœud dans l'espace monde.
    pub position: Vec3,
    /// Rayon de la sphère englobante du nœud.
    pub radius: f64,
    /// Intensité d'émission lumineuse du nœud.
    pub emission_strength: f64,
}

/// Graphe de scène léger dérivé d'un catalogue de corps célestes.
#[derive(Debug, Clone)]
pub struct SceneGraph {
    nodes: Vec<SceneNode>,
    focus_point: Vec3,
    scene_radius: f64,
}

impl SceneGraph {
    /// Construit le graphe à partir d'un catalogue de `CelestialBodies`.
    pub fn from_bodies(catalog: &CelestialBodies) -> Self {
        let nodes = catalog
            .bodies()
            .iter()
            .map(|body| SceneNode {
                position: body.position,
                radius: body.radius,
                emission_strength: body.material.emission.length(),
            })
            .collect::<Vec<_>>();

        Self {
            focus_point: catalog.scene_center(),
            scene_radius: catalog.scene_radius(),
            nodes,
        }
    }

    /// Retourne le point focal du graphe (centre de masse).
    pub fn focus_point(&self) -> Vec3 {
        self.focus_point
    }

    /// Retourne le rayon de la scène calculé depuis les corps sources.
    pub fn scene_radius(&self) -> f64 {
        self.scene_radius
    }

    /// Retourne le nombre total de nœuds dans le graphe.
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Retourne le nombre de nœuds dont l'émission dépasse un seuil visible.
    pub fn luminous_node_count(&self) -> usize {
        self.nodes
            .iter()
            .filter(|node| node.emission_strength > 0.01)
            .count()
    }

    /// Retourne le rayon moyen de tous les nœuds.
    pub fn average_radius(&self) -> f64 {
        if self.nodes.is_empty() {
            0.0
        } else {
            self.nodes.iter().map(|node| node.radius).sum::<f64>() / self.nodes.len() as f64
        }
    }

    /// Retourne une estimation du rayon d'étendue radiale (distance + rayon max).
    pub fn radial_extent_hint(&self) -> f64 {
        self.nodes
            .iter()
            .map(|node| (node.position - self.focus_point).length() + node.radius)
            .fold(self.scene_radius, f64::max)
    }
}
