use crate::core::engine::rendering::raytracing::Vec3;

use super::celestial::CelestialBodies;

#[derive(Debug, Clone, Copy)]
pub struct SceneNode {
    pub position: Vec3,
    pub radius: f64,
    pub emission_strength: f64,
}

#[derive(Debug, Clone)]
pub struct SceneGraph {
    nodes: Vec<SceneNode>,
    focus_point: Vec3,
    scene_radius: f64,
}

impl SceneGraph {
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

    pub fn focus_point(&self) -> Vec3 {
        self.focus_point
    }

    pub fn scene_radius(&self) -> f64 {
        self.scene_radius
    }

    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    pub fn luminous_node_count(&self) -> usize {
        self.nodes
            .iter()
            .filter(|node| node.emission_strength > 0.01)
            .count()
    }

    pub fn average_radius(&self) -> f64 {
        if self.nodes.is_empty() {
            0.0
        } else {
            self.nodes.iter().map(|node| node.radius).sum::<f64>() / self.nodes.len() as f64
        }
    }

    pub fn radial_extent_hint(&self) -> f64 {
        self.nodes
            .iter()
            .map(|node| (node.position - self.focus_point).length() + node.radius)
            .fold(self.scene_radius, f64::max)
    }
}
