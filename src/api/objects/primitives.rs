use crate::api::materials::catalog::MaterialCatalog;
use crate::api::objects::SceneObject;
use crate::core::engine::rendering::raytracing::Vec3;

pub use crate::core::engine::rendering::mesh::asset::MeshAsset;
pub use crate::core::engine::rendering::mesh::generators::{ground_plane, icosphere, torus, unit_cube};
pub use crate::core::engine::rendering::mesh::operations::{
    compute_tangents, recalculate_normals, subdivide,
};
pub use crate::core::engine::rendering::mesh::vertex::{cube, geometric_density, plane, uv_sphere};
pub use crate::core::engine::rendering::preprocessing::bvh_builder::Aabb;
pub use crate::core::engine::rendering::raytracing::{Material, Ray, Scene, Sphere, Triangle};

impl SceneObject {
    pub fn star(center: [f64; 3], radius: f64) -> Self {
        Self::sphere(center, radius, "stellar_surface")
    }

    pub fn planet(center: [f64; 3], radius: f64) -> Self {
        Self::sphere(center, radius, "rocky_world")
    }

    pub fn ocean_planet(center: [f64; 3], radius: f64) -> Self {
        Self::sphere(center, radius, "ocean_world")
    }

    pub fn ice_planet(center: [f64; 3], radius: f64) -> Self {
        Self::sphere(center, radius, "icy_world")
    }

    pub fn moon(center: [f64; 3], radius: f64) -> Self {
        Self::sphere(center, radius, "metallic_moon")
    }

    pub fn lush_planet(center: [f64; 3], radius: f64) -> Self {
        Self::sphere(center, radius, "lush_planet")
    }

    pub fn black_hole(center: [f64; 3], radius: f64) -> Self {
        Self::sphere(center, radius, "event_horizon")
    }

    pub fn ground_plane(y: f64, material_name: &str) -> Self {
        Self::sphere([0.0, y - 200.0, 0.0], 200.0, material_name)
    }

    pub fn colored_sphere(center: [f64; 3], radius: f64, rgb: [f64; 3], roughness: f64) -> Self {
        let mat = MaterialCatalog.custom(rgb, roughness, 0.0, 0.04, [0.0, 0.0, 0.0]);
        Self::Sphere {
            center: Vec3::new(center[0], center[1], center[2]),
            radius: radius.max(0.01),
            material: mat,
        }
    }
}
