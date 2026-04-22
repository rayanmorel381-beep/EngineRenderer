use crate::api::objects::scene_object::SceneObject;

use super::engine_api::EngineApi;

/// Configuration manager re-export.
pub use crate::core::coremanager::config_manager::{ConfigManager, ConfigPreset};
/// Core LOD manager re-export.
pub use crate::core::coremanager::lod_manager::LodManager as CoreLodManager;
/// Network manager re-export.
pub use crate::core::coremanager::network_manager::NetworkManager;
/// Resource tracker re-export.
pub use crate::core::coremanager::resource_manager::ResourceTracker;
/// Engine logger re-export.
pub use crate::core::debug::logger::EngineLogger;
/// Adaptive tile scheduler re-export.
pub use crate::core::scheduler::adaptive::TileScheduler;

impl EngineApi {
    // -- object factories ---------------------------------------------------

    /// Creates a star object.
    pub fn star(&self, center: [f64; 3], radius: f64) -> SceneObject {
        SceneObject::star(center, radius)
    }

    /// Creates a rocky planet object.
    pub fn planet(&self, center: [f64; 3], radius: f64) -> SceneObject {
        SceneObject::planet(center, radius)
    }

    /// Creates an ocean planet object.
    pub fn ocean_planet(&self, center: [f64; 3], radius: f64) -> SceneObject {
        SceneObject::ocean_planet(center, radius)
    }

    /// Creates an icy planet object.
    pub fn ice_planet(&self, center: [f64; 3], radius: f64) -> SceneObject {
        SceneObject::ice_planet(center, radius)
    }

    /// Creates a moon object.
    pub fn moon(&self, center: [f64; 3], radius: f64) -> SceneObject {
        SceneObject::moon(center, radius)
    }

    /// Creates a black hole object.
    pub fn black_hole(&self, center: [f64; 3], radius: f64) -> SceneObject {
        SceneObject::black_hole(center, radius)
    }

    /// Creates a procedural solar system composite.
    pub fn solar_system(&self, center: [f64; 3], star_radius: f64, planets: usize) -> SceneObject {
        SceneObject::solar_system(center, star_radius, planets)
    }

    /// Creates a procedural tree composite.
    pub fn tree(&self, base: [f64; 3], height: f64) -> SceneObject {
        SceneObject::tree(base, height)
    }

    /// Creates a procedural house composite.
    pub fn house(&self, center: [f64; 3], size: f64) -> SceneObject {
        SceneObject::house(center, size)
    }

    /// Creates a procedural car composite.
    pub fn car(&self, center: [f64; 3], length: f64) -> SceneObject {
        SceneObject::car(center, length)
    }

    /// Creates a sphere with a named material.
    pub fn custom_sphere(&self, center: [f64; 3], radius: f64, material_name: &str) -> SceneObject {
        SceneObject::sphere(center, radius, material_name)
    }

    /// Creates a sphere with an explicit base color and roughness.
    pub fn colored_sphere(&self, center: [f64; 3], radius: f64, rgb: [f64; 3], roughness: f64) -> SceneObject {
        SceneObject::colored_sphere(center, radius, rgb, roughness)
    }

    /// Creates a wide ground plane object.
    pub fn ground_plane(&self, y: f64, material_name: &str) -> SceneObject {
        SceneObject::ground_plane(y, material_name)
    }
}
