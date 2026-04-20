
use crate::core::engine::rendering::raytracing::Vec3;

#[derive(Debug, Clone, Copy)]
pub struct Vertex {
    pub position: Vec3,
    pub normal: Vec3,
    pub uv: Vec3,
    pub tangent: Vec3,
}

impl Vertex {
    pub fn new(position: Vec3, normal: Vec3, uv: Vec3, tangent: Vec3) -> Self {
        Self {
            position,
            normal,
            uv,
            tangent,
        }
    }

    pub fn uv_tuple(&self) -> (f64, f64) {
        (self.uv.x, self.uv.y)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct MeshDescriptor {
    pub vertex_count: usize,
    pub triangle_count: usize,
    pub bounding_radius: f64,
}

pub fn uv_sphere(stacks: usize, slices: usize, radius: f64) -> MeshDescriptor {
    let verts = (stacks + 1) * (slices + 1);
    let tris = stacks * slices * 2;
    MeshDescriptor {
        vertex_count: verts,
        triangle_count: tris,
        bounding_radius: radius,
    }
}

pub fn cube() -> MeshDescriptor {
    MeshDescriptor {
        vertex_count: 24,
        triangle_count: 12,
        bounding_radius: 3.0_f64.sqrt() * 0.5,
    }
}

pub fn plane(subdivisions: usize, half_extent: f64) -> MeshDescriptor {
    let n = subdivisions + 1;
    MeshDescriptor {
        vertex_count: n * n,
        triangle_count: subdivisions * subdivisions * 2,
        bounding_radius: half_extent * 2.0_f64.sqrt(),
    }
}

pub fn geometric_density(descriptor: &MeshDescriptor, surface_area: f64) -> f64 {
    if surface_area < 1e-12 {
        return 0.0;
    }
    descriptor.triangle_count as f64 / surface_area
}
