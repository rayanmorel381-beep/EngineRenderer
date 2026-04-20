
//! Volumetric light transport: medium, phase functions, and ray-march.
//!
//! * [`medium`]   — [`VolumetricMedium`] with density sampling, transmittance,
//!   phase functions, and full inscattering ray-march.
//! * [`god_rays`] — screen-space light shaft (god-ray) post-effect.

pub mod god_rays;
pub mod medium;
