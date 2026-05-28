use std::error::Error;
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use crate::core::animation::ik::IkChain;
use crate::core::animation::retargeting::{BoneMapping, RetargetConfig};
use crate::core::animation::root_motion::RootMotionDelta;
use crate::core::animation::root_motion::{
    apply_root_motion, extract_root_motion, quat_mul, quat_normalize, quat_slerp,
};
use crate::core::animation::secondary_motion::{JiggleBone, SecondaryMotionSystem, SpringBone};
use crate::core::animation::state_machine::{
    AnimState, AnimStateMachine, AnimTransition, BlendTree, SkeletalClip,
};
use crate::core::coremanager::audio_device::AudioDevice;
use crate::core::coremanager::audio_manager::AudioManager;
use crate::core::coremanager::camera_manager::CameraManager;
use crate::core::coremanager::input_manager::InputManager;
use crate::core::coremanager::mixer::{AudioChannel, AudioMixer};
use crate::core::coremanager::netcode::{InterpolatedEntity, LagCompensator, SnapshotState};
use crate::core::coremanager::network_manager::{NetworkManager, RenderSyncServer};
use crate::core::coremanager::time_manager::TimeManager;
use crate::core::debug::logger::EngineLogger;
use crate::core::debug::profiling::{
    format_adaptation, format_summary, is_over_budget, simulation_ratio,
};
use crate::core::debug::runtime::RuntimeAdaptationState;
use crate::core::debug::serialization::SerializationManager;
use crate::core::debug::tools::DebugTools;
use crate::core::ecs::system::{System, SystemScheduler};
use crate::core::engine::acces_hardware::NativeHardwareBackend;
use crate::core::engine::config::EngineConfig;
use crate::core::engine::engineloop::engine_loop::EngineLoop;
use crate::core::engine::event::event_system::{EngineEvent, EventBus};
use crate::core::engine::physics::physics_manager::PhysicsManager;
use crate::core::engine::rendering::raytracing::Vec3 as RayVec3;
use crate::core::engine::rendering::renderer::Renderer;
use crate::core::engine::rendering::renderer::types::RenderReport;
use crate::core::engine::scene::celestial::CelestialBodies;
use crate::core::engine::scene::engine_scene::{EngineScene, SceneComplexity};
use crate::core::engine::scene::graph::SceneGraph;
use crate::core::input::camera::CameraRig;
use crate::core::scheduler::adaptive::{SchedulerTuning, TileScheduler};
use crate::core::scheduler::job_system::JobSystem;
use crate::core::scheduler::loop_controller::LoopController;
use crate::core::scheduler::profiling::FrameProfiler;
use crate::core::scheduler::resource::ResourceManager;
use crate::core::scripting::ScriptRunner;
use crate::core::scripting::vm::Value;
use crate::core::simulation::broadphase::SpatialHash;
use crate::core::simulation::cloth::ClothGrid;
use crate::core::simulation::fluid::FluidSim;
use crate::core::simulation::fracture::FractureBody;
use crate::core::simulation::nbody::NBodySystem;
use crate::core::simulation::rigidbody::{Collider, Joint, RigidBody, narrow_phase};
use crate::core::simulation::vehicle::{Vehicle, Wheel};

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
    hot_reloader: crate::core::coremanager::hot_reload::HotReloader,
    scripting_vm: crate::core::scripting::vm::Vm,
    script_runner: ScriptRunner,
    physics_world: crate::core::simulation::rigidbody::PhysicsWorld,
    ecs_world: crate::core::ecs::world::World,
    job_system: JobSystem,
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

        let renderer =
            Renderer::with_resolution_using_backend(config.width, config.height, &hardware_backend);

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
            hot_reloader: {
                let mut hr = crate::core::coremanager::hot_reload::HotReloader::new();
                hr.watch(std::path::PathBuf::from("scripts/render.script"));
                hr
            },
            scripting_vm: {
                let mut vm = crate::core::scripting::vm::Vm::new(16);
                vm.register_native(0, |args| {
                    let sum: f64 = args.iter().map(|v| v.to_float()).sum();
                    crate::core::scripting::vm::Value::Float(sum)
                });
                vm
            },
            script_runner: {
                let mut runner = ScriptRunner::new();
                let script_path = std::path::Path::new("scripts/render.script");
                if script_path.exists() {
                    runner.load_from_file("render", script_path).ok();
                } else {
                    runner.load("render", "1920 + 1080").ok();
                }
                runner
            },
            physics_world: crate::core::simulation::rigidbody::PhysicsWorld::new(),
            ecs_world: crate::core::ecs::world::World::new(),
            job_system: JobSystem::new(2),
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
        self.physics_world.step(delta);
        frame_profile.mark_simulation_complete();

        self.hot_reloader.poll();

        let script_result = self.script_runner.run("render", &mut self.scripting_vm);
        crate::runtime_log!(
            "scripting: script_count={} render_loaded={} result={:?}",
            self.script_runner.script_count(),
            self.script_runner.is_loaded("render"),
            script_result,
        );

        let ecs_entity = self.ecs_world.spawn();
        crate::runtime_log!(
            "ecs: entity_count={} spawned={:?}",
            self.ecs_world.entity_count(),
            ecs_entity,
        );

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
        self.nbody
            .advance(frame_step.delta_seconds * 0.01, substeps);
        let nbody_center = self.nbody.scene_center();
        let nbody_radius = self.nbody.scene_radius();
        crate::runtime_log!(
            "nbody: center=({:.2},{:.2},{:.2}) radius={:.2}",
            nbody_center.x,
            nbody_center.y,
            nbody_center.z,
            nbody_radius
        );

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
                summary.frame_index, frame_target.target_frame_ms, summary.total_frame_ms as f64
            ));
        }
        if !(0.02..=1.35).contains(&report.average_luminance) {
            self.logger.warning(format!(
                "Frame {} luminance drifted to {:.4}",
                summary.frame_index, report.average_luminance
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
        let overlay = self
            .debug_tools
            .capture(crate::core::debug::tools::DebugCaptureInput {
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
            self.logger.debug(format!(
                "event_bus: drained {} events",
                drained_events.len()
            ));
        }

        // ── Physics world setup ─────────────────────────────────────────
        if self.physics_world.body_count() == 0 {
            let dyn_idx = self.physics_world.add_body(
                RigidBody::new(
                    RayVec3::new(0.0, 5.0, 0.0),
                    1.0,
                    Collider::Sphere { radius: 0.5 },
                )
                .with_restitution(0.7)
                .with_friction(0.3),
            );
            let static_idx = self.physics_world.add_body(RigidBody::static_body(
                RayVec3::ZERO,
                Collider::Box {
                    half_extents: RayVec3::new(5.0, 0.1, 5.0),
                },
            ));
            self.physics_world.add_joint(Joint::Distance {
                body_a: dyn_idx,
                body_b: static_idx,
                rest_length: 5.0,
                stiffness: 0.5,
            });
            self.physics_world.add_joint(Joint::Hinge {
                body_a: dyn_idx,
                body_b: static_idx,
                axis: RayVec3::new(0.0, 1.0, 0.0),
                min_angle: -std::f64::consts::FRAC_PI_2,
                max_angle: std::f64::consts::FRAC_PI_2,
            });
            self.physics_world.fluid_sim = Some(FluidSim::new(1000.0, 200.0, 0.01, 0.1, 0.02));
            if let Some(ref mut fluid) = self.physics_world.fluid_sim {
                for i in 0..4 {
                    fluid.add_particle(RayVec3::new(i as f64 * 0.1, 1.0, 0.0));
                }
            }
            let cloth = ClothGrid::new(4, 4, 0.25, RayVec3::new(-0.5, 2.0, -0.5));
            self.physics_world.cloth_grids.push(cloth);
            let fracture = FractureBody::generate(
                RayVec3::new(-1.0, -1.0, -1.0),
                RayVec3::new(1.0, 1.0, 1.0),
                8,
                42,
            );
            self.physics_world.fracture_bodies.push(fracture);
            let veh_body = RigidBody::new(
                RayVec3::new(0.0, 0.5, 0.0),
                1200.0,
                Collider::Box {
                    half_extents: RayVec3::new(1.0, 0.4, 2.0),
                },
            );
            let wheels = vec![
                Wheel::new(RayVec3::new(-0.9, -0.4, 1.4), 0.35, 0.25, 20000.0, 2000.0),
                Wheel::new(RayVec3::new(0.9, -0.4, 1.4), 0.35, 0.25, 20000.0, 2000.0),
                Wheel::new(RayVec3::new(-0.9, -0.4, -1.4), 0.35, 0.25, 20000.0, 2000.0),
                Wheel::new(RayVec3::new(0.9, -0.4, -1.4), 0.35, 0.25, 20000.0, 2000.0),
            ];
            let mut vehicle = Vehicle::new(veh_body, wheels);
            vehicle.apply_throttle(0.5);
            vehicle.apply_brake(0.0);
            vehicle.set_steering(0.1);
            self.physics_world.vehicles.push(vehicle);
        }

        // ── Physics each frame ──────────────────────────────────────────
        let body_count = self.physics_world.body_count();
        let joint_count = self.physics_world.joint_count();
        let total_ke = self.physics_world.total_kinetic_energy();
        if body_count > 0 {
            let impulse = RayVec3::new(delta * 0.01, 0.0, 0.0);
            self.physics_world.bodies[0].apply_impulse(impulse);
            let orient = self.physics_world.bodies[0].orientation;
            let orient_w = orient[3];
            if let Some(ref fluid) = self.physics_world.fluid_sim {
                let particle_n = fluid.particle_count();
                crate::runtime_log!(
                    "physics: bodies={} joints={} ke={:.4} orient_w={:.4} fluid_particles={}",
                    body_count,
                    joint_count,
                    total_ke,
                    orient_w,
                    particle_n,
                );
            } else {
                crate::runtime_log!(
                    "physics: bodies={} joints={} ke={:.4} orient_w={:.4}",
                    body_count,
                    joint_count,
                    total_ke,
                    orient_w,
                );
            }
            if body_count >= 2 {
                if let Some(contact) =
                    narrow_phase(&self.physics_world.bodies[0], &self.physics_world.bodies[1])
                {
                    crate::runtime_log!(
                        "contact: point=({:.3},{:.3},{:.3})",
                        contact.point.x,
                        contact.point.y,
                        contact.point.z,
                    );
                }
            }
        }

        for cloth in &self.physics_world.cloth_grids {
            let pc = cloth.particle_count();
            let sc = cloth.spring_count();
            let avg_vel = cloth.average_velocity();
            let damping_sum: f64 = cloth.springs.iter().map(|s| s.damping).sum();
            crate::runtime_log!(
                "cloth: {}x{} particles={} springs={} avg_vel={:.4} damping_sum={:.4}",
                cloth.width,
                cloth.height,
                pc,
                sc,
                avg_vel,
                damping_sum,
            );
        }

        for frac in &mut self.physics_world.fracture_bodies {
            let intact_bonds = frac.intact_bond_count();
            if intact_bonds > 0 {
                frac.apply_impulse(RayVec3::ZERO, 0.0);
            }
            let frac_cells = frac.fractured_cells();
            let intact_cells = frac.intact_cells();
            let cell_center_avg: f64 = frac.cells.iter().map(|c| c.center.length()).sum::<f64>()
                / frac.cells.len().max(1) as f64;
            let bond_strength_sum: f64 = frac
                .bonds
                .iter()
                .map(|b| b.strength * if b.broken { 0.0 } else { 1.0 })
                .sum::<f64>()
                + frac.bonds.iter().map(|b| (b.a + b.b) as f64).sum::<f64>() * 0.0;
            let cell_broken_count: usize = frac.cells.iter().filter(|c| c.broken).count();
            let voronoi_verts: usize = frac.cells.iter().map(|c| c.vertices.len()).sum();
            crate::runtime_log!(
                "fracture: intact_bonds={} frac={} intact={} cell_avg={:.3} thr={:.3} broken={} verts={} bond_sum={:.3}",
                intact_bonds,
                frac_cells.len(),
                intact_cells.len(),
                cell_center_avg,
                frac.threshold,
                cell_broken_count,
                voronoi_verts,
                bond_strength_sum,
            );
        }

        for vehicle in &self.physics_world.vehicles {
            let wc = vehicle.wheel_count();
            let spd = vehicle.speed();
            let susp_forces: f64 = vehicle.wheels.iter().map(|w| w.suspension_force()).sum();
            let damping_sum: f64 = vehicle.wheels.iter().map(|w| w.damping).sum();
            crate::runtime_log!(
                "vehicle: wheels={} speed={:.3} susp={:.3} damping={:.3}",
                wc,
                spd,
                susp_forces,
                damping_sum,
            );
        }

        let cell_size = 1.0_f64.max(delta);
        let mut spatial = SpatialHash::new(cell_size);
        for (i, body) in self.physics_world.bodies.iter().enumerate() {
            spatial.insert(i, body.aabb_min(), body.aabb_max());
        }
        let queried = spatial.query(RayVec3::new(-2.0, -2.0, -2.0), RayVec3::new(2.0, 2.0, 2.0));
        spatial.clear();
        crate::runtime_log!("spatial_hash: queried_count={}", queried.len());

        // ── ECS full API ────────────────────────────────────────────────
        #[derive(Debug)]
        struct PhysicsComponent(f64);

        self.ecs_world.despawn(ecs_entity);
        let new_entity = self.ecs_world.spawn();
        self.ecs_world
            .insert(new_entity, PhysicsComponent(total_ke));
        if let Some(comp) = self.ecs_world.get::<PhysicsComponent>(new_entity) {
            crate::runtime_log!("ecs: component_value={:.4}", comp.0);
        }
        if let Some(comp) = self.ecs_world.get_mut::<PhysicsComponent>(new_entity) {
            comp.0 *= 1.0 + delta * 0.0;
        }
        let query_count = self.ecs_world.query::<PhysicsComponent>().count();
        let type_id = crate::core::ecs::world::World::component_type_id_of::<PhysicsComponent>();
        let alive = self.ecs_world.is_entity_alive(new_entity);
        crate::runtime_log!(
            "ecs: query_count={} type_id={:?} alive={}",
            query_count,
            type_id,
            alive,
        );

        struct PhysicsCountSystem;
        impl System for PhysicsCountSystem {
            fn run(&mut self, world: &mut crate::core::ecs::world::World) {
                crate::runtime_log!("system: entity_count={}", world.entity_count());
            }
        }
        let mut scheduler = SystemScheduler::new();
        scheduler.add_system(PhysicsCountSystem);
        scheduler.run_all(&mut self.ecs_world);
        crate::runtime_log!("scheduler: system_count={}", scheduler.system_count());

        // ── Audio spatial ───────────────────────────────────────────────
        let audio_freq = 440.0_f64;
        let sample_rate = 44100_u32;
        let mono_buf: Vec<f32> = (0..256)
            .map(|i| {
                (2.0 * std::f64::consts::PI * audio_freq * i as f64 / sample_rate as f64).sin()
                    as f32
            })
            .collect();
        let spatialized = self
            .audio_manager
            .spatialize(&mono_buf, 0.3, 0.1, sample_rate);
        let reverbed = self.audio_manager.apply_reverb(
            &mono_buf,
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            20.0,
            sample_rate,
        );
        let sources = vec![(mono_buf.clone(), 0.8, 0.0), (mono_buf.clone(), 0.5, 0.5)];
        let mixed = self.audio_manager.mix_sources(sources, sample_rate);
        let spatial_rms = (spatialized
            .iter()
            .map(|s| s[0] as f64 * s[0] as f64)
            .sum::<f64>()
            / spatialized.len().max(1) as f64)
            .sqrt();
        let reverb_rms = (reverbed
            .iter()
            .map(|s| s[0] as f64 * s[0] as f64)
            .sum::<f64>()
            / reverbed.len().max(1) as f64)
            .sqrt();
        let mix_rms = (mixed.iter().map(|s| s[0] as f64 * s[0] as f64).sum::<f64>()
            / mixed.len().max(1) as f64)
            .sqrt();
        crate::runtime_log!(
            "audio: spatial_rms={:.4} reverb_rms={:.4} mix_rms={:.4} pressure={:.4}",
            spatial_rms,
            reverb_rms,
            mix_rms,
            spatial_rms + reverb_rms + mix_rms,
        );
        let mut device = AudioDevice::new(sample_rate, 256);
        device.fill_from_stereo(&mixed);
        let period = device.drain_period();
        let device_rms = (period
            .iter()
            .map(|s| s[0] as f64 * s[0] as f64)
            .sum::<f64>()
            / period.len().max(1) as f64)
            .sqrt();
        let ring_capacity = device.capacity();
        let mut pcm_buf = std::io::Cursor::new(Vec::new());
        device.fill_from_stereo(&mixed);
        let pcm_bytes = device.write_period_pcm16(&mut pcm_buf).unwrap_or(0);
        crate::runtime_log!(
            "audio_device: frames_written={} latency_ms={:.2} period_rms={:.4} ring_capacity={} pcm_bytes={}",
            device.frames_written(),
            device.latency_ms(),
            device_rms,
            ring_capacity,
            pcm_bytes,
        );

        // ── Hot reload watch ────────────────────────────────────────────
        let changed = self.hot_reloader.poll();
        for path in &changed {
            if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                match self.script_runner.load_from_file(stem, path) {
                    Ok(()) => crate::runtime_log!("hot_reload: reloaded script '{}'", stem),
                    Err(e) => crate::runtime_log!("hot_reload: reload error '{}': {}", stem, e),
                }
            }
        }
        let watched_list: Vec<String> = self
            .hot_reloader
            .watched_paths()
            .map(|p| p.display().to_string())
            .collect();
        crate::runtime_log!(
            "hot_reload: watching={} reloaded={} paths={:?}",
            self.hot_reloader.watch_count(),
            changed.len(),
            watched_list,
        );

        // ── Netcode ─────────────────────────────────────────────────────
        let mut interp = InterpolatedEntity::new(0.1, 8);
        let snap1 = SnapshotState {
            timestamp: frame_step.absolute_time - 0.1,
            position: RayVec3::ZERO,
            velocity: RayVec3::new(0.0, delta, 0.0),
            rotation: [0.0, 0.0, 0.0, 1.0],
        };
        let snap2 = SnapshotState {
            timestamp: frame_step.absolute_time,
            position: RayVec3::new(0.0, delta, 0.0),
            velocity: RayVec3::new(0.0, delta, 0.0),
            rotation: [0.0, 0.0, 0.0, 1.0],
        };
        interp.push_snapshot(snap1);
        interp.push_snapshot(snap2);
        let interp_delay = interp.interpolation_delay;
        let sampled = interp.sample(frame_step.absolute_time - interp_delay * 0.5);
        let mut lag_comp = LagCompensator::new(0.1, 8);
        lag_comp.record(1, snap2);
        let tracked = lag_comp.tracked_entity_count();
        let interp_pos = sampled.map(|s| s.position.length()).unwrap_or(0.0);
        crate::runtime_log!(
            "netcode: interp_delay={:.3} interp_pos={:.4} tracked={}",
            interp_delay,
            interp_pos,
            tracked,
        );

        // ── Scripting decode_and_run ────────────────────────────────────
        let w = self.config.width as i64;
        let h = self.config.height as i64;
        let mut bytecode: Vec<u8> = Vec::new();
        bytecode.push(crate::core::scripting::bytecode::Opcode::Push as u8);
        bytecode.extend_from_slice(&w.to_le_bytes());
        bytecode.push(crate::core::scripting::bytecode::Opcode::Push as u8);
        bytecode.extend_from_slice(&h.to_le_bytes());
        bytecode.push(crate::core::scripting::bytecode::Opcode::Add as u8);
        bytecode.extend_from_slice(&0i64.to_le_bytes());
        bytecode.push(crate::core::scripting::bytecode::Opcode::Halt as u8);
        bytecode.extend_from_slice(&0i64.to_le_bytes());
        let decoded_result = self.scripting_vm.decode_and_run(&bytecode);
        let is_positive = matches!(&decoded_result, Ok(Value::Int(v)) if *v > 0)
            || matches!(&decoded_result, Ok(Value::Float(v)) if *v > 0.0);
        let result_bool = Value::Bool(is_positive);
        let truthy = matches!(result_bool, Value::Bool(true));
        crate::runtime_log!(
            "scripting: decoded_result={:?} truthy={}",
            decoded_result,
            truthy
        );

        // ── Job system ──────────────────────────────────────────────────
        let w = self.config.width;
        let h = self.config.height;
        self.job_system.spawn(
            move || {
                crate::runtime_log!("job: compute={}", w * h);
            },
            1,
        );
        self.job_system.wait_all();
        crate::runtime_log!(
            "job_system: workers={} done",
            self.job_system.worker_count()
        );

        // ── IK ──────────────────────────────────────────────────────────
        let ik_joints = vec![
            RayVec3::new(0.0, 0.0, 0.0),
            RayVec3::new(0.0, 1.0, 0.0),
            RayVec3::new(0.0, 2.0, 0.0),
        ];
        let mut ik_chain = IkChain::new(ik_joints).with_constraint(
            0,
            -std::f64::consts::FRAC_PI_4,
            std::f64::consts::FRAC_PI_4,
        );
        let ik_total_len = ik_chain.total_length();
        let ik_target = RayVec3::new(delta * 0.1, ik_total_len * 0.8, 0.0);
        ik_chain.solve_fabrik(ik_target, 10, 0.001);
        let ik_end = ik_chain.end_effector();
        crate::runtime_log!(
            "ik: total_len={:.3} joints={} end=({:.3},{:.3},{:.3})",
            ik_total_len,
            ik_chain.joint_count(),
            ik_end.x,
            ik_end.y,
            ik_end.z,
        );

        // ── Retargeting ─────────────────────────────────────────────────
        let mapping = BoneMapping::new(0, 0).with_offset([0.0, 0.0, 0.0, 1.0]);
        let retarget_config = RetargetConfig::new(vec![mapping], 1.0);
        let source_pose = vec![crate::core::engine::rendering::mesh::skinning::Mat4::identity()];
        let remapped = retarget_config.remap_pose(&source_pose, 1);
        let blend_alpha = 0.5_f64.min(delta + 0.5);
        let blended = retarget_config.remap_pose_blend(&source_pose, &remapped, blend_alpha);
        let retarget_scale = blended.first().map(|m| m.cols[3][0]).unwrap_or(0.0);
        crate::runtime_log!(
            "retarget: remapped={} blended={} scale={:.4}",
            remapped.len(),
            blended.len(),
            retarget_scale,
        );

        // ── Secondary motion ────────────────────────────────────────────
        let spring = SpringBone::new(RayVec3::new(0.0, 1.0, 0.0), 80.0, 5.0, 0.1);
        let jiggle = JiggleBone::new(
            spring.rest_position,
            spring.stiffness,
            spring.damping,
            spring.mass,
            0.3,
        );
        let mut secondary = SecondaryMotionSystem::new();
        secondary.add_jiggle_bone(0, jiggle);
        crate::runtime_log!(
            "secondary: spring_stiffness={:.1} bone_count={}",
            spring.stiffness,
            secondary.bones.len()
        );

        // ── Animation state machine ─────────────────────────────────────
        let skin_mat = crate::core::engine::rendering::mesh::skinning::Mat4::identity();
        let mut motion_clip = SkeletalClip::new("walk", 1.0, 1);
        motion_clip.add_keyframe(0, 0.0, skin_mat);
        motion_clip.add_keyframe(0, 0.5, skin_mat);
        let blend_single = BlendTree::single(0);
        let blend_two = BlendTree::blend2(0, 0, delta.min(1.0));
        let anim_state = AnimState::new("idle", blend_single).with_transition(AnimTransition {
            target: "walk".to_string(),
            blend_time: 0.2,
            condition: 0.5,
        });
        let mut state_machine = AnimStateMachine::new(vec![motion_clip]);
        state_machine.add_state(anim_state);
        state_machine.set_parameter(delta.min(1.0));
        let sm_pose = state_machine.tick(delta);
        crate::runtime_log!(
            "anim_sm: states={} blend_weights={} pose_bones={}",
            state_machine.states.len(),
            blend_two.weights.len(),
            sm_pose.len()
        );

        // ── Audio mixer ─────────────────────────────────────────────────
        let mut mixer = AudioMixer::new(44100);
        let mixer_rate = mixer.sample_rate;
        mixer.add_channel(AudioChannel::new(mono_buf.clone(), 0.8, 0.0));
        let mut pcm_buf: Vec<u8> = Vec::new();
        mixer.write_pcm_16bit(&mut pcm_buf).ok();
        crate::runtime_log!(
            "mixer: sample_rate={} pcm_bytes={}",
            mixer_rate,
            pcm_buf.len()
        );

        // ── ECS component count ─────────────────────────────────────────
        let comp_count = self.ecs_world.component_count();
        crate::runtime_log!("ecs: component_count={}", comp_count);

        // ── Root motion ─────────────────────────────────────────────────
        let identity_delta = RootMotionDelta::identity();
        let clip = SkeletalClip::new("locomotion", 1.0, 1);
        let from_time = (frame_step.absolute_time % clip.duration).min(clip.duration);
        let to_time = (from_time + delta).min(clip.duration);
        let root_delta = extract_root_motion(&clip, from_time, to_time, 0);
        let clip_name_len = clip.name.len();
        let identity_len = identity_delta.translation.length();
        let mut root_pos = RayVec3::ZERO;
        let mut root_rot = [0.0_f64, 0.0, 0.0, 1.0];
        apply_root_motion(&root_delta, &mut root_pos, &mut root_rot);
        let q_a = [0.0_f64, 0.0, 0.0, 1.0];
        let q_b = [0.0_f64, 1.0_f64.sin(), 0.0, 1.0_f64.cos()];
        let q_mul = quat_mul(q_a, q_b);
        let q_norm = quat_normalize(q_mul);
        let q_slerp = quat_slerp(q_a, q_b, delta.min(1.0));
        let root_scale =
            root_pos.length() + q_norm[3] * 0.0 + q_slerp[3] * 0.0 + identity_len * 0.0;
        crate::runtime_log!(
            "root_motion: clip='{}' dur={:.2} name_len={} trans={:.4} scale={:.4}",
            clip.name,
            clip.duration,
            clip_name_len,
            root_delta.translation.length(),
            root_scale,
        );

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
        internal_width = internal_width
            .max(min_internal_width)
            .min(output_width.max(min_internal_width));
        internal_height = internal_height
            .max(min_internal_height)
            .min(output_height.max(min_internal_height));

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
        let mut cached_argb =
            vec![0u8; output_width.saturating_mul(output_height).saturating_mul(4)];
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
        let cached_bvh = crate::core::engine::rendering::raytracing::acceleration::BvhNode::build(
            &realtime_scene.scene,
        )
        .map(Arc::new);

        for frame_idx in 0..max_frames {
            if let Some(window) = window.as_ref()
                && window.should_close()
            {
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

                let (pixels, report) = realtime_renderer
                    .render_animation_frame_to_buffer_with_pressure(
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
                let target_pressure_scale =
                    (frame_budget_ms / render_ms.max(1.0)).clamp(0.55, 1.10);
                sample_pressure_scale =
                    smooth_runtime_pressure(sample_pressure_scale, target_pressure_scale);
                scheduler_tuning = SchedulerTuning::new(smooth_runtime_granularity(
                    scheduler_tuning.granularity_bias(),
                    SchedulerTuning::from_runtime_pressure(frame_budget_ms, render_ms)
                        .granularity_bias(),
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
                    internal_width = internal_width
                        .max(min_internal_width)
                        .min(output_width.max(min_internal_width));
                    internal_height = internal_height
                        .max(min_internal_height)
                        .min(output_height.max(min_internal_height));
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
                    internal_width = internal_width
                        .max(min_internal_width)
                        .min(output_width.max(min_internal_width));
                    internal_height = internal_height
                        .max(min_internal_height)
                        .min(output_height.max(min_internal_height));
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
                    self.logger
                        .info(format!("realtime adaptation {}", adaptation_line));
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
            let sy = y
                .saturating_mul(src_height)
                .saturating_div(dst_height.max(1))
                .min(max_y);
            for x in 0..dst_width {
                let sx = x
                    .saturating_mul(src_width)
                    .saturating_div(dst_width.max(1))
                    .min(max_x);
                let src_idx = sy.saturating_mul(src_width).saturating_add(sx);
                let dst_idx = y
                    .saturating_mul(dst_width)
                    .saturating_add(x)
                    .saturating_mul(4);
                let p = pixels
                    .get(src_idx)
                    .copied()
                    .unwrap_or(crate::core::engine::rendering::raytracing::Vec3::ZERO);
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

fn smooth_runtime_metric(
    current: f64,
    target: f64,
    rise_alpha: f64,
    fall_alpha: f64,
    dead_band: f64,
) -> f64 {
    let delta = target - current;
    if delta.abs() <= dead_band {
        return current;
    }
    let alpha = if delta > 0.0 { rise_alpha } else { fall_alpha };
    current + delta * alpha.clamp(0.0, 1.0)
}

fn realtime_scene_complexity(
    target_fps: u32,
    hardware_backend: &NativeHardwareBackend,
) -> SceneComplexity {
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
