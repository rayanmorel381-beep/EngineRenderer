use std::error::Error;
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use crate::core::coremanager::audio_manager::AudioManager;
use crate::core::coremanager::camera_manager::CameraManager;
use crate::core::coremanager::input_manager::InputManager;
use crate::core::coremanager::network_manager::{NetworkManager, RenderSyncServer};
use crate::core::engine::engineloop::engine_loop::EngineLoop;
use crate::core::coremanager::time_manager::TimeManager;
use crate::core::debug::logger::EngineLogger;
use crate::core::debug::profiling::{format_adaptation, format_summary, is_over_budget, simulation_ratio};
use crate::core::debug::runtime::RuntimeAdaptationState;
use crate::core::debug::serialization::SerializationManager;
use crate::core::debug::tools::DebugTools;
use crate::core::engine::acces_hardware::NativeHardwareBackend;
use crate::core::engine::config::EngineConfig;
use crate::core::engine::event::event_system::{EngineEvent, EventBus};
use crate::core::engine::physics::physics_manager::PhysicsManager;
use crate::core::engine::rendering::renderer::types::RenderReport;
use crate::core::engine::rendering::renderer::Renderer;
use crate::core::engine::scene::celestial::CelestialBodies;
use crate::core::engine::scene::engine_scene::{EngineScene, SceneComplexity};
use crate::core::engine::scene::graph::SceneGraph;
use crate::core::input::camera::CameraRig;
use crate::core::scheduler::loop_controller::LoopController;
use crate::core::scheduler::profiling::FrameProfiler;
use crate::core::scheduler::resource::ResourceManager;
use crate::core::scheduler::adaptive::{SchedulerTuning, TileScheduler};
use crate::core::simulation::nbody::NBodySystem;

#[derive(Debug)]
pub struct EngineManager {
    config: EngineConfig,
    hardware_backend: NativeHardwareBackend,
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
    nbody: NBodySystem,
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

        let hardware_backend = NativeHardwareBackend::detect();

        let renderer = Renderer::with_resolution_using_backend(config.width, config.height, &hardware_backend);

