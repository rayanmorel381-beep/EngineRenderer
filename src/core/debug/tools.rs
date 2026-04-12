use crate::core::coremanager::audio_manager::AudioMix;
use crate::core::coremanager::network_manager::NetworkStatus;
use crate::core::engine::event::event_system::EventSummary;
use crate::core::engine::rendering::renderer::types::RenderReport;
use crate::core::scheduler::profiling::FrameSummary;

#[derive(Debug, Clone)]
pub struct DebugOverlay {
    pub headline: String,
    pub frame_index: u64,
    pub frame_time_ms: u128,
    pub render_time_ms: u128,
    pub network_latency_ms: f64,
    pub master_gain: f64,
    pub spatial_width: f64,
    pub reverb_send: f64,
    pub warning_count: usize,
    pub event_history: usize,
    pub momentum_hint: f64,
    pub log_depth: usize,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct DebugTools;

impl DebugTools {
    pub fn capture(
        &self,
        summary: &FrameSummary,
        report: &RenderReport,
        network: NetworkStatus,
        audio: AudioMix,
        event_summary: &EventSummary,
        warning_count: usize,
        momentum_hint: f64,
        log_depth: usize,
    ) -> DebugOverlay {
        DebugOverlay {
            headline: format!(
                "frame={} px={} net={:.1}ms audio={:.2}/{:.2} rvb={:.2} warn={} evt={} mom={:.2}",
                summary.frame_index,
                report.rendered_pixels,
                network.latency_ms,
                audio.master_gain,
                audio.spatial_width,
                audio.reverb_send,
                warning_count,
                event_summary.clients,
                momentum_hint
            ),
            frame_index: summary.frame_index,
            frame_time_ms: summary.total_frame_ms,
            render_time_ms: report.duration_ms,
            network_latency_ms: network.latency_ms,
            master_gain: audio.master_gain,
            spatial_width: audio.spatial_width,
            reverb_send: audio.reverb_send,
            warning_count,
            event_history: event_summary.clients.max(event_summary.node_count),
            momentum_hint,
            log_depth,
        }
    }
}
