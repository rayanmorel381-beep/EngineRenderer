pub mod animation_api;

pub use crate::core::animation::clip::{AnimationClip, CameraFrame, SkyFrame, SunFrame};
pub use crate::core::animation::timeline::{Interpolation, Keyframe, Lerp, Timeline};
pub use crate::core::animation::sequence::{FrameResult, FrameSequencer, SequenceResult};
pub use crate::core::animation::video::{VideoExporter, FfmpegNotFound};
