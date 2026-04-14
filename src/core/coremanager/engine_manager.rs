use std::error::Error;

use crate::core::coremanager::audio_manager::AudioManager;
use crate::core::coremanager::camera_manager::CameraManager;
use crate::core::coremanager::input_manager::InputManager;
use crate::core::coremanager::network_manager::{NetworkManager, RenderSyncServer};
use crate::core::engine::engineloop::engine_loop::EngineLoop;
use crate::core::coremanager::time_manager::TimeManager;
use crate::core::debug::logger::EngineLogger;
use crate::core::debug::profiling::{format_summary, is_over_budget, simulation_ratio};
use crate::core::debug::serialization::SerializationManager;
use crate::core::debug::tools::DebugTools;
use crate::core::engine::config::EngineConfig;
use crate::core::engine::event::event_system::{EngineEvent, EventBus};
use crate::core::engine::physics::physics_manager::PhysicsManager;
use crate::core::engine::rendering::renderer::types::RenderReport;
use crate::core::engine::rendering::renderer::Renderer;
use crate::core::engine::scene::celestial::CelestialBodies;
use crate::core::engine::scene::engine_scene::EngineScene;
use crate::core::engine::scene::graph::SceneGraph;
use crate::core::input::camera::CameraRig;
use crate::core::scheduler::loop_controller::LoopController;
use crate::core::scheduler::profiling::FrameProfiler;
use crate::core::scheduler::resource::ResourceManager;
use crate::core::simulation::nbody::NBodySystem;

#[derive(Debug)]
pub struct EngineManager {
    config: EngineConfig,
    renderer: Renderer,
    camera_manager: CameraManager,
    bodies: CelestialBodies,
    resource_manager: ResourceManager,
    time_manager: TimeManager,
    profiler: FrameProfiler,
    loop_controller: LoopController,
    input_manager: InputManager,
    event_bus: EventBus,
    logger: EngineLogger,
    audio_manager: AudioManager,
    network_manager: NetworkManager,
    sync_server: RenderSyncServer,
    physics_manager: PhysicsManager,
    debug_tools: DebugTools,
    serializer: SerializationManager,
}

impl EngineManager {
    pub fn new(config: EngineConfig) -> Self {
        let bodies = CelestialBodies::showcase();
        let graph = SceneGraph::from_bodies(&bodies);
        let physics_manager = PhysicsManager::from_bodies(&bodies);
        let resource_manager = ResourceManager::from_config(&config);
        let mut logger = EngineLogger::with_capacity(96);
        logger.info(format!(
            "Initialized runtime at {}x{}",
            config.width, config.height
        ));

        let renderer = if config.width == 1600 && config.height == 900 {
            Renderer::default_cpu_hd()
        } else {
            Renderer::with_resolution(config.width, config.height)
        };

        Self {
            renderer,
            camera_manager: CameraManager::cinematic_for_scene(
                graph.focus_point(),
                graph.scene_radius(),
            ),
            bodies,
            resource_manager,
            time_manager: TimeManager::new(0.024),
            profiler: FrameProfiler,
            loop_controller: LoopController::new(60.0, 1),
            input_manager: InputManager::new(true),
            event_bus: EventBus::default(),
            logger,
            audio_manager: AudioManager::new(0.9),
            network_manager: NetworkManager::new(2),
            sync_server: RenderSyncServer::new(2),
            physics_manager,
            debug_tools: DebugTools,
            serializer: SerializationManager,
            config,
        }
    }

