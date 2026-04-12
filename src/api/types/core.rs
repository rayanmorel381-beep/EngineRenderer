use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OutputFormat {
    #[default]
    Ppm,
    Png,
    Exr,
}

impl OutputFormat {
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
    Preview,
    Hd,
    Production,
}

/// A render request — resolution, quality, output location.
#[derive(Debug, Clone)]
pub struct RenderRequest {
    pub width: usize,
    pub height: usize,
    pub quality: Quality,
    pub output_dir: PathBuf,
    pub file_name: String,
}

impl RenderRequest {
    pub fn hd() -> Self {
        Self {
            width: 1920,
            height: 1080,
            quality: Quality::Hd,
            output_dir: PathBuf::from("output"),
            file_name: "ai_render.ppm".to_string(),
        }
    }

    pub fn production() -> Self {
        Self {
            width: 2560,
            height: 1440,
            quality: Quality::Production,
            output_dir: PathBuf::from("output"),
            file_name: "ai_render_production.ppm".to_string(),
        }
    }

    pub fn preview() -> Self {
        Self {
            width: 1280,
            height: 720,
            quality: Quality::Preview,
            output_dir: PathBuf::from("output"),
            file_name: "ai_render_preview.ppm".to_string(),
        }
    }

    pub fn with_resolution(mut self, width: usize, height: usize) -> Self {
        self.width = width.clamp(64, 3840);
        self.height = height.clamp(64, 2160);
        self
    }

    pub fn with_quality(mut self, quality: Quality) -> Self {
        self.quality = quality;
        self
    }

    pub fn with_output(mut self, dir: impl Into<PathBuf>, name: impl Into<String>) -> Self {
        self.output_dir = dir.into();
        self.file_name = name.into();
        self
    }

    pub fn aspect_ratio(&self) -> f64 {
        self.width as f64 / self.height as f64
    }

    pub fn output_path(&self) -> PathBuf {
        self.output_dir.join(&self.file_name)
    }

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
    pub output_path: PathBuf,
    pub width: usize,
    pub height: usize,
    pub rendered_pixels: usize,
    pub duration_ms: u128,
    pub object_count: usize,
    pub triangle_count: usize,
    pub average_luminance: f64,
    pub min_luminance: f64,
    pub max_luminance: f64,
    pub estimated_samples_per_pixel: usize,
}

// ---------------------------------------------------------------------------
// Descriptors
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy)]
pub struct MaterialDesc {
    pub name: &'static str,
    pub category: &'static str,
    pub albedo: [f64; 3],
    pub roughness: f64,
    pub metallic: f64,
    pub emissive: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct ObjectDesc {
    pub position: [f64; 3],
    pub radius: f64,
    pub material_name: &'static str,
}

#[derive(Debug, Clone, Copy)]
pub struct LightDesc {
    pub kind: LightKind,
    pub color: [f64; 3],
    pub intensity: f64,
}

#[derive(Debug, Clone, Copy)]
pub enum LightKind {
    Directional { direction: [f64; 3] },
    Area { position: [f64; 3], size: [f64; 2] },
}

/// Camera placement description.
#[derive(Debug, Clone, Copy)]
pub struct CameraDesc {
    pub eye: [f64; 3],
    pub target: [f64; 3],
    pub fov_degrees: f64,
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
