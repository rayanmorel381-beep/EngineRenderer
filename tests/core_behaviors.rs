use enginerenderer::api::engine::scenes::CameraManager;
use enginerenderer::api::engine::rendering::{luminance, smoothstep};
use enginerenderer::api::objects::primitives::Vec3;
use enginerenderer::api::engine::objects::CoreLodManager;

#[test]
fn larger_scene_pushes_camera_back() {
    let small = CameraManager::cinematic_for_scene(Vec3::ZERO, 2.0);
    let large = CameraManager::cinematic_for_scene(Vec3::ZERO, 12.0);

    assert!(large.distance_to_focus() > small.distance_to_focus());
}

#[test]
fn smoothstep_stays_in_unit_interval() {
    let value = smoothstep(0.0, 1.0, 0.4);
    assert!((0.0..=1.0).contains(&value));
}

#[test]
fn luminance_is_positive_for_bright_colors() {
    assert!(luminance(Vec3::new(1.0, 0.8, 0.6)) > 0.5);
}

#[test]
fn nearby_objects_receive_more_detail() {
    let manager = CoreLodManager::default();
    let near = manager.select(3.0, 1.2);
    let far = manager.select(80.0, 1.2);

    assert!(near.primary_samples > far.primary_samples);
    assert!(near.max_bounces >= far.max_bounces);
    assert!(near.texture_frequency > far.texture_frequency);
}
