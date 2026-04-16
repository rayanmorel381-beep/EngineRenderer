/// Builder de scène et structures descriptives.
pub mod builder;
/// Presets de scène.
pub mod presets;

/// Presets réexportés.
pub use self::presets::*;
/// Structures descriptives réexportées.
pub use self::builder::{SceneDescriptor, SphereEntry, TriangleEntry, AreaLightEntry};
