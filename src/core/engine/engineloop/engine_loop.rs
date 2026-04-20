
use std::error::Error;

use crate::core::engine::rendering::renderer::types::RenderReport;
use crate::core::engine::rendering::renderer::Renderer;

use crate::core::engine::config::EngineConfig;
use crate::core::engine::scene::celestial::CelestialBodies;
use crate::core::engine::scene::engine_scene::EngineScene;
use crate::core::engine::scene::graph::SceneGraph;
use crate::core::scheduler::loop_controller::LoopController;
use crate::core::scheduler::profiling::FrameProfiler;
use crate::core::scheduler::resource::ResourceManager;
use crate::core::coremanager::audio_manager::AudioManager;
use crate::core::coremanager::camera_manager::CameraManager;
use crate::core::coremanager::input_manager::InputManager;
use crate::core::coremanager::network_manager::{NetworkManager, RenderSyncServer};
use crate::core::coremanager::time_manager::TimeManager;
use crate::core::debug::logger::EngineLogger;
use crate::core::debug::tools::DebugTools;
use crate::core::engine::event::event_system::{EngineEvent, EventBus};
use crate::core::engine::physics::physics_manager::PhysicsManager;

#[derive(Debug)]
pub struct EngineLoop {
    config: EngineConfig,
    time: TimeManager,
    loop_controller: LoopController,
    resource: ResourceManager,
    profiler: FrameProfiler,
    logger: EngineLogger,
    debug: DebugTools,
    audio: AudioManager,
    network: NetworkManager,
    sync_server: RenderSyncServer,
    input: InputManager,
    physics: PhysicsManager,
    events: EventBus,
}

impl EngineLoop {
    pub fn new(config: EngineConfig) -> Self {
        let bodies = CelestialBodies::showcase();
        let physics = PhysicsManager::from_bodies(&bodies);
        let resource = ResourceManager::from_config(&config);

        Self {
            config,
            time: TimeManager::new(1.0 / 120.0),
            loop_controller: LoopController::new(120.0, 8),
            resource,
            profiler: FrameProfiler,
            logger: EngineLogger::with_capacity(256),
            debug: DebugTools,
            audio: AudioManager::new(0.85),
            network: NetworkManager::new(2),
            sync_server: RenderSyncServer::new(4),
            input: InputManager::new(true),
            physics,
            events: EventBus::default(),
        }
    }

    pub fn run_frame(&mut self) -> Result<RenderReport, Box<dyn Error>> {
        let frame_target = self.loop_controller.frame_target(
            self.config.width,
            self.config.height,
            self.resource.surface_detail_scale(),
        );

        let substeps = self.loop_controller.recommended_substeps(frame_target.quality_bias, 4);
        let step = self.time.advance_frame(1.0 / 30.0, substeps);

        self.events.push(EngineEvent::FrameStarted {
            frame_index: step.frame_index,
            target_ms: frame_target.target_frame_ms,
        });

        let mut profile = self.profiler.begin_frame(step.frame_index);

        // ── simulation ──────────────────────────────────────────────
        let bodies = CelestialBodies::showcase();
        self.physics.rebuild_from_bodies(&bodies);
        let frame_input = self.input.sample_cinematic_input(step.absolute_time);
        profile.mark_simulation_complete();

        self.events.push(EngineEvent::SimulationAdvanced {
            body_count: self.physics.body_count(),
        });

        // ── scene assembly ──────────────────────────────────────────
        let graph = SceneGraph::from_bodies(&bodies);
        let camera_manager = CameraManager::cinematic_for_scene(
            graph.focus_point(),
            graph.scene_radius(),
        );
        let engine_scene = EngineScene::from_bodies(
            &bodies,
            &camera_manager,
            &self.resource,
            graph,
            self.config.aspect_ratio(),
            step.absolute_time,
        );
        profile.mark_scene_prepared();

        self.events.push(EngineEvent::ScenePrepared {
            node_count: engine_scene.node_count(),
        });

        // ── audio & network ─────────────────────────────────────────
        let audio_mix = self.audio.mix_for_scene(
            &engine_scene.graph,
            camera_manager.distance_to_focus(),
            frame_input.exposure_nudge,
        );
        self.events.push(EngineEvent::AudioMixed {
            master_gain: audio_mix.master_gain,
        });

        let net_snap = self.network.sync_scene(&engine_scene.graph, step.frame_index);
        self.sync_server.publish(step.frame_index, &net_snap);
        self.events.push(EngineEvent::NetworkSynchronized {
            checksum: net_snap.checksum,
            clients: self.sync_server.client_count(),
        });

        // ── render ──────────────────────────────────────────────────
        self.logger.info(format!(
            "frame {} — {}×{} quality_bias={:.2}",
            step.frame_index, self.config.width, self.config.height, frame_target.quality_bias,
        ));

        let renderer = Renderer::with_resolution(self.config.width, self.config.height);

        let report = renderer.render_scene_to_file(
            &engine_scene.scene,
            &engine_scene.camera,
            &self.config.output_path,
            self.config.render_preset,
        )?;

        self.events.push(EngineEvent::FrameRendered {
            pixels: report.rendered_pixels,
            output_path: self.config.output_path.display().to_string(),
        });

        // ── profiling & debug ───────────────────────────────────────
        let summary = self.profiler.finish_frame(profile, &report, engine_scene.node_count());
        let event_summary = self.events.summarize_history();
        let network_status = self.network.status();
        let overlay = self.debug.capture(crate::core::debug::tools::DebugCaptureInput {
            summary: &summary,
            report: &report,
            network: network_status,
            audio: audio_mix,
            event_summary: &event_summary,
            warning_count: self.logger.warning_count(),
            momentum_hint: self.physics.total_momentum(),
            log_depth: self.logger.len(),
        });

        // ── serialization ───────────────────────────────────────────
        self.logger.debug(format!(
            "{} | warnings={} | events={} | momentum={:.3}",
            overlay.headline,
            self.logger.warning_count(),
            self.events.history_len(),
            self.physics.total_momentum(),
        ));

        self.logger.info(format!(
            "frame {} done — {} px, {} ms total, luminance {:.6}",
            summary.frame_index,
            summary.rendered_pixels,
            summary.total_frame_ms,
            report.average_luminance,
        ));

        Ok(report)
    }

    pub fn run_gallery(&mut self) -> Result<Vec<RenderReport>, Box<dyn Error>> {
        let shots = EngineScene::dedicated_gallery_shots();
        let renderer = Renderer::with_resolution(self.config.width, self.config.height);
        let mut reports = Vec::with_capacity(shots.len());

        for shot in &shots {
            let path = self.config.output_path.with_file_name(format!("gallery_{}.ppm", shot.name));
            let report = renderer.render_scene_to_file(
                &shot.scene,
                &shot.camera,
                &path,
                self.config.render_preset,
            )?;
            self.logger.info(format!(
                "gallery '{}' — {} px, {:.1} ms",
                shot.name, report.rendered_pixels, report.duration_ms,
            ));
            reports.push(report);
        }

        Ok(reports)
    }
}
