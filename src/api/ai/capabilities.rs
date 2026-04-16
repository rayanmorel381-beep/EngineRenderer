/// Engine capability introspection.
///
/// An AI agent can call [`discover()`] to learn what the engine supports
/// *before* building a scene or issuing a render request. Every field is
/// a plain `Vec<String>` or scalar so it serialises trivially.

#[derive(Debug, Clone)]
pub struct Capabilities {
    /// Types d'objets supportés par le moteur (sphères, triangles, corps célestes…).
    pub object_types: Vec<&'static str>,
    /// Préréglages de matériaux disponibles dans le catalogue.
    pub material_presets: Vec<&'static str>,
    /// Niveaux de qualité supportés (`"preview"`, `"hd"`, `"production"`).
    pub quality_levels: Vec<&'static str>,
    /// Résolution maximale en pixels `(largeur, hauteur)`.
    pub max_resolution: (usize, usize),
    /// Nombre maximum de rebonds de rayons (profondeur de récursion).
    pub max_bounces: u32,
    /// Nombre maximum d'échantillons par pixel.
    pub max_samples_per_pixel: u32,
    /// Indique si les effets volumétriques sont disponibles.
    pub supports_volumetrics: bool,
    /// Indique si les lumières surfaciques sont disponibles.
    pub supports_area_lights: bool,
    /// Indique si la profondeur de champ est disponible.
    pub supports_depth_of_field: bool,
    /// Indique si le flou de mouvement est disponible.
    pub supports_motion_blur: bool,
    /// Indique si les textures procédurales sont disponibles.
    pub supports_procedural_textures: bool,
    /// Format de sortie des images rendu (`"ppm"`, etc.).
    pub output_format: &'static str,
}

/// Détecte et retourne les capacités courantes du moteur de rendu.
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
