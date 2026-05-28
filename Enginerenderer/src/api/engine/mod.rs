/// Camera-related helpers exposed by the engine API.
pub mod cameras;
/// Engine descriptor helpers.
pub mod descriptor;
/// Compute and hardware diagnostics.
pub mod diagnostics;
/// Core engine API type and base construction helpers.
pub mod engine_api;
/// Object factory helpers exposed by the engine API.
pub mod objects;
/// Rendering entry points and runtime requests.
pub mod rendering;
/// Scene construction helpers and scene-related exports.
pub mod scenes;

/// Core engine runtime object.
pub use crate::core::coremanager::engine_manager::Engine;
/// Main high-level API for engine features.
pub use engine_api::EngineApi;
