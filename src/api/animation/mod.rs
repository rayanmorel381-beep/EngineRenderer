/// API haut niveau pour le rendu d'animations.
pub mod animation_api;

/// Clip d'animation et états interpolables.
pub use crate::core::animation::clip::{AnimationClip, CameraFrame, SkyFrame, SunFrame};
/// Timeline et interpolation.
pub use crate::core::animation::timeline::{Interpolation, Keyframe, Lerp, Timeline};
/// Résultats de séquences.
pub use crate::core::animation::sequence::{FrameResult, SequenceResult};
/// Encodeur vidéo et erreur associée.
pub use crate::core::animation::video::{VideoExporter, FfmpegNotFound};