        Self {
            hardware_backend,
            renderer,
            camera_manager: CameraManager::cinematic_for_scene(
                graph.focus_point(),
                graph.scene_radius(),
            ),
            bodies,
            resource_manager,
            time_manager: TimeManager::new(1.0 / 120.0),
            profiler: FrameProfiler,
            loop_controller: LoopController::new(120.0, 1),
            input_manager: InputManager::new(true),
            event_bus: EventBus::default(),
            logger,
            audio_manager: AudioManager::new(0.9),
            network_manager: NetworkManager::new(2),
            sync_server: RenderSyncServer::new(2),
            physics_manager,
            debug_tools: DebugTools,
            serializer: SerializationManager,
            nbody: NBodySystem::showcase(),
            config,
        }
    }

    pub fn render_frame(&mut self) -> Result<RenderReport, Box<dyn Error>> {
        let frame_step = self.time_manager.advance_frame(1.0 / 120.0, 1);

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
        self.nbody.advance(frame_step.delta_seconds * 0.01, substeps);
        let nbody_center = self.nbody.scene_center();
        let nbody_radius = self.nbody.scene_radius();
        crate::runtime_log!("nbody: center=({:.2},{:.2},{:.2}) radius={:.2}", nbody_center.x, nbody_center.y, nbody_center.z, nbody_radius);

        // ── Input facades ───────────────────────────────────────────────

        let rig = CameraRig::cinematic(graph.scene_radius());
        let rig_cam = rig.build_camera(self.config.aspect_ratio(), frame_step.absolute_time);

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

        let summary = self
            .profiler
            .finish_frame(frame_profile, &report, engine_scene.node_count());
        let over_budget = is_over_budget(&summary, frame_target.target_frame_ms);
        let adaptation_state = RuntimeAdaptationState {
            target_frame_ms: frame_target.target_frame_ms,
            frame_p50_ms: summary.total_frame_ms as f64,
            frame_p95_ms: summary.total_frame_ms as f64,
            frame_p99_ms: summary.total_frame_ms as f64,
            jitter_ms: 0.0,
            quality_bias,
            sample_pressure_scale: 1.0,
            scheduler_granularity: 1.0,
            substeps,
            internal_width: self.config.width,
            internal_height: self.config.height,
            output_width: self.config.width,
            output_height: self.config.height,
            resize_cooldown_frames: 0,
            over_budget_streak: usize::from(over_budget),
            under_budget_streak: 0,
        };

        // ── Debug profiling functions ────────────────────────────────────
        let summary_str = format_summary(&summary);
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

        self.event_bus.push(EngineEvent::FrameRendered {
            pixels: report.rendered_pixels,
            output_path: report.output_path.display().to_string(),
        });
        self.event_bus.push(EngineEvent::AdaptationUpdated {
            state: adaptation_state,
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
            adaptation: adaptation_state,
        });
        let overlay_payload = self.serializer.serialize_overlay(&overlay);
        self.logger.debug(format!(
            "{} | adaptation={} | payload={}B | clients={} (sync={})",
            overlay.headline,
            format_adaptation(&adaptation_state),
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

    /// Runs realtime rendering for a duration at a target FPS.
    pub fn run_realtime(&mut self, seconds: u32, fps: u32) -> Result<(), Box<dyn Error>> {
        let target_fps = fps.clamp(1, 240);
        let target_seconds = seconds.max(1);
        let ultra_target = target_fps >= 120;
        let scene_complexity = realtime_scene_complexity(target_fps, &self.hardware_backend);
        let ultra_constrained = ultra_target && scene_complexity.proxy_showcase_meshes;
        let frame_time = Duration::from_secs_f64(1.0 / target_fps as f64);
        let pacing_frame_time = if ultra_constrained {
            Duration::from_secs_f64((1.0 / target_fps as f64) * 0.90)
        } else {
            frame_time
        };
        let max_frames = (target_fps as usize).saturating_mul(target_seconds as usize);
        let frame_budget_ms = 1000.0 / target_fps as f64;
        let realtime_threads = std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(4)
            .clamp(1, 16);
        let output_width = self.config.width;
        let output_height = self.config.height;
        let initial_scale = if ultra_constrained {
            (20.0 / target_fps as f64).sqrt().clamp(0.03, 0.18)
        } else if ultra_target {
            (30.0 / target_fps as f64).sqrt().clamp(0.06, 0.35)
        } else {
            (30.0 / target_fps as f64).sqrt().clamp(0.18, 1.0)
        };
        let min_internal_width = if ultra_constrained {
            32
        } else if ultra_target {
            64
        } else {
            160
        };
        let min_internal_height = if ultra_constrained {
            18
        } else if ultra_target {
            36
        } else {
            90
        };
        let mut internal_width = ((output_width as f64) * initial_scale).round() as usize;
        let mut internal_height = ((output_height as f64) * initial_scale).round() as usize;
        internal_width = internal_width.max(min_internal_width).min(output_width.max(min_internal_width));
        internal_height = internal_height.max(min_internal_height).min(output_height.max(min_internal_height));

        let mut window = crate::core::engine::acces_hardware::NativeWindow::open(
            output_width,
            output_height,
            "EngineRenderer realtime",
        );
        let headless = window.is_none();

        let mut realtime_renderer = Renderer::with_resolution_using_backend(
            internal_width,
            internal_height,
            &self.hardware_backend,
        );
        let mut scheduler_tuning = SchedulerTuning::default();
        let mut scheduler = TileScheduler::new_with_backend_tuned(
            internal_width,
            internal_height,
            realtime_threads,
            &self.hardware_backend,
            scheduler_tuning,
        );

        let mut rendered_frames = 0usize;
        let mut total_render_ms = 0u128;
        let mut over_budget_frames = 0usize;
        let mut sample_pressure_scale = 1.0_f64;
        let mut resize_cooldown_frames = 0usize;
        let mut over_budget_streak = 0usize;
        let mut under_budget_streak = 0usize;
        let render_interval = if ultra_constrained {
            8usize
        } else if ultra_target {
            4usize
        } else {
            1usize
        };
        let mut present_width = output_width;
        let mut present_height = output_height;
        let mut cached_argb = vec![0u8; output_width.saturating_mul(output_height).saturating_mul(4)];
        let total_start = Instant::now();
        let mut next_frame_deadline = Instant::now() + pacing_frame_time;
        let graph = SceneGraph::from_bodies(&self.bodies);
        let hint = scene_hint(&graph, self.resource_manager.surface_detail_scale());
        self.camera_manager.reframe(
            graph.focus_point(),
            graph.scene_radius() * hint.camera_distance_scale,
        );
        let realtime_scene = EngineScene::from_bodies_with_complexity(
            &self.bodies,
            &self.camera_manager,
            &self.resource_manager,
            graph.clone(),
            output_width as f64 / output_height.max(1) as f64,
            0.0,
            scene_complexity,
        );
        let cached_bvh = crate::core::engine::rendering::raytracing::acceleration::BvhNode::build(&realtime_scene.scene)
            .map(Arc::new);

        for frame_idx in 0..max_frames {
            if let Some(window) = window.as_ref() && window.should_close() {
                break;
            }

            let frame_step = self.time_manager.advance_frame(1.0 / target_fps as f64, 1);

            let render_this_frame = frame_idx % render_interval == 0;
            if render_this_frame {
                let frame_input = self
                    .input_manager
                    .sample_cinematic_input(frame_step.absolute_time);
                let realtime_camera = EngineScene::realtime_camera(
                    &self.camera_manager,
                    &graph,
                    internal_width as f64 / internal_height.max(1) as f64,
                    frame_step.absolute_time + frame_input.orbit_bias * 0.15,
                );

                let (pixels, report) = realtime_renderer.render_animation_frame_to_buffer_with_pressure(
                    &realtime_scene.scene,
                    &realtime_camera,
                    cached_bvh.as_deref(),
                    &scheduler,
                    self.config.render_preset,
                    sample_pressure_scale,
                )?;

                let target_present_width = if ultra_constrained {
                    report.width
                } else {
                    output_width
                };
                let target_present_height = if ultra_constrained {
                    report.height
                } else {
                    output_height
                };
                present_width = target_present_width;
                present_height = target_present_height;
                cached_argb = Self::upscale_argb_from_vec3(
                    &pixels,
                    report.width,
                    report.height,
                    target_present_width,
                    target_present_height,
                );

                total_render_ms = total_render_ms.saturating_add(report.duration_ms);
                if (report.duration_ms as f64) > frame_budget_ms {
                    over_budget_frames = over_budget_frames.saturating_add(1);
                }

                let render_ms = report.duration_ms as f64;
                let target_pressure_scale = (frame_budget_ms / render_ms.max(1.0)).clamp(0.55, 1.10);
                sample_pressure_scale = smooth_runtime_pressure(sample_pressure_scale, target_pressure_scale);
                scheduler_tuning = SchedulerTuning::new(smooth_runtime_granularity(
                    scheduler_tuning.granularity_bias(),
                    SchedulerTuning::from_runtime_pressure(frame_budget_ms, render_ms).granularity_bias(),
                ));
                if resize_cooldown_frames > 0 {
                    resize_cooldown_frames = resize_cooldown_frames.saturating_sub(1);
                }

                if render_ms > frame_budget_ms * 1.02 {
                    over_budget_streak = over_budget_streak.saturating_add(1);
                    under_budget_streak = 0;
                } else if render_ms < frame_budget_ms * 0.50 {
                    under_budget_streak = under_budget_streak.saturating_add(1);
                    over_budget_streak = 0;
                } else {
                    over_budget_streak = 0;
                    under_budget_streak = 0;
                }

                if resize_cooldown_frames == 0
                    && over_budget_streak >= 3
                    && internal_width > min_internal_width
                    && internal_height > min_internal_height
                {
                    let shrink = if ultra_constrained {
                        0.60
                    } else if ultra_target {
                        0.74
                    } else {
                        0.82
                    };
                    internal_width = ((internal_width as f64) * shrink).round() as usize;
                    internal_height = ((internal_height as f64) * shrink).round() as usize;
                    internal_width = internal_width.max(min_internal_width).min(output_width.max(min_internal_width));
                    internal_height = internal_height.max(min_internal_height).min(output_height.max(min_internal_height));
                    realtime_renderer = Renderer::with_resolution_using_backend(
                        internal_width,
                        internal_height,
                        &self.hardware_backend,
                    );
                    scheduler = TileScheduler::new_with_backend_tuned(
                        internal_width,
                        internal_height,
                        realtime_threads,
                        &self.hardware_backend,
                        scheduler_tuning,
                    );
                    resize_cooldown_frames = 18;
                    over_budget_streak = 0;
                    under_budget_streak = 0;
                } else if resize_cooldown_frames == 0
                    && under_budget_streak >= 8
                    && internal_width < output_width
                    && internal_height < output_height
                {
                    let grow = if ultra_target { 1.04 } else { 1.10 };
                    internal_width = ((internal_width as f64) * grow).round() as usize;
                    internal_height = ((internal_height as f64) * grow).round() as usize;
                    internal_width = internal_width.max(min_internal_width).min(output_width.max(min_internal_width));
                    internal_height = internal_height.max(min_internal_height).min(output_height.max(min_internal_height));
                    realtime_renderer = Renderer::with_resolution_using_backend(
                        internal_width,
                        internal_height,
                        &self.hardware_backend,
                    );
                    scheduler = TileScheduler::new_with_backend_tuned(
                        internal_width,
                        internal_height,
                        realtime_threads,
                        &self.hardware_backend,
                        scheduler_tuning,
                    );
                    resize_cooldown_frames = 24;
                    over_budget_streak = 0;
                    under_budget_streak = 0;
                }

                let adaptation_state = RuntimeAdaptationState {
                    target_frame_ms: frame_budget_ms,
                    frame_p50_ms: render_ms,
                    frame_p95_ms: render_ms,
                    frame_p99_ms: render_ms,
                    jitter_ms: 0.0,
                    quality_bias: 1.0,
                    sample_pressure_scale,
                    scheduler_granularity: scheduler_tuning.granularity_bias(),
                    substeps: 1,
                    internal_width,
                    internal_height,
                    output_width,
                    output_height,
                    resize_cooldown_frames,
                    over_budget_streak,
                    under_budget_streak,
                };

                if frame_idx % 30 == 0 {
                    let adaptation_line = format_adaptation(&adaptation_state);
                    self.logger.info(format!("realtime adaptation {}", adaptation_line));
                    crate::runtime_log!("realtime adaptation {}", adaptation_line);
                }
            }

            if let Some(window) = window.as_mut()
                && (!ultra_constrained || render_this_frame)
            {
                window.present_frame(&cached_argb, present_width, present_height);
            }

            rendered_frames = rendered_frames.saturating_add(1);

            let now = Instant::now();
            if now < next_frame_deadline {
                thread::sleep(next_frame_deadline - now);
            }
            next_frame_deadline += pacing_frame_time;
            let drift_now = Instant::now();
            if drift_now > next_frame_deadline + pacing_frame_time {
                next_frame_deadline = drift_now + pacing_frame_time;
            }
        }

        let rendered_samples = rendered_frames.div_ceil(render_interval);
        let avg_ms = if rendered_samples == 0 {
            0.0
        } else {
            total_render_ms as f64 / rendered_samples as f64
        };
        let total_elapsed = total_start.elapsed().as_secs_f64();
        let achieved_fps = if total_elapsed > 0.0 {
            rendered_frames as f64 / total_elapsed
        } else {
            0.0
        };
        let stable_ratio = if rendered_samples == 0 {
            0.0
        } else {
            1.0 - (over_budget_frames as f64 / rendered_samples as f64)
        };
        crate::runtime_log!(
            "realtime: frames={} target_fps={} achieved_fps={:.1} stable_ratio={:.2} avg_render_ms={:.2} internal={}x{} output={}x{} headless={}",
            rendered_frames,
            target_fps,
            achieved_fps,
            stable_ratio,
            avg_ms,
            internal_width,
            internal_height,
            output_width,
            output_height,
            headless,
        );

        Ok(())
    }

    fn upscale_argb_from_vec3(
        pixels: &[crate::core::engine::rendering::raytracing::Vec3],
        src_width: usize,
        src_height: usize,
        dst_width: usize,
        dst_height: usize,
    ) -> Vec<u8> {
        let mut out = vec![0u8; dst_width.saturating_mul(dst_height).saturating_mul(4)];
        let clamp = |v: f64| -> u8 { (v.clamp(0.0, 1.0) * 255.0).round() as u8 };
        let max_x = src_width.saturating_sub(1);
        let max_y = src_height.saturating_sub(1);

        for y in 0..dst_height {
            let sy = y.saturating_mul(src_height).saturating_div(dst_height.max(1)).min(max_y);
            for x in 0..dst_width {
                let sx = x.saturating_mul(src_width).saturating_div(dst_width.max(1)).min(max_x);
                let src_idx = sy.saturating_mul(src_width).saturating_add(sx);
                let dst_idx = y
                    .saturating_mul(dst_width)
                    .saturating_add(x)
                    .saturating_mul(4);
                let p = pixels.get(src_idx).copied().unwrap_or(crate::core::engine::rendering::raytracing::Vec3::ZERO);
                out[dst_idx] = 255;
                out[dst_idx + 1] = clamp(p.x);
                out[dst_idx + 2] = clamp(p.y);
                out[dst_idx + 3] = clamp(p.z);
            }
        }

        out
    }
}

fn smooth_runtime_pressure(current: f64, target: f64) -> f64 {
    smooth_runtime_metric(current, target, 0.12, 0.30, 0.02)
}

fn smooth_runtime_granularity(current: f64, target: f64) -> f64 {
    smooth_runtime_metric(current, target, 0.14, 0.34, 0.03)
}

fn smooth_runtime_metric(current: f64, target: f64, rise_alpha: f64, fall_alpha: f64, dead_band: f64) -> f64 {
    let delta = target - current;
    if delta.abs() <= dead_band {
        return current;
    }
    let alpha = if delta > 0.0 { rise_alpha } else { fall_alpha };
    current + delta * alpha.clamp(0.0, 1.0)
}

fn realtime_scene_complexity(target_fps: u32, hardware_backend: &NativeHardwareBackend) -> SceneComplexity {
    if target_fps >= 120 {
        return SceneComplexity::full();
    }

    let logical_cores = hardware_backend.hw_caps().logical_cores;
    let arm_family = cfg!(any(target_arch = "arm", target_arch = "aarch64"));

    if target_fps >= 60 {
        if arm_family || logical_cores <= 8 {
            return SceneComplexity {
                showcase_mesh_budget: 2,
                area_light_budget: 1,
                panorama_enabled: false,
                refined_showcase_meshes: false,
                proxy_showcase_meshes: true,
            };
        }

        return SceneComplexity {
            showcase_mesh_budget: 6,
            area_light_budget: 2,
            panorama_enabled: true,
            refined_showcase_meshes: true,
            proxy_showcase_meshes: false,
        };
    }

    SceneComplexity::full()
}

/// High-level engine facade.
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
    /// Creates an engine configured for realtime preview.
    pub fn realtime() -> Self {
        Self {
            manager: EngineManager::new(EngineConfig::realtime_preview()),
        }
    }

    /// Creates an engine for realtime preview with explicit resolution.
    pub fn realtime_with_resolution(width: usize, height: usize) -> Self {
        let mut config = EngineConfig::realtime_preview();
        config.width = width.max(1);
        config.height = height.max(1);
        Self {
            manager: EngineManager::new(config),
        }
    }

    /// Creates an engine configured for production reference rendering.
    pub fn production_reference() -> Self {
        Self {
            manager: EngineManager::new(EngineConfig::production_reference()),
        }
    }

    /// Creates a minimal engine configuration for fast tests.
    pub fn test_minimal() -> Self {
        Self {
            manager: EngineManager::new(EngineConfig::test_minimal()),
        }
    }

    /// Renders a single frame and returns its report.
    pub fn run(mut self) -> Result<RenderReport, Box<dyn Error>> {
        self.manager.render_frame()
    }

    /// Renders the full gallery sequence.
    pub fn render_gallery(self) -> Result<Vec<RenderReport>, Box<dyn Error>> {
        let config = self.manager.config.clone();
        let mut loop_runner = EngineLoop::new(config);
        loop_runner.run_frame()?;
        loop_runner.run_gallery()
    }

    /// Runs the realtime loop for a duration at target FPS.
    pub fn run_realtime(mut self, seconds: u32, fps: u32) -> Result<(), Box<dyn Error>> {
        self.manager.run_realtime(seconds, fps)
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
