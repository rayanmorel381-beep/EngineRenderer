pub mod ai;
pub mod camera;
pub mod engine;
pub mod materials;
pub mod objects;
pub mod scenes;
pub mod types;

pub use self::engine::EngineApi;
pub mod animation;
pub mod scene_descriptor;

pub fn diagnose_compute_environment() {
	crate::core::engine::rendering::shader_dispatcher::diagnose_compute_environment();
}

