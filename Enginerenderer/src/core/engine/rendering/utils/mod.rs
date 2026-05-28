//! Pure-function rendering math: interpolation, color, Fresnel, noise,
//! geometry and tone mapping helpers.

mod color;
mod fresnel;
mod geometry;
mod interpolation;
mod noise;
mod tonemap;

pub use color::{
    color_temperature, hsv_to_rgb, linear_to_srgb, luminance, rgb_to_hsv, srgb_to_linear,
};
pub use fresnel::{fresnel_dielectric, fresnel_schlick, fresnel_schlick_vec};
pub use geometry::{
    barycentric, build_tangent_frame, cartesian_to_spherical, reflect, spherical_to_cartesian,
    triangle_area,
};
pub use interpolation::{
    bias, gain, inverse_lerp, lerp, quintic_smooth, remap, saturate, smoothstep,
};
pub use noise::{fbm_3d, value_noise_3d};
pub use tonemap::{
    aces_tonemap, ev100_from_luminance, exposure_from_ev100, reinhard_extended, uncharted2_tonemap,
};
