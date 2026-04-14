use crate::core::engine::acces_hardware::HwInstant;

use crate::core::engine::rendering::renderer::types::RenderReport;

#[derive(Debug, Clone)]
pub struct FrameSummary {
    pub frame_index: u64,
    pub simulation_ms: u128,
    pub scene_prep_ms: u128,
    pub total_frame_ms: u128,
    pub rendered_pixels: usize,
    pub scene_nodes: usize,
}

#[derive(Debug, Default)]
pub struct FrameProfiler;

#[derive(Debug)]
pub struct ActiveFrameProfile {
    frame_index: u64,
    started_at: HwInstant,
    simulation_completed_at: Option<HwInstant>,
    scene_prepared_at: Option<HwInstant>,
}

impl FrameProfiler {
    pub fn begin_frame(&self, frame_index: u64) -> ActiveFrameProfile {
        ActiveFrameProfile {
            frame_index,
            started_at: HwInstant::now(),
            simulation_completed_at: None,
            scene_prepared_at: None,
        }
    }

    pub fn finish_frame(
        &self,
        profile: ActiveFrameProfile,
        report: &RenderReport,
        scene_nodes: usize,
    ) -> FrameSummary {
        let finished_at = HwInstant::now();
        let simulation_completed_at = profile
            .simulation_completed_at
            .unwrap_or(profile.started_at);
        let scene_prepared_at = profile.scene_prepared_at.unwrap_or(simulation_completed_at);

        FrameSummary {
            frame_index: profile.frame_index,
            simulation_ms: simulation_completed_at.duration_since_ms(&profile.started_at),
            scene_prep_ms: scene_prepared_at.duration_since_ms(&simulation_completed_at),
            total_frame_ms: finished_at.duration_since_ms(&profile.started_at),
            rendered_pixels: report.rendered_pixels,
            scene_nodes,
        }
    }
}

impl ActiveFrameProfile {
    pub fn mark_simulation_complete(&mut self) {
        self.simulation_completed_at = Some(HwInstant::now());
    }

    pub fn mark_scene_prepared(&mut self) {
        self.scene_prepared_at = Some(HwInstant::now());
    }
}
