//! High-level CPU/GPU `Renderer` orchestrator: hardware-aware initialisation,
//! preset → `RenderConfig` mapping, BVH cache, and GPU compute/upload helpers.

pub mod pipeline;
pub mod render_thread;
pub mod scene_builder;
pub mod types;

mod bvh_cache;
mod gpu_ops;
mod init;
mod presets;
mod state;

pub use state::Renderer;
