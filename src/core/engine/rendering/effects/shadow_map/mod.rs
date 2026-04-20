
//! Shadow mapping: cascaded shadow maps, PCF filtering, contact shadows.
//!
//! * [`frustum_corners`] — camera frustum corner extraction for cascade fitting.
//! * [`light_matrix`]    — orthographic light-space projection.
//! * [`cascade`]         — cascade split computation and index lookup.
//! * [`sampling`]        — PCF and contact-shadow ray-march sampling.

pub mod cascade;
pub mod frustum_corners;
pub mod light_matrix;
pub mod sampling;
