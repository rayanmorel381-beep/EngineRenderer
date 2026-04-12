use std::{fs, path::PathBuf};

use enginerenderer::api::engine::scenes::{EngineConfig, CameraManager, EngineScene, CelestialBodies, SceneGraph, ResourceManager};
use enginerenderer::api::objects::primitives::{Renderer, RenderPreset};

#[test]
fn showcase_scene_contains_realistic_environment_assets() {
    let bodies = CelestialBodies::showcase();
    let graph = SceneGraph::from_bodies(&bodies);
    let camera_manager = CameraManager::cinematic_for_scene(graph.focus_point(), graph.scene_radius());
    let config = EngineConfig::ultra_hd_cpu();
    let resource_manager = ResourceManager::from_config(&config);

    let scene = EngineScene::from_bodies(
        &bodies,
        &camera_manager,
        &resource_manager,
        graph,
        16.0 / 9.0,
        0.0,
    );

    assert!(scene.scene.objects.len() >= bodies.bodies().len() + 20);
    assert!(scene.scene.triangles.len() >= 500);
    assert!(scene.scene.area_lights.len() >= 5);
}

#[test]
fn gallery_shots_cover_requested_subjects() {
    let names = EngineScene::dedicated_gallery_shots()
        .into_iter()
        .map(|shot| shot.name)
        .collect::<Vec<_>>();

    assert!(names.contains(&"car"));
    assert!(names.contains(&"tree"));
    assert!(names.contains(&"house"));
    assert!(names.contains(&"planet"));
    assert!(names.contains(&"sun"));
    assert!(names.contains(&"black_hole"));
}

#[test]
fn preview_render_outputs_realistic_showcase_frame() {
    fs::create_dir_all("output").expect("output directory should be creatable");

    let shot = EngineScene::dedicated_gallery_shots()
        .into_iter()
        .find(|candidate| candidate.name == "car")
        .expect("car showcase must exist");

    let renderer = Renderer::with_resolution(80, 45);
    let output_path = PathBuf::from("output/test_showcase_preview.ppm");
    let report = renderer
        .render_scene_to_file(&shot.scene, &shot.camera, &output_path, RenderPreset::PreviewCpu)
        .expect("preview render should succeed");

    assert!(output_path.exists());
    assert!(report.rendered_pixels >= 80 * 45);
    assert!(report.object_count >= 1);
}

#[test]
fn gallery_renders_all_six_subjects() {
    fs::create_dir_all("output").expect("output directory should be creatable");

    let shots = EngineScene::dedicated_gallery_shots();
    assert_eq!(shots.len(), 6);

    let renderer = Renderer::with_resolution(80, 45);

    for shot in &shots {
        let output_path = PathBuf::from(format!("output/test_gallery_{}.ppm", shot.name));
        let report = renderer
            .render_scene_to_file(&shot.scene, &shot.camera, &output_path, RenderPreset::PreviewCpu)
            .expect(&format!("gallery render for '{}' should succeed", shot.name));

        assert!(output_path.exists(), "gallery '{}' not written", shot.name);
        assert!(report.rendered_pixels >= 80 * 45, "gallery '{}' missing pixels", shot.name);
    }
}
