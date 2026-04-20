
pub mod ai;
pub mod camera;
pub mod engine;
pub mod materials;
pub mod objects;
pub mod scenes;
pub mod types;

pub use self::engine::EngineApi;
pub mod animation;

pub fn diagnose_compute_environment() {
	let api = self::engine::EngineApi::new();
	api.diagnose_compute_environment(&self::engine::diagnostics::DiagnosticsOptions::default());
}

