
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
use crate::core::debug::profiling::format_adaptation;
use crate::core::debug::runtime::RuntimeAdaptationState;
use crate::core::debug::serialization::SerializationManager;
use crate::core::debug::tools::DebugTools;
use crate::core::engine::event::event_system::{EngineEvent, EventBus};
use crate::core::engine::physics::physics_manager::PhysicsManager;

#[derive(Debug, Clone, Copy)]
struct FrameTimingStats {
    p50_ms: f64,
    p95_ms: f64,
    p99_ms: f64,
    jitter_ms: f64,
}

#[derive(Debug)]
pub struct EngineLoop {
    config: EngineConfig,
    time: TimeManager,
    loop_controller: LoopController,
    resource: ResourceManager,
    profiler: FrameProfiler,
    logger: EngineLogger,
    debug: DebugTools,
    serializer: SerializationManager,
    audio: AudioManager,
    network: NetworkManager,
    sync_server: RenderSyncServer,
    input: InputManager,
    physics: PhysicsManager,
    events: EventBus,
    frame_times_ms: [f64; 120],
    frame_times_len: usize,
    frame_times_cursor: usize,
    smoothed_quality_bias: f64,
    smoothed_sample_pressure_scale: f64,
    smoothed_substep_scale: f64,
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
            serializer: SerializationManager,
            audio: AudioManager::new(0.85),
            network: NetworkManager::new(2),
            sync_server: RenderSyncServer::new(4),
            input: InputManager::new(true),
            physics,
            events: EventBus::default(),
            frame_times_ms: [0.0; 120],
            frame_times_len: 0,
            frame_times_cursor: 0,
            smoothed_quality_bias: 1.0,
            smoothed_sample_pressure_scale: 1.0,
            smoothed_substep_scale: 1.0,
        }
    }

    pub fn run_frame(&mut self) -> Result<RenderReport, Box<dyn Error>> {
        let mut frame_target = self.loop_controller.frame_target(
            self.config.width,
            self.config.height,
            self.resource.surface_detail_scale(),
        );
        let timing = self.frame_timing_stats();

        let mut requested_substeps = 4_u32;
        let mut sample_pressure_scale = 1.0_f64;
        if let Some(stats) = timing {
            let pressure = ((stats.p95_ms / frame_target.target_frame_ms) - 1.0).max(0.0);
            let tail = ((stats.p99_ms / frame_target.target_frame_ms) - 1.0).max(0.0);
            let jitter = (stats.jitter_ms / frame_target.target_frame_ms).max(0.0);

            let quality_scale = (1.0 - pressure * 0.22 - tail * 0.18 - jitter * 0.15).clamp(0.58, 1.08);
            let target_quality_bias = (frame_target.quality_bias * quality_scale).clamp(0.55, 1.15);
            self.smoothed_quality_bias = smooth_with_hysteresis(
                self.smoothed_quality_bias,
                target_quality_bias,
                0.18,
                0.45,
                0.025,
            );
            frame_target.quality_bias = self.smoothed_quality_bias;

            let target_sample_pressure_scale = (1.0 - pressure * 0.30 - tail * 0.24 - jitter * 0.18).clamp(0.55, 1.12);
            self.smoothed_sample_pressure_scale = smooth_with_hysteresis(
                self.smoothed_sample_pressure_scale,
                target_sample_pressure_scale,
                0.16,
                0.38,
                0.02,
            );
            sample_pressure_scale = self.smoothed_sample_pressure_scale;

            let target_substep_scale = (1.0 - pressure * 0.35 - tail * 0.30 - jitter * 0.20).clamp(0.45, 1.0);
            self.smoothed_substep_scale = smooth_with_hysteresis(
                self.smoothed_substep_scale,
                target_substep_scale,
                0.14,
                0.34,
                0.03,
            );
            requested_substeps = ((4.0 * self.smoothed_substep_scale).round() as u32).clamp(1, 8);
        }

        let substeps = self.loop_controller.recommended_substeps(frame_target.quality_bias, requested_substeps);
        let adaptation_state = RuntimeAdaptationState {
            target_frame_ms: frame_target.target_frame_ms,
            frame_p50_ms: timing.map(|s| s.p50_ms).unwrap_or(frame_target.target_frame_ms),
            frame_p95_ms: timing.map(|s| s.p95_ms).unwrap_or(frame_target.target_frame_ms),
            frame_p99_ms: timing.map(|s| s.p99_ms).unwrap_or(frame_target.target_frame_ms),
            jitter_ms: timing.map(|s| s.jitter_ms).unwrap_or(0.0),
            quality_bias: frame_target.quality_bias,
            sample_pressure_scale,
            scheduler_granularity: 1.0,
            substeps,
            internal_width: self.config.width,
            internal_height: self.config.height,
            output_width: self.config.width,
            output_height: self.config.height,
            resize_cooldown_frames: 0,
            over_budget_streak: 0,
            under_budget_streak: 0,
        };
        let step = self.time.advance_frame(frame_target.target_frame_ms / 1000.0, substeps);

        self.events.push(EngineEvent::FrameStarted {
            frame_index: step.frame_index,
            target_ms: frame_target.target_frame_ms,
        });
        self.events.push(EngineEvent::AdaptationUpdated {
            state: adaptation_state,
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
        if step.frame_index.is_multiple_of(30) {
            self.logger.info(format!("frame {} — {}", step.frame_index, format_adaptation(&adaptation_state)));
        }

        let renderer = Renderer::with_resolution(self.config.width, self.config.height);

        let report = renderer.render_scene_to_file_with_pressure(
            &engine_scene.scene,
            &engine_scene.camera,
            &self.config.output_path,
            self.config.render_preset,
            sample_pressure_scale,
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
            adaptation: adaptation_state,
        });
        let overlay_payload = self.serializer.serialize_overlay(&overlay);

        self.logger.debug(format!(
            "{} | warnings={} | events={} | momentum={:.3} | payload={}B",
            overlay.headline,
            self.logger.warning_count(),
            self.events.history_len(),
            self.physics.total_momentum(),
            overlay_payload.len(),
        ));

        if summary.frame_index.is_multiple_of(30) {
            self.logger.info(format!(
                "frame {} done — {} px, {} ms total, luminance {:.6}",
                summary.frame_index,
                summary.rendered_pixels,
                summary.total_frame_ms,
                report.average_luminance,
            ));
        }

        self.record_frame_time_ms(summary.total_frame_ms as f64);

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

    fn record_frame_time_ms(&mut self, frame_ms: f64) {
        self.frame_times_ms[self.frame_times_cursor] = frame_ms.max(0.0);
        self.frame_times_cursor = (self.frame_times_cursor + 1) % self.frame_times_ms.len();
        self.frame_times_len = (self.frame_times_len + 1).min(self.frame_times_ms.len());
    }

    fn frame_timing_stats(&self) -> Option<FrameTimingStats> {
        if self.frame_times_len == 0 {
            return None;
        }

        let values = &self.frame_times_ms[..self.frame_times_len];
        let mut sorted = values.to_vec();
        sorted.sort_by(f64::total_cmp);

        let p50_ms = percentile_from_sorted(&sorted, 0.50);
        let p95_ms = percentile_from_sorted(&sorted, 0.95);
        let p99_ms = percentile_from_sorted(&sorted, 0.99);

        let mean = values.iter().copied().sum::<f64>() / self.frame_times_len as f64;
        let variance = values
            .iter()
            .map(|v| {
                let d = *v - mean;
                d * d
            })
            .sum::<f64>()
            / self.frame_times_len as f64;
        let jitter_ms = variance.sqrt();

        Some(FrameTimingStats {
            p50_ms,
            p95_ms,
            p99_ms,
            jitter_ms,
        })
    }
}

fn percentile_from_sorted(sorted: &[f64], percentile: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    let idx = ((sorted.len() - 1) as f64 * percentile.clamp(0.0, 1.0)).round() as usize;
    sorted[idx]
}

fn smooth_with_hysteresis(current: f64, target: f64, rise_alpha: f64, fall_alpha: f64, dead_band: f64) -> f64 {
    let delta = target - current;
    if delta.abs() <= dead_band {
        return current;
    }

    let alpha = if delta > 0.0 { rise_alpha } else { fall_alpha };
    current + delta * alpha.clamp(0.0, 1.0)
}
