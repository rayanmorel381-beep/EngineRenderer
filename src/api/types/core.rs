use std::path::PathBuf;

/// Output image format.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OutputFormat {
    /// Portable Pixmap output.
    #[default]
    Ppm,
    /// PNG output.
    Png,
    /// OpenEXR output.
    Exr,
}

impl OutputFormat {
    /// Returns the queue extension for the selected format.
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

/// Render quality profile.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Quality {
    /// Fast preview quality.
    Preview,
    /// HD still quality.
    Hd,
    /// Production reference quality.
    Production,
}

/// Input request used to render a frame.
#[derive(Debug, Clone)]
pub struct RenderRequest {
    /// Output width.
    pub width: usize,
    /// Output height.
    pub height: usize,
    /// Render quality profile.
    pub quality: Quality,
    /// Output directory.
    pub output_dir: PathBuf,
    /// Output queue name.
    pub file_name: String,
}

impl RenderRequest {
    /// Returns the HD preset request.
    pub fn hd() -> Self {
        Self {
            width: 1920,
            height: 1080,
            quality: Quality::Hd,
            output_dir: PathBuf::from("output"),
            file_name: "ai_render.ppm".to_string(),
        }
    }

    /// Returns the production preset request.
    pub fn production() -> Self {
        Self {
            width: 3840,
            height: 2160,
            quality: Quality::Production,
            output_dir: PathBuf::from("output"),
            file_name: "ai_render_production.ppm".to_string(),
        }
    }

    /// Returns the preview preset request.
    pub fn preview() -> Self {
        Self {
            width: 1280,
            height: 720,
            quality: Quality::Preview,
            output_dir: PathBuf::from("output"),
            file_name: "ai_render_preview.ppm".to_string(),
        }
    }

    /// Returns a copy with clamped output resolution.
    pub fn with_resolution(mut self, width: usize, height: usize) -> Self {
        self.width = width.clamp(64, 3840);
        self.height = height.clamp(64, 2160);
        self
    }

    /// Returns a copy with a different quality profile.
    pub fn with_quality(mut self, quality: Quality) -> Self {
        self.quality = quality;
        self
    }

    /// Returns a copy with a custom output directory and queue name.
    pub fn with_output(mut self, dir: impl Into<PathBuf>, name: impl Into<String>) -> Self {
        self.output_dir = dir.into();
        self.file_name = name.into();
        self
    }

    /// Computes width-to-height aspect ratio.
    pub fn aspect_ratio(&self) -> f64 {
        self.width as f64 / self.height as f64
    }

    /// Returns the full output path.
    pub fn output_path(&self) -> PathBuf {
        self.output_dir.join(&self.file_name)
    }

    /// Returns a copy with the queue extension set from the format.
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

/// Render output summary returned by the API.
#[derive(Debug, Clone)]
pub struct RenderResult {
    /// Final output path.
    pub output_path: PathBuf,
    /// Rendered image width.
    pub width: usize,
    /// Rendered image height.
    pub height: usize,
    /// Total rendered pixel count.
    pub rendered_pixels: usize,
    /// Render duration in milliseconds.
    pub duration_ms: u128,
    /// Scene object count.
    pub object_count: usize,
    /// Scene triangle count.
    pub triangle_count: usize,
    /// Average luminance.
    pub average_luminance: f64,
    /// Minimum luminance.
    pub min_luminance: f64,
    /// Maximum luminance.
    pub max_luminance: f64,
    /// Estimated samples per pixel.
    pub estimated_samples_per_pixel: usize,
}

// ---------------------------------------------------------------------------
// Descriptors
// ---------------------------------------------------------------------------

/// Material descriptor used in declarative scene definitions.
#[derive(Debug, Clone, Copy)]
pub struct MaterialDesc {
    /// Material name.
    pub name: &'static str,
    /// Material category.
    pub category: &'static str,
    /// Base albedo color.
    pub albedo: [f64; 3],
    /// Surface roughness.
    pub roughness: f64,
    /// Metallic factor.
    pub metallic: f64,
    /// Emissive capability flag.
    pub emissive: bool,
}

/// Sphere-like object descriptor used by high-level scene APIs.
#[derive(Debug, Clone, Copy)]
pub struct ObjectDesc {
    /// Object position.
    pub position: [f64; 3],
    /// Object radius.
    pub radius: f64,
    /// Material preset name.
    pub material_name: &'static str,
}

/// Light descriptor used by high-level scene APIs.
#[derive(Debug, Clone, Copy)]
pub struct LightDesc {
    /// Light kind.
    pub kind: LightKind,
    /// Light color.
    pub color: [f64; 3],
    /// Light intensity.
    pub intensity: f64,
}

/// Supported light kinds for declarative scene definitions.
#[derive(Debug, Clone, Copy)]
pub enum LightKind {
    /// Directional light with a direction vector.
    Directional {
        /// Direction vector.
        direction: [f64; 3],
    },
    /// Area light with position and rectangular size.
    Area {
        /// World-space position.
        position: [f64; 3],
        /// Rectangle size.
        size: [f64; 2],
    },
}

/// Camera descriptor used by scene builders.
#[derive(Debug, Clone, Copy)]
pub struct CameraDesc {
    /// Camera eye position.
    pub eye: [f64; 3],
    /// Camera target position.
    pub target: [f64; 3],
    /// Vertical field of view in degrees.
    pub fov_degrees: f64,
    /// Lens aperture value.
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
