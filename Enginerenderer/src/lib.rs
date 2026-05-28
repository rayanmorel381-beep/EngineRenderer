//! `enginerenderer` — 3D ray-tracing rendering engine.
//!
//! The main entry point is [`api::engine::EngineApi`].
//! Most public types are exposed under [`api`].
//!
//! `unsafe` is restricted to: SIMD intrinsics (NEON), FFI bindings (OpenGL,
//! Linux syscalls), and DMA framebuffer Send/Sync impls. All other code is
//! safe Rust.
#![warn(unsafe_op_in_unsafe_fn)]

/// API public of high niveau.
pub mod api;
pub(crate) mod core;
