//! Public engine-facing API modules.

/// Définition de `EngineApi`, point d'entrée du moteur.
pub mod engine_api;
/// Méthodes de l'API caméra.
pub mod cameras;
/// Méthodes de l'API objets (étoiles, planètes, véhicules…).
pub mod objects;
/// Méthodes de l'API rendu + types `RealtimeRequest`/`RealtimeResult`/`GeneratorRequest`.
pub mod rendering;
/// Méthodes de l'API scène.
pub mod scenes;
/// Rendu depuis un `SceneDescriptor` ou un fichier.
pub mod descriptor;
/// Diagnostic des capacités matérielles et de dispatch.
pub mod diagnostics;

/// Main public engine API entry point.
pub use engine_api::EngineApi;
/// High-level runtime engine used by the terminal mode helpers.
pub use crate::core::coremanager::engine_manager::Engine;
