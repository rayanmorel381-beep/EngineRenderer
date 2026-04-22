
use crate::core::engine::rendering::raytracing::{Material, Triangle, Vec3};
use crate::core::engine::rendering::raytracing::primitives::TrianglePatch;

use super::vertex::{MeshDescriptor, Vertex};

/// Mesh asset with geometry, descriptor metadata and default transform.
#[derive(Debug, Clone)]
pub struct MeshAsset {
    /// Asset name.
    pub name: String,
    /// Mesh descriptor statistics.
    pub descriptor: MeshDescriptor,
    /// Vertex buffer.
    pub vertices: Vec<Vertex>,
    /// Triangle index buffer.
    pub indices: Vec<usize>,
    /// Optional preferred surface material.
    pub preferred_material: Option<Material>,
    /// Base translation applied to the mesh.
    pub base_translation: Vec3,
    /// Base non-uniform scale applied to the mesh.
    pub base_scale: Vec3,
    /// Base quaternion rotation [x, y, z, w].
    pub base_rotation: [f64; 4],
}

impl MeshAsset {
    /// Returns preferred material or fallback.
    pub fn material_or(&self, fallback: Material) -> Material {
        self.preferred_material.unwrap_or(fallback)
    }

    /// Returns the descriptor bounding radius.
    pub fn effective_radius(&self) -> f64 {
        self.descriptor.bounding_radius
    }

    /// Computes the centroid of all vertices.
    pub fn centroid(&self) -> Vec3 {
        if self.vertices.is_empty() {
            return Vec3::ZERO;
        }
        let sum = self
            .vertices
            .iter()
            .fold(Vec3::ZERO, |a, v| a + v.position);
        sum * (1.0 / self.vertices.len() as f64)
    }

    /// Computes axis-aligned bounding box in transformed local space.
    pub fn aabb(&self) -> (Vec3, Vec3) {
        let big = f64::MAX;
        let mut mn = Vec3::new(big, big, big);
        let mut mx = Vec3::new(-big, -big, -big);
        for v in &self.vertices {
            let p = v.position * self.base_scale.x + self.base_translation;
            mn = Vec3::new(mn.x.min(p.x), mn.y.min(p.y), mn.z.min(p.z));
            mx = Vec3::new(mx.x.max(p.x), mx.y.max(p.y), mx.z.max(p.z));
        }
        (mn, mx)
    }

    /// Converts indexed mesh geometry to a triangle list.
    pub fn to_triangles(
        &self,
        translation: Vec3,
        scale: f64,
        material: Material,
    ) -> Vec<Triangle> {
        self.indices
            .chunks_exact(3)
            .map(|tri| {
                let a = self.vertices[tri[0]].position * scale + translation;
                let b = self.vertices[tri[1]].position * scale + translation;
                let c = self.vertices[tri[2]].position * scale + translation;

                let na = self.vertices[tri[0]].normal.normalize();
                let nb = self.vertices[tri[1]].normal.normalize();
                let nc = self.vertices[tri[2]].normal.normalize();

                Triangle::new(TrianglePatch {
                    positions: [a, b, c],
                    normals: [na, nb, nc],
                    uvs: [
                        self.vertices[tri[0]].uv_tuple(),
                        self.vertices[tri[1]].uv_tuple(),
                        self.vertices[tri[2]].uv_tuple(),
                    ],
                    material,
                })
            })
            .collect()
    }

    /// Sets preferred material and returns updated asset.
    pub fn with_material(mut self, material: Material) -> Self {
        self.preferred_material = Some(material);
        self
    }

    /// Sets base transform and returns updated asset.
    pub fn with_transform(mut self, translation: Vec3, scale: Vec3, rotation: Option<[f64; 4]>) -> Self {
        self.base_translation = translation;
        self.base_scale = scale;
        if let Some(rot) = rotation {
            self.base_rotation = rot;
        }
        self
    }
}