    pub fn render_frame(&mut self) -> Result<RenderReport, Box<dyn Error>> {
        let frame_step = self.time_manager.advance_frame(0.024, 1);

        let delta = frame_step.delta_seconds;
        let substeps = frame_step.integration_steps;

        let mut frame_profile = self.profiler.begin_frame(frame_step.frame_index);
        let frame_target = self.loop_controller.frame_target(
            self.config.width,
            self.config.height,
            self.resource_manager.surface_detail_scale(),
        );

        let quality_bias = frame_target.quality_bias;
        let max_substeps = self.loop_controller.recommended_substeps(quality_bias, 4);

        let frame_input = self
            .input_manager
            .sample_cinematic_input(frame_step.absolute_time);

        let time_scale = frame_input.time_scale;

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

        let hint = scene_hint(&graph, self.resource_manager.surface_detail_scale());
        self.camera_manager.reframe(
            graph.focus_point(),
            graph.scene_radius() * hint.camera_distance_scale,
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

        // ── N-body simulation ───────────────────────────────────────────
        let mut nbody = NBodySystem::showcase();
        nbody.advance(frame_step.absolute_time * 0.01, substeps);
        let nbody_center = nbody.scene_center();
        let nbody_radius = nbody.scene_radius();
        eprintln!("nbody: center=({:.2},{:.2},{:.2}) radius={:.2}", nbody_center.x, nbody_center.y, nbody_center.z, nbody_radius);

        // ── Input facades ───────────────────────────────────────────────

        let rig = CameraRig::cinematic(graph.scene_radius());
        let rig_cam = rig.build_camera(self.config.aspect_ratio(), frame_step.absolute_time);












        // ── Debug profiling functions ───────────────────────────────────
        // (used after profiler.finish_frame below)

        let audio_mix = self.audio_manager.mix_for_scene(
            &graph,
            self.camera_manager.distance_to_focus(),
            hint.exposure_bias + frame_input.exposure_nudge,
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

        // ── Debug profiling functions ────────────────────────────────────
        let summary_str = format_summary(&summary);
        let over_budget = is_over_budget(&summary, frame_target.target_frame_ms);
        let sim_ratio = simulation_ratio(&summary);
        self.logger.debug(format!(
            "profiling: ratio={:.2} budget={} | {}",
            sim_ratio, over_budget, summary_str
        ));

        self.logger.debug(format!(
            "frame done: delta={:.4} substeps={} scale={:.2} quality={:.2} max_sub={} cam_at=({:.2},{:.2},{:.2})",
            delta, substeps, time_scale, quality_bias, max_substeps,
            rig_cam.origin.x, rig_cam.origin.y, rig_cam.origin.z,
        ));

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
        let overlay = self.debug_tools.capture(crate::core::debug::tools::DebugCaptureInput {
            summary: &summary,
            report: &report,
            network: self.network_manager.status(),
            audio: audio_mix,
            event_summary: &event_summary,
            warning_count,
            momentum_hint: self.physics_manager.total_momentum(),
            log_depth: self.logger.len(),
        });
        let overlay_payload = self.serializer.serialize_overlay(&overlay);
        self.logger.debug(format!(
            "{} | payload={}B | clients={} (sync={})",
            overlay.headline,
            overlay_payload.len(),
            delivered_clients,
            self.sync_server.client_count(),
        ));

        let drained_events = self.event_bus.drain();
        if !drained_events.is_empty() {
            self.logger.debug(format!("event_bus: drained {} events", drained_events.len()));
        }

        Ok(report)
    }
}

#[derive(Debug)]
pub struct Engine {
    manager: EngineManager,
}

impl Default for Engine {
    fn default() -> Self {
        Self {
            manager: EngineManager::new(EngineConfig::ultra_hd_cpu()),
        }
    }
}

impl Engine {
    pub fn production_reference() -> Self {
        Self {
            manager: EngineManager::new(EngineConfig::production_reference()),
        }
    }

    /// Tiny resolution engine for integration tests. Avoids blowing up
    /// CPU/RAM on a full render pipeline.
    pub fn test_minimal() -> Self {
        Self {
            manager: EngineManager::new(EngineConfig::test_minimal()),
        }
    }

    pub fn run(mut self) -> Result<RenderReport, Box<dyn Error>> {
        self.manager.render_frame()
    }

    pub fn render_gallery(self) -> Result<Vec<RenderReport>, Box<dyn Error>> {
        let config = self.manager.config.clone();
        let mut loop_runner = EngineLoop::new(config);
        let _ = loop_runner.run_frame()?;
        loop_runner.run_gallery()
    }
}

// ── Scene analysis (pure math, no AI) ───────────────────────────────────

struct SceneHint {
    camera_distance_scale: f64,
    exposure_bias: f64,
}

fn scene_hint(graph: &SceneGraph, detail_scale: f64) -> SceneHint {
    let node_count = graph.node_count();
    let luminous_ratio = graph.luminous_node_count() as f64 / node_count.max(1) as f64;
    let base = (node_count as f64).ln().max(1.0);
    let radius_factor = (graph.scene_radius() / 10.0).clamp(0.5, 3.0);
    let detail = (base * (1.0 + luminous_ratio) * radius_factor).clamp(0.1, 5.0)
        * detail_scale.clamp(0.75, 2.5);

    SceneHint {
        camera_distance_scale: (1.0 + detail * 0.15).clamp(0.8, 2.0),
        exposure_bias: (1.0 + detail * 0.12).clamp(0.95, 1.18),
    }
}
