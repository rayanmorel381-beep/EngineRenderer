use crate::core::engine::rendering::materials::material::MaterialLibrary;
use crate::core::engine::rendering::raytracing::{Material, Vec3};

/// Named material catalogue — maps human-readable string names to engine
/// materials. AI agents can use [`by_name()`] with any preset string from
/// [`Capabilities::material_presets`].
#[derive(Debug)]
pub struct MaterialCatalog;

impl MaterialCatalog {
    /// Look up a material preset by its lowercase name.
    ///
    /// Unrecognised names return a neutral grey diffuse material so that the
    /// render never panics.
    pub fn by_name(&self, name: &str) -> Material {
        match name {
            "stellar_surface" => MaterialLibrary::stellar_surface(),
            "rocky_world" => MaterialLibrary::rocky_world(Vec3::new(0.71, 0.42, 0.26)),
            "ocean_world" => MaterialLibrary::ocean_world(),
            "icy_world" => MaterialLibrary::icy_world(),
            "metallic_moon" => MaterialLibrary::metallic_moon(),
            "automotive_paint" => MaterialLibrary::automotive_paint(Vec3::new(0.72, 0.10, 0.14)),
            "window_glass" => MaterialLibrary::window_glass(),
            "architectural_plaster" => MaterialLibrary::architectural_plaster(),
            "roof_tiles" => MaterialLibrary::roof_tiles(),
            "tree_bark" => MaterialLibrary::tree_bark(),
            "tree_foliage" => MaterialLibrary::tree_foliage(),
            "asphalt" => MaterialLibrary::asphalt(),
            "lush_planet" => MaterialLibrary::lush_planet(),
            "solar_corona" => MaterialLibrary::solar_corona(),
            "accretion_disk" => MaterialLibrary::accretion_disk(),
            "event_horizon" => MaterialLibrary::event_horizon(),
            "rubber_tire" => MaterialLibrary::rubber_tire(),
            _ => self.fallback(),
        }
    }

    /// Parametric rocky world with a custom base colour.
    pub fn rocky_world_colored(&self, r: f64, g: f64, b: f64) -> Material {
        MaterialLibrary::rocky_world(Vec3::new(r, g, b))
    }

    /// Parametric automotive paint with a custom colour.
    pub fn automotive_paint_colored(&self, r: f64, g: f64, b: f64) -> Material {
        MaterialLibrary::automotive_paint(Vec3::new(r, g, b))
    }

    /// Build a fully custom material from raw parameters.
    pub fn custom(
        &self,
        albedo: [f64; 3],
        roughness: f64,
        metallic: f64,
        reflectivity: f64,
        emission: [f64; 3],
    ) -> Material {
        Material::new(
            Vec3::new(albedo[0], albedo[1], albedo[2]),
            roughness,
            metallic,
            reflectivity,
            Vec3::new(emission[0], emission[1], emission[2]),
        )
    }

    /// All preset names, useful for listing in a tool‐use description.
    pub fn all_names(&self) -> &'static [&'static str] {
        &[
            "stellar_surface",
            "rocky_world",
            "ocean_world",
            "icy_world",
            "metallic_moon",
            "automotive_paint",
            "window_glass",
            "architectural_plaster",
            "roof_tiles",
            "tree_bark",
            "tree_foliage",
            "asphalt",
            "lush_planet",
            "solar_corona",
            "accretion_disk",
            "event_horizon",
            "rubber_tire",
        ]
    }

    fn fallback(&self) -> Material {
        Material::new(Vec3::new(0.5, 0.5, 0.5), 0.5, 0.0, 0.04, Vec3::ZERO)
    }
}
