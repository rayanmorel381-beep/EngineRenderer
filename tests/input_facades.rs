//! Tests for input facades: Mixer, CameraRig, InputDriver.

use enginerenderer::api::engine::cameras::*;
use enginerenderer::api::engine::scenes::*;
use enginerenderer::api::objects::primitives::Vec3;

// ── Mixer ───────────────────────────────────────────────────────────────

#[test]
fn mixer_cinematic_produces_valid_gain() {
    let bodies = CelestialBodies::showcase();
    let graph = SceneGraph::from_bodies(&bodies);
    let mixer = Mixer::cinematic();
    assert!(!mixer.label().is_empty());
    let result = mixer.mix(&graph, 100.0, 1.0);
    assert!(result.master_gain >= 0.0);
}

#[test]
fn mixer_preview_produces_valid_gain() {
    let bodies = CelestialBodies::showcase();
    let graph = SceneGraph::from_bodies(&bodies);
    let mixer = Mixer::preview();
    assert!(!mixer.label().is_empty());
    let result = mixer.mix(&graph, 100.0, 0.0);
    assert!(result.master_gain >= 0.0);
}

#[test]
fn mixer_custom_gain() {
    let mixer = Mixer::new(0.6);
    assert!(!mixer.label().is_empty());
}

// ── CameraRig ───────────────────────────────────────────────────────────

#[test]
fn camera_rig_cinematic_builds_camera() {
    let rig = CameraRig::cinematic(50.0);
    let _cam = rig.build_camera(16.0 / 9.0, 0.0);
    assert!(rig.distance_to_focus() > 0.0);
    assert!(rig.inner().distance_to_focus() >= 0.0);
}

#[test]
fn camera_rig_focused_on_and_reframe() {
    let mut focused = CameraRig::focused_on(Vec3::ZERO, 50.0);
    assert!(focused.distance_to_focus() > 0.0);
    focused.reframe(Vec3::ZERO, 75.0);
    assert!(focused.distance_to_focus() > 0.0);
}

// ── InputDriver ─────────────────────────────────────────────────────────

#[test]
fn input_driver_cinematic_mode() {
    let driver = InputDriver::cinematic();
    assert!(matches!(driver.mode(), InputMode::Cinematic));
    let sample = driver.sample(0.5);
    assert!(sample.time_scale > 0.0);
}

#[test]
fn input_driver_manual_mode() {
    let manual = InputDriver::manual();
    assert!(matches!(manual.mode(), InputMode::Manual));
    let custom = InputDriver::new(InputMode::Manual);
    assert!(matches!(custom.mode(), InputMode::Manual));
}
