//! Render types: quality presets and render reports.

use std::path::PathBuf;

use crate::core::engine::rendering::raytracing::{BvhStats, Vec3};

/// Render quality preset selector.
#[derive(Debug, Clone, Copy)]
pub enum RenderPreset {
    /// Ultra-fast animation (1 SPP, direct lighting, denoised).
    AnimationFast,
    /// Fast preview (low SPP, single bounce).
    PreviewCpu,
    /// High-quality CPU render.
    UltraHdCpu,
    /// Reference render with maximum fidelity.
    ProductionReference,
}

/// Summary produced after a successful render pass.
#[derive(Debug, Clone)]
pub struct RenderReport {
    /// Output image width.
    pub width: usize,
    /// Output image height.
    pub height: usize,
    /// Total rendered pixels.
    pub rendered_pixels: usize,
    /// Wall-clock render time in milliseconds.
    pub duration_ms: u128,
    /// Path where the image was written.
    pub output_path: PathBuf,
    /// Number of scene objects.
    pub object_count: usize,
    /// Number of triangles submitted to the BVH.
    pub triangle_count: usize,
    /// Mean luminance of the final frame.
    pub average_luminance: f64,
    /// Darkest pixel luminance.
    pub min_luminance: f64,
    /// Brightest pixel luminance.
    pub max_luminance: f64,
    /// RGB of the brightest pixel.
    pub brightest_pixel: Vec3,
    /// Samples per pixel used.
    pub estimated_samples_per_pixel: usize,
    /// BVH construction / traversal statistics.
    pub bvh: BvhStats,
}
