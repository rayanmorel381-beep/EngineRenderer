
use std::path::PathBuf;

use crate::core::engine::rendering::raytracing::{BvhStats, Vec3};

#[derive(Debug, Clone, Copy)]
pub enum RenderPreset {
    AnimationFast,
    PreviewCpu,
    UltraHdCpu,
    ProductionReference,
}

#[derive(Debug, Clone)]
pub struct RenderReport {
    pub width: usize,
    pub height: usize,
    pub rendered_pixels: usize,
    pub duration_ms: u128,
    pub output_path: PathBuf,
    pub object_count: usize,
    pub triangle_count: usize,
    pub average_luminance: f64,
    pub min_luminance: f64,
    pub max_luminance: f64,
    pub brightest_pixel: Vec3,
    pub estimated_samples_per_pixel: usize,
    pub bvh: BvhStats,
}
