//! `enginerenderer` — 3D ray-tracing rendering engine.
//!
//! The main entry point is [`api::engine::EngineApi`].
//! Most public types are exposed under [`api`].
pub mod api;
pub(crate) mod core;
