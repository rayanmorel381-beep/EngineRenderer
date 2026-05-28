use crate::core::engine::rendering::raytracing::Vec3;

use super::celestial::CelestialBodies;

#[derive(Debug, Clone, Copy)]
/// Lightweight node extracted from scene bodies.
pub struct SceneNode {
    /// Node position.
    pub position: Vec3,
    /// Node bounding radius.
    pub radius: f64,
    /// Emission magnitude hint.
    pub emission_strength: f64,
}

#[derive(Debug, Clone)]
/// Scene graph summary used by runtime systems.
pub struct SceneGraph {
    nodes: Vec<SceneNode>,
    focus_point: Vec3,
    scene_radius: f64,
}

impl SceneGraph {
    /// Builds a graph summary from celestial bodies.
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

    /// Returns the graph focus point.
    pub fn focus_point(&self) -> Vec3 {
        self.focus_point
    }

    /// Returns the graph scene radius.
    pub fn scene_radius(&self) -> f64 {
        self.scene_radius
    }

    /// Returns the number of nodes in the graph.
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Returns the number of emissive nodes.
    pub fn luminous_node_count(&self) -> usize {
        self.nodes
            .iter()
            .filter(|node| node.emission_strength > 0.01)
            .count()
    }

    /// Returns average node radius.
    pub fn average_radius(&self) -> f64 {
        if self.nodes.is_empty() {
            0.0
        } else {
            self.nodes.iter().map(|node| node.radius).sum::<f64>() / self.nodes.len() as f64
        }
    }

    /// Returns a radial extent hint used for runtime heuristics.
    pub fn radial_extent_hint(&self) -> f64 {
        self.nodes
            .iter()
            .map(|node| (node.position - self.focus_point).length() + node.radius)
            .fold(self.scene_radius, f64::max)
    }
}
