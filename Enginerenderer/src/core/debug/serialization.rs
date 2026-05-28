use super::tools::DebugOverlay;

#[derive(Debug, Default, Clone, Copy)]
pub struct SerializationManager;

impl SerializationManager {
    pub fn serialize_overlay(&self, overlay: &DebugOverlay) -> String {
        format!(
            "frame={};frame_ms={};render_ms={};latency_ms={:.2};audio_gain={:.2};spatial={:.2};reverb={:.2};warnings={};events={};momentum={:.3};logs={};target_ms={:.2};p50_ms={:.2};p95_ms={:.2};p99_ms={:.2};jitter_ms={:.2};quality_bias={:.3};sample_scale={:.3};scheduler_granularity={:.3};substeps={};internal={}x{};output={}x{};internal_scale={:.3};budget_pressure={:.3};tail_pressure={:.3};resize_cooldown={};over_budget_streak={};under_budget_streak={}",
            overlay.frame_index,
            overlay.frame_time_ms,
            overlay.render_time_ms,
            overlay.network_latency_ms,
            overlay.master_gain,
            overlay.spatial_width,
            overlay.reverb_send,
            overlay.warning_count,
            overlay.event_history,
            overlay.momentum_hint,
            overlay.log_depth,
            overlay.adaptation.target_frame_ms,
            overlay.adaptation.frame_p50_ms,
            overlay.adaptation.frame_p95_ms,
            overlay.adaptation.frame_p99_ms,
            overlay.adaptation.jitter_ms,
            overlay.adaptation.quality_bias,
            overlay.adaptation.sample_pressure_scale,
            overlay.adaptation.scheduler_granularity,
            overlay.adaptation.substeps,
            overlay.adaptation.internal_width,
            overlay.adaptation.internal_height,
            overlay.adaptation.output_width,
            overlay.adaptation.output_height,
            overlay.adaptation.internal_scale(),
            overlay.adaptation.budget_pressure(),
            overlay.adaptation.tail_pressure(),
            overlay.adaptation.resize_cooldown_frames,
            overlay.adaptation.over_budget_streak,
            overlay.adaptation.under_budget_streak,
        )
    }

}
