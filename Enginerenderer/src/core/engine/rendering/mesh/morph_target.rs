use super::asset::MeshAsset;
use super::vertex::Vertex;
use crate::core::engine::rendering::raytracing::Vec3;

#[derive(Debug, Clone)]
pub struct MorphTarget {
    pub name: String,
    pub delta_positions: Vec<Vec3>,
    pub delta_normals: Vec<Vec3>,
}

impl MorphTarget {
    pub fn new(name: &str, base: &MeshAsset, target_positions: &[Vec3]) -> Self {
        let delta_positions = base
            .vertices
            .iter()
            .zip(target_positions.iter())
            .map(|(v, &tp)| tp - v.position)
            .collect();
        let delta_normals = vec![Vec3::ZERO; base.vertices.len()];
        Self {
            name: name.to_string(),
            delta_positions,
            delta_normals,
        }
    }

    pub fn with_normals(mut self, base: &MeshAsset, target_normals: &[Vec3]) -> Self {
        self.delta_normals = base
            .vertices
            .iter()
            .zip(target_normals.iter())
            .map(|(v, &tn)| tn - v.normal)
            .collect();
        self
    }
}

#[derive(Debug, Clone)]
pub struct MorphState {
    pub targets: Vec<MorphTarget>,
    pub weights: Vec<f64>,
}

impl MorphState {
    pub fn new(targets: Vec<MorphTarget>) -> Self {
        let n = targets.len();
        Self {
            targets,
            weights: vec![0.0; n],
        }
    }

    pub fn set_weight(&mut self, index: usize, weight: f64) {
        if index < self.weights.len() {
            self.weights[index] = weight.clamp(0.0, 1.0);
        }
    }

    pub fn apply(&self, base: &MeshAsset) -> Vec<Vertex> {
        let mut vertices = base.vertices.clone();
        for (target, &weight) in self.targets.iter().zip(self.weights.iter()) {
            if weight < f64::EPSILON {
                continue;
            }
            for (i, v) in vertices.iter_mut().enumerate() {
                if i < target.delta_positions.len() {
                    v.position += target.delta_positions[i] * weight;
                }
                if i < target.delta_normals.len() {
                    v.normal += target.delta_normals[i] * weight;
                }
            }
        }
        for v in &mut vertices {
            let len = v.normal.length();
            if len > f64::EPSILON {
                v.normal = v.normal * (1.0 / len);
            }
        }
        vertices
    }

    pub fn active_count(&self) -> usize {
        self.weights.iter().filter(|&&w| w > f64::EPSILON).count()
    }
}
