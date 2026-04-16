use std::path::PathBuf;

/// Format de sortie pour les images rendues.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OutputFormat {
    /// Format PPM.
    #[default]
    Ppm,
    /// Format PNG.
    Png,
    /// Format OpenEXR.
    Exr,
}

impl OutputFormat {
    /// Retourne l'extension de fichier associée.
    pub fn extension(self) -> &'static str {
        match self {
            Self::Ppm => "ppm",
            Self::Png => "png",
            Self::Exr => "exr",
        }
    }
}

// ---------------------------------------------------------------------------
// Quality / render request / result
// ---------------------------------------------------------------------------

/// Quality tier for a render request.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Quality {
    /// Qualité preview.
    Preview,
    /// Qualité HD.
    Hd,
    /// Qualité production.
    Production,
}

/// A render request — resolution, quality, output location.
#[derive(Debug, Clone)]
pub struct RenderRequest {
    /// Largeur cible.
    pub width: usize,
    /// Hauteur cible.
    pub height: usize,
    /// Niveau de qualité.
    pub quality: Quality,
    /// Dossier de sortie.
    pub output_dir: PathBuf,
    /// Nom de fichier de sortie.
    pub file_name: String,
}

impl RenderRequest {
    /// Crée une requête HD.
    pub fn hd() -> Self {
        Self {
            width: 1920,
            height: 1080,
            quality: Quality::Hd,
            output_dir: PathBuf::from("output"),
            file_name: "ai_render.ppm".to_string(),
        }
    }

    /// Crée une requête production.
    pub fn production() -> Self {
        Self {
            width: 2560,
            height: 1440,
            quality: Quality::Production,
            output_dir: PathBuf::from("output"),
            file_name: "ai_render_production.ppm".to_string(),
        }
    }

    /// Crée une requête preview.
    pub fn preview() -> Self {
        Self {
            width: 1280,
            height: 720,
            quality: Quality::Preview,
            output_dir: PathBuf::from("output"),
            file_name: "ai_render_preview.ppm".to_string(),
        }
    }

    /// Surcharge la résolution.
    pub fn with_resolution(mut self, width: usize, height: usize) -> Self {
        self.width = width.clamp(64, 3840);
        self.height = height.clamp(64, 2160);
        self
    }

    /// Surcharge la qualité.
    pub fn with_quality(mut self, quality: Quality) -> Self {
        self.quality = quality;
        self
    }

    /// Surcharge la sortie (dossier + nom).
    pub fn with_output(mut self, dir: impl Into<PathBuf>, name: impl Into<String>) -> Self {
        self.output_dir = dir.into();
        self.file_name = name.into();
        self
    }

    /// Retourne le ratio d'aspect.
    pub fn aspect_ratio(&self) -> f64 {
        self.width as f64 / self.height as f64
    }

    /// Retourne le chemin complet de sortie.
    pub fn output_path(&self) -> PathBuf {
        self.output_dir.join(&self.file_name)
    }

    /// Force un format de sortie via l'extension.
    pub fn with_format(mut self, format: OutputFormat) -> Self {
        let stem = std::path::Path::new(&self.file_name)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("render")
            .to_string();
        self.file_name = format!("{}.{}", stem, format.extension());
        self
    }
}

impl Default for RenderRequest {
    fn default() -> Self {
        Self::hd()
    }
}

/// Result of a completed render.
#[derive(Debug, Clone)]
pub struct RenderResult {
    /// Chemin du fichier rendu.
    pub output_path: PathBuf,
    /// Largeur rendue.
    pub width: usize,
    /// Hauteur rendue.
    pub height: usize,
    /// Pixels rendus.
    pub rendered_pixels: usize,
    /// Durée totale en ms.
    pub duration_ms: u128,
    /// Nombre d'objets scène.
    pub object_count: usize,
    /// Nombre de triangles scène.
    pub triangle_count: usize,
    /// Luminance moyenne.
    pub average_luminance: f64,
    /// Luminance minimale.
    pub min_luminance: f64,
    /// Luminance maximale.
    pub max_luminance: f64,
    /// SPP estimé.
    pub estimated_samples_per_pixel: usize,
}

// ---------------------------------------------------------------------------
// Descriptors
// ---------------------------------------------------------------------------

/// Descripteur compact de matériau.
#[derive(Debug, Clone, Copy)]
pub struct MaterialDesc {
    /// Nom du matériau.
    pub name: &'static str,
    /// Catégorie du matériau.
    pub category: &'static str,
    /// Albédo linéaire.
    pub albedo: [f64; 3],
    /// Rugosité.
    pub roughness: f64,
    /// Métallicité.
    pub metallic: f64,
    /// Présence d'émission.
    pub emissive: bool,
}

/// Descripteur compact d'objet sphérique.
#[derive(Debug, Clone, Copy)]
pub struct ObjectDesc {
    /// Position monde.
    pub position: [f64; 3],
    /// Rayon.
    pub radius: f64,
    /// Nom du matériau.
    pub material_name: &'static str,
}

/// Descripteur compact de lumière.
#[derive(Debug, Clone, Copy)]
pub struct LightDesc {
    /// Type de lumière.
    pub kind: LightKind,
    /// Couleur.
    pub color: [f64; 3],
    /// Intensité.
    pub intensity: f64,
}

/// Variantes de sources lumineuses supportées.
#[derive(Debug, Clone, Copy)]
pub enum LightKind {
    /// Lumière directionnelle.
    Directional {
        /// Direction normalisée de la lumière.
        direction: [f64; 3],
    },
    /// Lumière surfacique rectangulaire.
    Area {
        /// Position centrale de la source.
        position: [f64; 3],
        /// Taille du rectangle émetteur.
        size: [f64; 2],
    },
}

/// Camera placement description.
#[derive(Debug, Clone, Copy)]
pub struct CameraDesc {
    /// Position caméra.
    pub eye: [f64; 3],
    /// Cible caméra.
    pub target: [f64; 3],
    /// FOV vertical.
    pub fov_degrees: f64,
    /// Ouverture de lentille.
    pub aperture: f64,
}

impl Default for CameraDesc {
    fn default() -> Self {
        Self {
            eye: [12.0, 6.0, 12.0],
            target: [0.0, 0.0, 0.0],
            fov_degrees: 42.0,
            aperture: 0.0,
        }
    }
}
