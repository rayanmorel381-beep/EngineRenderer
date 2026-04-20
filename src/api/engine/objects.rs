use crate::api::objects::scene_object::SceneObject;

use super::engine_api::EngineApi;

pub use crate::core::coremanager::config_manager::{ConfigManager, ConfigPreset};
pub use crate::core::coremanager::lod_manager::LodManager as CoreLodManager;
pub use crate::core::coremanager::network_manager::NetworkManager;
pub use crate::core::coremanager::resource_manager::ResourceTracker;
pub use crate::core::debug::logger::EngineLogger;
pub use crate::core::scheduler::adaptive::TileScheduler;

impl EngineApi {
    // -- object factories ---------------------------------------------------

    pub fn star(&self, center: [f64; 3], radius: f64) -> SceneObject {
        SceneObject::star(center, radius)
    }

    pub fn planet(&self, center: [f64; 3], radius: f64) -> SceneObject {
        SceneObject::planet(center, radius)
    }

    pub fn ocean_planet(&self, center: [f64; 3], radius: f64) -> SceneObject {
        SceneObject::ocean_planet(center, radius)
    }

    pub fn ice_planet(&self, center: [f64; 3], radius: f64) -> SceneObject {
        SceneObject::ice_planet(center, radius)
    }

    pub fn moon(&self, center: [f64; 3], radius: f64) -> SceneObject {
        SceneObject::moon(center, radius)
    }

    pub fn black_hole(&self, center: [f64; 3], radius: f64) -> SceneObject {
        SceneObject::black_hole(center, radius)
    }

    pub fn solar_system(&self, center: [f64; 3], star_radius: f64, planets: usize) -> SceneObject {
        SceneObject::solar_system(center, star_radius, planets)
    }

    pub fn tree(&self, base: [f64; 3], height: f64) -> SceneObject {
        SceneObject::tree(base, height)
    }

    pub fn house(&self, center: [f64; 3], size: f64) -> SceneObject {
        SceneObject::house(center, size)
    }

    pub fn car(&self, center: [f64; 3], length: f64) -> SceneObject {
        SceneObject::car(center, length)
    }

    pub fn custom_sphere(&self, center: [f64; 3], radius: f64, material_name: &str) -> SceneObject {
        SceneObject::sphere(center, radius, material_name)
    }

    pub fn colored_sphere(&self, center: [f64; 3], radius: f64, rgb: [f64; 3], roughness: f64) -> SceneObject {
        SceneObject::colored_sphere(center, radius, rgb, roughness)
    }

    pub fn ground_plane(&self, y: f64, material_name: &str) -> SceneObject {
        SceneObject::ground_plane(y, material_name)
    }
}
