use crate::api::objects::scene_object::SceneObject;
use crate::api::scenes::builder::SceneBuilder;

use super::engine_api::EngineApi;

/// Engine config re-export.
pub use crate::core::engine::config::EngineConfig;
/// Event bus and event type re-exports.
pub use crate::core::engine::event::event_system::{EventBus, EngineEvent};
/// Physics manager re-export.
pub use crate::core::engine::physics::physics_manager::PhysicsManager;
/// Celestial catalog re-export.
pub use crate::core::engine::scene::celestial::CelestialBodies;
/// Engine scene re-export.
pub use crate::core::engine::scene::engine_scene::EngineScene;
/// Scene graph re-export.
pub use crate::core::engine::scene::graph::SceneGraph;
/// N-body simulation re-exports.
pub use crate::core::simulation::nbody::{NBodySystem, GRAVITY};
/// Camera manager re-export.
pub use crate::core::coremanager::camera_manager::CameraManager;
/// Resource manager re-export.
pub use crate::core::scheduler::resource::ResourceManager;

impl EngineApi {
    // -- scene construction -------------------------------------------------

    /// Starts an empty scene builder.
    pub fn scene(&self) -> SceneBuilder {
        SceneBuilder::new()
    }

    /// Builds a scene builder pre-populated with objects.
    pub fn scene_from_objects(&self, objects: Vec<SceneObject>) -> SceneBuilder {
        let mut builder = SceneBuilder::new();
        for obj in objects {
            builder = builder.add_object(obj);
        }
        builder.auto_frame()
    }
}
