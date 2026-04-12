//! Post-processing pipeline: blur kernels, per-pixel effects,
//! depth-of-field, and the orchestrating [`PostProcessor`].

pub mod blur;
pub mod depth_of_field;
pub mod effects;
pub mod processor;
