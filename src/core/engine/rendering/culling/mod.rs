//! View-volume and visibility culling.
//!
//! * [`frustum`] — half-space planes, frustum construction, containment tests.
//! * [`helpers`] — contribution, back-face, and sphere-occlusion helpers.
//! * [`scene_culler`] — configurable multi-strategy scene culler.

pub mod frustum;
pub mod helpers;
pub mod scene_culler;
