/// Types colorimétriques.
pub mod color;
/// Types de configuration.
pub mod config;
/// Types cœur (requêtes/résultats/descripteurs).
pub mod core;
/// Types de transformation.
pub mod transform;

/// Réexports color.
pub use self::color::*;
/// Réexports config.
pub use self::config::*;
/// Réexports core.
pub use self::core::*;
/// Réexports transform.
pub use self::transform::*;

// Re-export material construction types alongside ours.
/// Réexports matériaux utilisés par les types API.
pub use crate::api::materials::{MaterialBuilder, PhysicsConfig};
