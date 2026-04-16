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

    /// Crée une étoile lumineuse.
    pub fn star(&self, center: [f64; 3], radius: f64) -> SceneObject {
        SceneObject::star(center, radius)
    }

    /// Crée une planète rocheuse.
    pub fn planet(&self, center: [f64; 3], radius: f64) -> SceneObject {
        SceneObject::planet(center, radius)
    }

    /// Crée une planète océanique.
    pub fn ocean_planet(&self, center: [f64; 3], radius: f64) -> SceneObject {
        SceneObject::ocean_planet(center, radius)
    }

    /// Crée une planète glacée.
    pub fn ice_planet(&self, center: [f64; 3], radius: f64) -> SceneObject {
        SceneObject::ice_planet(center, radius)
    }

    /// Crée une lune métallique.
    pub fn moon(&self, center: [f64; 3], radius: f64) -> SceneObject {
        SceneObject::moon(center, radius)
    }

    /// Crée un trou noir.
    pub fn black_hole(&self, center: [f64; 3], radius: f64) -> SceneObject {
        SceneObject::black_hole(center, radius)
    }

    /// Crée un système solaire procédural avec `planets` planètes autour d'une étoile de rayon `star_radius`.
    pub fn solar_system(&self, center: [f64; 3], star_radius: f64, planets: usize) -> SceneObject {
        SceneObject::solar_system(center, star_radius, planets)
    }

    /// Crée un arbre procédural.
    pub fn tree(&self, base: [f64; 3], height: f64) -> SceneObject {
        SceneObject::tree(base, height)
    }

    /// Crée une maison procédurale.
    pub fn house(&self, center: [f64; 3], size: f64) -> SceneObject {
        SceneObject::house(center, size)
    }

    /// Crée un véhicule procédural.
    pub fn car(&self, center: [f64; 3], length: f64) -> SceneObject {
        SceneObject::car(center, length)
    }

    /// Crée une sphère avec le matériau nommé du catalogue.
    pub fn custom_sphere(&self, center: [f64; 3], radius: f64, material_name: &str) -> SceneObject {
        SceneObject::sphere(center, radius, material_name)
    }

    /// Crée une sphère avec une couleur et une rugosité personnalisées.
    pub fn colored_sphere(&self, center: [f64; 3], radius: f64, rgb: [f64; 3], roughness: f64) -> SceneObject {
        SceneObject::colored_sphere(center, radius, rgb, roughness)
    }

    /// Crée un plan au sol à hauteur `y` avec le matériau nommé.
    pub fn ground_plane(&self, y: f64, material_name: &str) -> SceneObject {
        SceneObject::ground_plane(y, material_name)
    }
}
