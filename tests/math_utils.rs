//! Tests de stabilité sur les utilitaires mathématiques.

use enginerenderer::api::engine::rendering::{bias, gain, inverse_lerp, remap, smoothstep};

#[test]
fn interpolation_helpers_stay_in_expected_ranges() {
    let s = smoothstep(0.0, 1.0, 0.25);
    let i = inverse_lerp(10.0, 20.0, 15.0);
    let r = remap(0.5, 0.0, 1.0, 10.0, 20.0);

    assert!((0.0..=1.0).contains(&s));
    assert!((i - 0.5).abs() < 1e-9);
    assert!((r - 15.0).abs() < 1e-9);
}

#[test]
fn bias_and_gain_return_finite_values() {
    let b = bias(0.42, 0.6);
    let g = gain(0.42, 0.6);
    assert!(b.is_finite());
    assert!(g.is_finite());
}
