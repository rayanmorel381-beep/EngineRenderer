//! Public re-exports of asset loaders.
//!
//! These loaders are implemented in the internal `core` module but their
//! public surface (errors, constants, validators) is intentionally part of
//! the crate's stable API.

/// Strict OBJ wavefront mesh loader.
pub mod obj {
    pub use crate::core::engine::rendering::loader::obj_loader::{
        MAX_OBJ_FACE_VERTICES, MAX_OBJ_FILE_SIZE, MAX_OBJ_INDICES, MAX_OBJ_VERTICES,
        ObjLoadError, ObjLoader,
    };
}

/// Strict GLB (binary glTF 2.0) container loader.
pub mod glb {
    pub use crate::core::engine::rendering::loader::glb_loader::{
        GLB_CHUNK_TYPE_BIN, GLB_CHUNK_TYPE_JSON, GLB_HEADER_SIZE, GLB_MAGIC,
        GLB_SUPPORTED_VERSION, GlbHeader, GlbLoadError, GlbLoader, MAX_GLB_CHUNK_SIZE,
        MAX_GLB_FILE_SIZE, iter_glb_chunks, validate_glb_header,
    };
}
