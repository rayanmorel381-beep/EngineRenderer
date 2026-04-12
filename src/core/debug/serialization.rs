use super::tools::DebugOverlay;

#[derive(Debug, Default, Clone, Copy)]
pub struct SerializationManager;

impl SerializationManager {
    pub fn serialize_overlay(&self, overlay: &DebugOverlay) -> String {
        format!(
            "frame={};frame_ms={};render_ms={};latency_ms={:.2};audio_gain={:.2};spatial={:.2};reverb={:.2};warnings={};events={};momentum={:.3};logs={}",
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
            overlay.log_depth
        )
    }

}
