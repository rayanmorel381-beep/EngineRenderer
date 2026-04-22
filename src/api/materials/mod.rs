/// Material builder entry points.
pub mod builder;
/// Built-in material catalog definitions.
pub mod catalog;
/// Physical material parameters.
pub mod physics;
/// Convenience constructors for common materials.
pub mod shortcuts;
/// Spectral material utilities.
pub mod spectrum;

pub use self::builder::MaterialBuilder;
pub use self::physics::PhysicsConfig;
pub use self::shortcuts::*;
pub use self::spectrum::{Spectrum, SPECTRAL_BANDS};
