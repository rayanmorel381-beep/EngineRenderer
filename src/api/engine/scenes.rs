use crate::api::objects::scene_object::SceneObject;
use crate::api::scenes::builder::SceneBuilder;

use super::engine_api::EngineApi;

// Re-export scene/physics/event internals
pub use crate::core::engine::config::EngineConfig;
pub use crate::core::engine::event::event_system::{EventBus, EngineEvent};
pub use crate::core::engine::physics::physics_manager::PhysicsManager;
pub use crate::core::engine::scene::celestial::CelestialBodies;
pub use crate::core::engine::scene::engine_scene::EngineScene;
pub use crate::core::engine::scene::graph::SceneGraph;
pub use crate::core::simulation::nbody::{NBodySystem, GRAVITY};
pub use crate::core::coremanager::camera_manager::CameraManager;
pub use crate::core::scheduler::resource::ResourceManager;

impl EngineApi {
    // -- scene construction -------------------------------------------------

    pub fn scene(&self) -> SceneBuilder {
        SceneBuilder::new()
    }

    pub fn scene_from_objects(&self, objects: Vec<SceneObject>) -> SceneBuilder {
        let mut builder = SceneBuilder::new();
        for obj in objects {
            builder = builder.add_object(obj);
        }
        builder.auto_frame()
    }
}
