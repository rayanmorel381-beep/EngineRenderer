
use crate::core::scheduler::profiling::FrameSummary;
use crate::core::debug::runtime::RuntimeAdaptationState;

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

pub fn format_adaptation(state: &RuntimeAdaptationState) -> String {
    format!(
        "target={:.2}ms p50/p95/p99={:.2}/{:.2}/{:.2} jitter={:.2} q={:.2} spp={:.2} tile={:.2} sub={} res={}x{} scale={:.2} cd={} streak={}/{}",
        state.target_frame_ms,
        state.frame_p50_ms,
        state.frame_p95_ms,
        state.frame_p99_ms,
        state.jitter_ms,
        state.quality_bias,
        state.sample_pressure_scale,
        state.scheduler_granularity,
        state.substeps,
        state.internal_width,
        state.internal_height,
        state.internal_scale(),
        state.resize_cooldown_frames,
        state.over_budget_streak,
        state.under_budget_streak,
    )
}
