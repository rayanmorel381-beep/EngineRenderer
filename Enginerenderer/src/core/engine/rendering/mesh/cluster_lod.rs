use super::asset::MeshAsset;
use super::vertex::Vertex;
use crate::core::engine::rendering::raytracing::Vec3;

pub const CLUSTER_SIZE: usize = 128;

#[derive(Debug, Clone)]
pub struct MeshCluster {
    pub triangle_start: usize,
    pub triangle_count: usize,
    pub bounds_center: Vec3,
    pub bounds_radius: f64,
    pub lod_error: f64,
}

#[derive(Debug, Clone)]
pub struct ClusteredMesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<usize>,
    pub clusters: Vec<MeshCluster>,
}

impl ClusteredMesh {
    pub fn build(asset: &MeshAsset) -> Self {
        let tri_count = asset.indices.len() / 3;
        let mut clusters = Vec::new();
        let cluster_tri_count = CLUSTER_SIZE;

        let mut start = 0;
        while start < tri_count {
            let count = (cluster_tri_count).min(tri_count - start);
            let mut bounds_min = Vec3::new(f64::MAX, f64::MAX, f64::MAX);
            let mut bounds_max = Vec3::new(f64::MIN, f64::MIN, f64::MIN);
            for ti in start..start + count {
                for &vi in &asset.indices[ti * 3..ti * 3 + 3] {
                    let p = asset.vertices[vi].position;
                    bounds_min.x = bounds_min.x.min(p.x);
                    bounds_min.y = bounds_min.y.min(p.y);
                    bounds_min.z = bounds_min.z.min(p.z);
                    bounds_max.x = bounds_max.x.max(p.x);
                    bounds_max.y = bounds_max.y.max(p.y);
                    bounds_max.z = bounds_max.z.max(p.z);
                }
            }
            let center = (bounds_min + bounds_max) * 0.5;
            let radius = (bounds_max - bounds_min).length() * 0.5;
            clusters.push(MeshCluster {
                triangle_start: start,
                triangle_count: count,
                bounds_center: center,
                bounds_radius: radius,
                lod_error: 0.0,
            });
            start += count;
        }

        Self {
            vertices: asset.vertices.clone(),
            indices: asset.indices.clone(),
            clusters,
        }
    }

    pub fn simplify(&self, target_tri_count: usize) -> MeshAsset {
        let original_tri = self.indices.len() / 3;
        if target_tri_count >= original_tri {
            return MeshAsset {
                name: String::new(),
                descriptor: crate::core::engine::rendering::mesh::vertex::MeshDescriptor {
                    vertex_count: self.vertices.len(),
                    triangle_count: original_tri,
                    bounding_radius: 1.0,
                },
                vertices: self.vertices.clone(),
                indices: self.indices.clone(),
                preferred_material: None,
                base_translation: Vec3::ZERO,
                base_scale: Vec3::new(1.0, 1.0, 1.0),
                base_rotation: [0.0, 0.0, 0.0, 1.0],
            };
        }

        let ratio = target_tri_count as f64 / original_tri as f64;
        let kept_clusters = ((self.clusters.len() as f64 * ratio).ceil() as usize).max(1);
        let mut new_indices = Vec::new();
        for cluster in self.clusters.iter().take(kept_clusters) {
            let start = cluster.triangle_start * 3;
            let end = (cluster.triangle_start + cluster.triangle_count) * 3;
            new_indices.extend_from_slice(&self.indices[start..end]);
        }

        let new_tri = new_indices.len() / 3;
        MeshAsset {
            name: String::new(),
            descriptor: crate::core::engine::rendering::mesh::vertex::MeshDescriptor {
                vertex_count: self.vertices.len(),
                triangle_count: new_tri,
                bounding_radius: 1.0,
            },
            vertices: self.vertices.clone(),
            indices: new_indices,
            preferred_material: None,
            base_translation: Vec3::ZERO,
            base_scale: Vec3::new(1.0, 1.0, 1.0),
            base_rotation: [0.0, 0.0, 0.0, 1.0],
        }
    }

    pub fn select_lod(&self, screen_error_threshold: f64, camera_dist: f64, object_radius: f64) -> Vec<usize> {
        let projected_error = object_radius / camera_dist.max(f64::EPSILON);
        let mut visible = Vec::new();
        for (i, cluster) in self.clusters.iter().enumerate() {
            let cluster_error = cluster.lod_error + projected_error;
            if cluster_error >= screen_error_threshold || cluster.lod_error == 0.0 {
                visible.push(i);
            }
        }
        visible
    }
}

pub struct ClusterLodChain {
    pub levels: Vec<ClusteredMesh>,
}

impl ClusterLodChain {
    pub fn build(asset: &MeshAsset, lod_count: usize) -> Self {
        let base = ClusteredMesh::build(asset);
        let tri_count = asset.indices.len() / 3;
        let mut levels = vec![base];
        for level in 1..lod_count {
            let factor = 1.0 / (1 << level) as f64;
            let target = ((tri_count as f64 * factor) as usize).max(4);
            let simplified = levels[level - 1].simplify(target);
            levels.push(ClusteredMesh::build(&simplified));
        }
        Self { levels }
    }

    pub fn level_count(&self) -> usize {
        self.levels.len()
    }

    pub fn get_level(&self, camera_dist: f64, object_radius: f64) -> &ClusteredMesh {
        let projected = object_radius / camera_dist.max(f64::EPSILON);
        let level = if projected > 0.05 {
            0
        } else if projected > 0.02 {
            1
        } else if projected > 0.008 {
            2
        } else {
            self.levels.len() - 1
        };
        &self.levels[level.min(self.levels.len() - 1)]
    }
}
