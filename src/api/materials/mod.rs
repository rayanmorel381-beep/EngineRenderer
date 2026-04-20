pub mod builder;
pub mod catalog;
pub mod physics;
pub mod shortcuts;
pub mod spectrum;

pub use self::builder::MaterialBuilder;
pub use self::physics::PhysicsConfig;
pub use self::shortcuts::*;
pub use self::spectrum::{Spectrum, SPECTRAL_BANDS};
