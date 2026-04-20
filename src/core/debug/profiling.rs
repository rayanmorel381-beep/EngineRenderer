
use crate::core::scheduler::profiling::FrameSummary;

pub fn format_summary(summary: &FrameSummary) -> String {
    format!(
        "[frame {}] sim={}ms scene={}ms total={}ms pixels={} nodes={}",
        summary.frame_index,
        summary.simulation_ms,
        summary.scene_prep_ms,
        summary.total_frame_ms,
        summary.rendered_pixels,
        summary.scene_nodes,
    )
}

pub fn is_over_budget(summary: &FrameSummary, target_ms: f64) -> bool {
    summary.total_frame_ms as f64 > target_ms * 1.20
}

pub fn simulation_ratio(summary: &FrameSummary) -> f64 {
    if summary.total_frame_ms == 0 {
        return 0.0;
    }
    summary.simulation_ms as f64 / summary.total_frame_ms as f64
}
