//! Tests for N-body simulation.

use enginerenderer::api::engine::scenes::*;

#[test]
fn nbody_showcase_advance() {
    let mut nbody = NBodySystem::showcase();
    nbody.advance(0.01, 1);
    let center = nbody.scene_center();
    let radius = nbody.scene_radius();
    assert!(radius > 0.0);
    assert!(center.length().is_finite());
}

#[test]
fn nbody_bodies_accessor() {
    let nbody = NBodySystem::showcase();
    let bodies = nbody.bodies();
    assert!(!bodies.is_empty());
    let total_kinetic: f64 = bodies
        .iter()
        .map(|b| 0.5 * b.mass * b.velocity.length().powi(2))
        .sum();
    assert!(total_kinetic >= 0.0);
}

#[test]
fn nbody_to_scene() {
    let nbody = NBodySystem::showcase();
    let scene = nbody.to_scene();
    assert!(!scene.objects.is_empty());
}

#[test]
fn gravity_constant_positive() {
    const { assert!(GRAVITY > 0.0) };
}
