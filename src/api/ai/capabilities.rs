
#[derive(Debug, Clone)]
/// Declarative capabilities exposed for AI clients.
pub struct Capabilities {
    /// Supported high-level object families.
    pub object_types: Vec<&'static str>,
    /// Built-in material preset identifiers.
    pub material_presets: Vec<&'static str>,
    /// Available quality levels.
    pub quality_levels: Vec<&'static str>,
    /// Maximum supported output resolution `(width, height)`.
    pub max_resolution: (usize, usize),
    /// Maximum supported bounce count.
    pub max_bounces: u32,
    /// Maximum supported samples-per-pixel value.
    pub max_samples_per_pixel: u32,
    /// Whether volumetric effects are supported.
    pub supports_volumetrics: bool,
    /// Whether area lights are supported.
    pub supports_area_lights: bool,
    /// Whether depth-of-field is supported.
    pub supports_depth_of_field: bool,
    /// Whether motion blur is supported.
    pub supports_motion_blur: bool,
    /// Whether procedural textures are supported.
    pub supports_procedural_textures: bool,
    /// Default output format identifier.
    pub output_format: &'static str,
}

/// Discovers the current AI-facing feature set.
pub fn discover() -> Capabilities {
    Capabilities {
        object_types: vec![
            "sphere",
            "triangle",
            "celestial_body",
            "mesh_flat_triangle",
        ],
        material_presets: vec![
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
            "custom",
        ],
        quality_levels: vec!["preview", "hd", "production"],
        max_resolution: (3840, 2160),
        max_bounces: 16,
        max_samples_per_pixel: 64,
        supports_volumetrics: true,
        supports_area_lights: true,
        supports_depth_of_field: true,
        supports_motion_blur: true,
        supports_procedural_textures: true,
        output_format: "ppm",
    }
}
