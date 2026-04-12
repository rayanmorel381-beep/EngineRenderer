use crate::{
    audio::mixer::AudioManager,
    debug::{
        logger::EngineLogger,
        profiling::FrameProfiler,
        serialization::SerializationManager,
        tools::DebugTools,
    },
    engine::{
        config::EngineConfig,
        loop_controller::LoopController,
        resource::ResourceManager,
        time_manager::TimeManager,
    },
    input::{
        camera::CameraManager,
        events::EventBus,
        manager::InputManager,
    },
    network::{manager::NetworkManager, server::RenderSyncServer},
    physics::physics_manager::PhysicsManager,
    rendering::renderer::Renderer,
    scene::{celestial::CelestialBodies, graph::SceneGraph},
};

#[derive(Debug)]
pub struct EngineManager {
    pub(super) config: EngineConfig,
    pub(super) renderer: Renderer,
    pub(super) camera_manager: CameraManager,
    pub(super) bodies: CelestialBodies,
    pub(super) resource_manager: ResourceManager,
    pub(super) time_manager: TimeManager,
    pub(super) profiler: FrameProfiler,
    pub(super) loop_controller: LoopController,
    pub(super) input_manager: InputManager,
    pub(super) event_bus: EventBus,
    pub(super) logger: EngineLogger,
    pub(super) audio_manager: AudioManager,
    pub(super) network_manager: NetworkManager,
    pub(super) sync_server: RenderSyncServer,
    pub(super) physics_manager: PhysicsManager,
    pub(super) debug_tools: DebugTools,
    pub(super) serializer: SerializationManager,
}

impl EngineManager {
    pub fn new(config: EngineConfig) -> Self {
        let bodies = CelestialBodies::showcase();
        let graph = SceneGraph::from_bodies(&bodies);
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

        let physics_manager = PhysicsManager::from_bodies(&bodies);

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
}
