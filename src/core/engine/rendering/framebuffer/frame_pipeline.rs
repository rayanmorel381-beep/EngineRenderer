use std::error::Error;

use crate::{
    core::event_system::EngineEvent,
    rendering::renderer::{RenderPreset, RenderReport},
    scene::{engine_scene::EngineScene, graph::SceneGraph},
};

use super::manager::EngineManager;

impl EngineManager {
    pub fn render_frame(&mut self) -> Result<RenderReport, Box<dyn Error>> {
        let frame_step = self.time_manager.advance_frame(0.024, 1);
        let mut frame_profile = self.profiler.begin_frame(frame_step.frame_index);
        let frame_target = self.loop_controller.frame_target(
            self.config.width,
            self.config.height,
            self.resource_manager.surface_detail_scale(),
        );
        let frame_input = self
            .input_manager
            .sample_cinematic_input(frame_step.absolute_time);

        self.event_bus.push(EngineEvent::FrameStarted {
            frame_index: frame_step.frame_index,
            target_ms: frame_target.target_frame_ms,
        });

        let graph = SceneGraph::from_bodies(&self.bodies);

        self.physics_manager.rebuild_from_bodies(&self.bodies);
        frame_profile.mark_simulation_complete();

        self.event_bus.push(EngineEvent::SimulationAdvanced {
            body_count: self.physics_manager.body_count(),
        });

        let detail_scale = self.resource_manager.surface_detail_scale();
        let node_count = graph.node_count();
        let luminous_ratio = graph.luminous_node_count() as f64 / node_count.max(1) as f64;
        let base = (node_count as f64).ln().max(1.0);
        let radius_factor = (graph.scene_radius() / 10.0).clamp(0.5, 3.0);
        let detail = (base * (1.0 + luminous_ratio) * radius_factor).clamp(0.1, 5.0)
            * detail_scale.clamp(0.75, 2.5);
        let camera_distance_scale = (1.0 + detail * 0.15).clamp(0.8, 2.0);
        let exposure_bias = (1.0 + detail * 0.12).clamp(0.95, 1.18);
        self.camera_manager.reframe(
            graph.focus_point(),
            graph.scene_radius() * camera_distance_scale,
        );

        let engine_scene = EngineScene::from_bodies(
            &self.bodies,
            &self.camera_manager,
            &self.resource_manager,
            graph.clone(),
            self.config.aspect_ratio(),
            frame_step.absolute_time + frame_input.orbit_bias * 0.15,
        );
        frame_profile.mark_scene_prepared();

        self.event_bus.push(EngineEvent::ScenePrepared {
            node_count: engine_scene.node_count(),
        });

        let network_snapshot = self
            .network_manager
            .sync_scene(&graph, frame_step.frame_index);
        let delivered_clients = self
            .sync_server
            .publish(frame_step.frame_index, &network_snapshot);
        self.event_bus.push(EngineEvent::NetworkSynchronized {
            checksum: network_snapshot.checksum,
            clients: delivered_clients,
        });

        let audio_mix = self.audio_manager.mix_for_scene(
            &graph,
            self.camera_manager.distance_to_focus(),
            exposure_bias + frame_input.exposure_nudge,
        );
        self.event_bus.push(EngineEvent::AudioMixed {
            master_gain: audio_mix.master_gain,
        });

        let report = self.renderer.render_scene_to_file(
            &engine_scene.scene,
            &engine_scene.camera,
            self.resource_manager.output_path(),
            self.config.render_preset,
        )?;
        let preview_report: Option<RenderReport> = None;
        let gallery_reports: Vec<RenderReport> = Vec::new();

        let summary = self
            .profiler
            .finish_frame(frame_profile, &report, engine_scene.node_count());

        if summary.total_frame_ms as f64 > frame_target.target_frame_ms * 1.20 {
            self.logger.warning(format!(
                "Frame {} exceeded target {:.2}ms with {:.2}ms",
                summary.frame_index,
                frame_target.target_frame_ms,
                summary.total_frame_ms as f64
            ));
        }
        if !(0.02..=1.35).contains(&report.average_luminance) {
            self.logger.warning(format!(
                "Frame {} luminance drifted to {:.4}",
                summary.frame_index,
                report.average_luminance
            ));
        }
        if let Some(preview_report) = &preview_report {
            self.logger.info(format!(
                "Generated preview reference at {} ({} px)",
                preview_report.output_path.display(),
                preview_report.rendered_pixels
            ));
        }
        if !gallery_reports.is_empty() {
            self.logger.info(format!(
                "Generated {} dedicated showcase renders",
                gallery_reports.len()
            ));
        }

        self.event_bus.push(EngineEvent::FrameRendered {
            pixels: report.rendered_pixels,
            output_path: report.output_path.display().to_string(),
        });

        let event_summary = self.event_bus.summarize_history();
        let warning_count = self.logger.warning_count();
        let overlay = self.debug_tools.capture(
            &summary,
            &report,
            self.network_manager.status(),
            audio_mix,
            &event_summary,
            warning_count,
            self.physics_manager.total_momentum(),
            self.logger.len(),
        );
        self.logger.debug(format!(
            "{} | clients={} | warnings={} | momentum={:.3}",
            overlay.headline,
            delivered_clients,
            warning_count,
            self.physics_manager.total_momentum(),
        ));

        let drained_events = self.event_bus.drain();
        if !drained_events.is_empty() {
            self.logger.debug(format!("event_bus: drained {} events", drained_events.len()));
        }

        Ok(report)
    }
}
