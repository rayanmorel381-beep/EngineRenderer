use crate::core::coremanager::audio_manager::AudioMix;
use crate::core::coremanager::network_manager::NetworkStatus;
use crate::core::debug::runtime::RuntimeAdaptationState;
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
    pub adaptation: RuntimeAdaptationState,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct DebugTools;

#[derive(Debug, Clone, Copy)]
pub struct DebugCaptureInput<'a> {
    pub summary: &'a FrameSummary,
    pub report: &'a RenderReport,
    pub network: NetworkStatus,
    pub audio: AudioMix,
    pub event_summary: &'a EventSummary,
    pub warning_count: usize,
    pub momentum_hint: f64,
    pub log_depth: usize,
    pub adaptation: RuntimeAdaptationState,
}

impl DebugTools {
    pub fn capture(&self, input: DebugCaptureInput<'_>) -> DebugOverlay {
        DebugOverlay {
            headline: format!(
                "frame={} px={} p95={:.1}/{:.1}ms q={:.2} spp={:.2} sub={} res={}x{} tile={:.2} net={:.1}ms warn={} evt={} mom={:.2}",
                input.summary.frame_index,
                input.report.rendered_pixels,
                input.adaptation.frame_p95_ms,
                input.adaptation.target_frame_ms,
                input.adaptation.quality_bias,
                input.adaptation.sample_pressure_scale,
                input.adaptation.substeps,
                input.adaptation.internal_width,
                input.adaptation.internal_height,
                input.adaptation.scheduler_granularity,
                input.network.latency_ms,
                input.warning_count,
                input.event_summary.clients,
                input.momentum_hint
            ),
            frame_index: input.summary.frame_index,
            frame_time_ms: input.summary.total_frame_ms,
            render_time_ms: input.report.duration_ms,
            network_latency_ms: input.network.latency_ms,
            master_gain: input.audio.master_gain,
            spatial_width: input.audio.spatial_width,
            reverb_send: input.audio.reverb_send,
            warning_count: input.warning_count,
            event_history: input.event_summary.clients.max(input.event_summary.node_count),
            momentum_hint: input.momentum_hint,
            log_depth: input.log_depth,
            adaptation: input.adaptation,
        }
    }
}
