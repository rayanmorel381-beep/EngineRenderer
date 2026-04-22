
/// AI-facing scene generation and capability APIs.
pub mod ai;
/// Camera controllers and presets.
pub mod camera;
/// Main rendering engine entry points.
pub mod engine;
/// Material builders, presets and lookup catalog.
pub mod materials;
/// High-level scene objects and composites.
pub mod objects;
/// Scene descriptors, builders and presets.
pub mod scenes;
/// Shared API-level data types.
pub mod types;

/// Public engine facade.
pub use self::engine::EngineApi;
/// Animation clip and sequencing APIs.
pub mod animation;

/// Prints the current compute environment diagnostics using default options.
pub fn diagnose_compute_environment() {
	let api = self::engine::EngineApi::new();
	api.diagnose_compute_environment(&self::engine::diagnostics::DiagnosticsOptions::default());
}

