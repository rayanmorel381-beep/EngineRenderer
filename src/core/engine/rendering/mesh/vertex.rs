//! Vertex layout, mesh descriptor, and descriptor factory functions.
//!
//! [`Vertex`] is the fundamental per-vertex data record used across
//! the entire rendering pipeline.

use crate::core::engine::rendering::raytracing::Vec3;

/// A single vertex with position, normal, texture coordinate, and
/// tangent.
#[derive(Debug, Clone, Copy)]
pub struct Vertex {
    /// World-space or object-space position.
    pub position: Vec3,
    /// Surface normal (should be unit-length).
    pub normal: Vec3,
    /// Texture coordinate `(u, v)` packed as `(x, y)`.
    pub uv: Vec3,
    /// Tangent vector for normal-mapping (unit-length, or `ZERO` when
    /// uncomputed).
    pub tangent: Vec3,
}

impl Vertex {
    /// Creates a new vertex with an explicit tangent.
    pub fn new(position: Vec3, normal: Vec3, uv: Vec3, tangent: Vec3) -> Self {
        Self {
            position,
            normal,
            uv,
            tangent,
        }
    }

    /// Returns the UV as a `(u, v)` tuple for triangle construction.
    pub fn uv_tuple(&self) -> (f64, f64) {
        (self.uv.x, self.uv.y)
    }
}

/// High-level statistics about a mesh asset.
#[derive(Debug, Clone, Copy)]
pub struct MeshDescriptor {
    /// Number of vertices in the mesh.
    pub vertex_count: usize,
    /// Number of triangles (`index_count / 3`).
    pub triangle_count: usize,
    /// Radius of the bounding sphere centred at the centroid.
    pub bounding_radius: f64,
}

/// Creates a [`MeshDescriptor`] for a UV-sphere with the given
/// parameters.
pub fn uv_sphere(stacks: usize, slices: usize, radius: f64) -> MeshDescriptor {
    let verts = (stacks + 1) * (slices + 1);
    let tris = stacks * slices * 2;
    MeshDescriptor {
        vertex_count: verts,
        triangle_count: tris,
        bounding_radius: radius,
    }
}

/// Creates a [`MeshDescriptor`] for a unit cube.
pub fn cube() -> MeshDescriptor {
    MeshDescriptor {
        vertex_count: 24,
        triangle_count: 12,
        bounding_radius: 3.0_f64.sqrt() * 0.5,
    }
}

/// Creates a [`MeshDescriptor`] for a subdivided ground plane.
pub fn plane(subdivisions: usize, half_extent: f64) -> MeshDescriptor {
    let n = subdivisions + 1;
    MeshDescriptor {
        vertex_count: n * n,
        triangle_count: subdivisions * subdivisions * 2,
        bounding_radius: half_extent * 2.0_f64.sqrt(),
    }
}

/// Computes the geometric density (triangles per unit area) of a
/// mesh described by `descriptor` with the given `surface_area`.
pub fn geometric_density(descriptor: &MeshDescriptor, surface_area: f64) -> f64 {
    if surface_area < 1e-12 {
        return 0.0;
    }
    descriptor.triangle_count as f64 / surface_area
}
