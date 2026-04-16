/// Builder fluent de matériaux.
pub mod builder;
/// Catalogue de matériaux prêts à l'emploi.
pub mod catalog;
/// Paramètres physiques matériaux.
pub mod physics;
/// Raccourcis de création.
pub mod shortcuts;
/// Outils spectraux.
pub mod spectrum;

/// Builder principal réexporté.
pub use self::builder::MaterialBuilder;
/// Configuration physique réexportée.
pub use self::physics::PhysicsConfig;
/// Raccourcis réexportés.
pub use self::shortcuts::*;
/// Types spectraux réexportés.
pub use self::spectrum::{Spectrum, SPECTRAL_BANDS};
