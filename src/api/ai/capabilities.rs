/// Engine capability introspection.
///
/// An AI agent can call [`discover()`] to learn what the engine supports
/// *before* building a scene or issuing a render request. Every field is
/// a plain `Vec<String>` or scalar so it serialises trivially.

#[derive(Debug, Clone)]
pub struct Capabilities {
    pub object_types: Vec<&'static str>,
    pub material_presets: Vec<&'static str>,
    pub quality_levels: Vec<&'static str>,
    pub max_resolution: (usize, usize),
    pub max_bounces: u32,
    pub max_samples_per_pixel: u32,
    pub supports_volumetrics: bool,
    pub supports_area_lights: bool,
    pub supports_depth_of_field: bool,
    pub supports_motion_blur: bool,
    pub supports_procedural_textures: bool,
    pub output_format: &'static str,
}

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
