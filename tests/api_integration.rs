//! Integration tests for the high-level EngineApi: scenes, cameras,
//! materials, objects, and rendering.

use enginerenderer::api::engine::EngineApi;
use enginerenderer::api::types::{Quality, RenderRequest};
use enginerenderer::api::objects::SceneObject;
use enginerenderer::api::scenes::presets;
use enginerenderer::api::materials::{MaterialBuilder, PhysicsConfig, Spectrum, shortcuts};

// ── EngineApi basics ────────────────────────────────────────────────────

#[test]
fn engine_api_capabilities() {
    let api = EngineApi::new();
    let caps = api.capabilities();
    assert!(caps.max_resolution.0 > 0);
}

#[test]
fn engine_api_material_names_non_empty() {
    let api = EngineApi::new();
    let names = api.material_names();
    assert!(!names.is_empty());
    // All catalog names are resolvable
    let catalog = api.materials();
    for &name in names {
        let _mat = catalog.by_name(name);
    }
}

// ── Scene presets ───────────────────────────────────────────────────────

#[test]
fn scene_preset_test_single_sphere_builds() {
    let builder = presets::test_single_sphere();
    let (_scene, _camera) = builder.build(16.0 / 9.0);
}

#[test]
fn scene_preset_deep_space_builds() {
    let builder = presets::deep_space();
    let (_scene, _camera) = builder.build(16.0 / 9.0);
}

#[test]
fn scene_preset_golden_hour_builds() {
    let builder = presets::golden_hour();
    let (_scene, _camera) = builder.build(16.0 / 9.0);
}

#[test]
fn scene_preset_foggy_builds() {
    let builder = presets::foggy();
    let (_scene, _camera) = builder.build(16.0 / 9.0);
}

// ── Camera presets ──────────────────────────────────────────────────────

#[test]
fn camera_presets_produce_valid_cameras() {
    let api = EngineApi::new();
    let radius = 50.0;
    let cam_cin = api.camera_cinematic(radius);
    let cam_front = api.camera_front(radius);
    let cam_top = api.camera_top_down(radius);
    let cam_dram = api.camera_dramatic(radius);
    let cam_close = api.camera_closeup([0.0, 0.0, 0.0], 5.0);

    // All should produce cameras with finite positions
    for cam in [cam_cin, cam_front, cam_top, cam_dram, cam_close] {
        let desc = cam.descriptor();
        assert!(desc.fov_degrees > 0.0);
        assert!(desc.eye.iter().all(|v| v.is_finite()));
    }
}

// ── Object factories ────────────────────────────────────────────────────

#[test]
fn object_primitives_create_successfully() {
    let api = EngineApi::new();
    let _star = api.star([0.0, 0.0, 0.0], 5.0);
    let _planet = api.planet([10.0, 0.0, 0.0], 2.0);
    let _ocean = api.ocean_planet([20.0, 0.0, 0.0], 2.0);
    let _ice = api.ice_planet([30.0, 0.0, 0.0], 2.0);
    let _moon = api.moon([15.0, 3.0, 0.0], 0.5);
    let _bh = api.black_hole([0.0, 0.0, -50.0], 3.0);
}

#[test]
fn object_composites_create_successfully() {
    let api = EngineApi::new();
    let _system = api.solar_system([0.0, 0.0, 0.0], 5.0, 4);
    let _tree = api.tree([0.0, 0.0, 0.0], 3.0);
    let _house = api.house([10.0, 0.0, 0.0], 5.0);
    let _car = api.car([20.0, 0.0, 0.0], 4.0);
}

#[test]
fn scene_object_into_primitives() {
    let obj = SceneObject::star([0.0, 0.0, 0.0], 5.0);
    let (spheres, _triangles) = obj.into_primitives();
    assert!(!spheres.is_empty());
}

// ── Materials ───────────────────────────────────────────────────────────

#[test]
fn material_builder_produces_valid_material() {
    let mat = MaterialBuilder::new()
        .albedo_rgb(0.8, 0.2, 0.1)
        .roughness(0.5)
        .metallic(0.0)
        .build();
    // Should not panic
    let _ = format!("{:?}", mat);
}

#[test]
fn material_shortcuts_produce_valid_materials() {
    let _d = shortcuts::diffuse(0.5, 0.3, 0.1, 0.8);
    let _m = shortcuts::metal(0.9, 0.9, 0.9, 0.1, 0.95);
    let _g = shortcuts::dielectric(1.0, 1.0, 1.0, 1.5, 0.9, 0.0);
    let _e = shortcuts::emissive(1.0, 0.8, 0.3, 5.0);
    let _s = shortcuts::subsurface(0.8, 0.4, 0.3, 0.5, 0.5);
    let _c = shortcuts::clearcoat(0.1, 0.1, 0.8, 0.9, 0.3);
    let _i = shortcuts::iridescent(0.5, 0.5, 0.5, 0.7, 0.3);
    let _a = shortcuts::anisotropic(0.8, 0.8, 0.8, 0.3, 0.6);
}

#[test]
fn material_spectrum_round_trip() {
    let spec = Spectrum::from_wavelength(550.0, 1.0, 30.0);
    let rgb = spec.to_rgb();
    assert!(rgb.x >= 0.0 && rgb.y >= 0.0 && rgb.z >= 0.0);
    let _back = Spectrum::from_rgb(rgb);
}

#[test]
fn material_physics_config_defaults() {
    let phys = PhysicsConfig::default();
    assert!(phys.ior > 0.0);
}

// ── Rendering requests ──────────────────────────────────────────────────

#[test]
fn render_request_aspect_ratio() {
    let req = RenderRequest::hd();
    assert!(req.aspect_ratio() > 0.0);
    let req2 = RenderRequest::preview();
    assert!(req2.width > 0 && req2.height > 0);
}

// ── Full render through API ─────────────────────────────────────────────

#[test]
fn api_render_test_sphere_preview() {
    let api = EngineApi::new();
    let scene = presets::test_single_sphere();
    let request = api.request_custom(160, 90, Quality::Preview);
    let result = api.render(scene, &request).expect("render failed");
    assert!(result.rendered_pixels > 0);
    assert!(result.output_path.exists());
    assert_eq!(result.rendered_pixels, result.width * result.height);
}

#[test]
fn api_render_showcase() {
    let api = EngineApi::new();
    let request = api.request_custom(160, 90, Quality::Preview);
    let result = api.render_showcase(&request).expect("render_showcase failed");
    assert!(result.rendered_pixels > 0);
}

#[test]
fn api_render_custom_scene() {
    let api = EngineApi::new();
    let scene = api
        .scene()
        .add_object(api.star([0.0, 0.0, 0.0], 5.0))
        .add_object(api.planet([15.0, 0.0, 0.0], 2.0))
        .auto_frame();
    let request = api.request_custom(320, 180, Quality::Preview);
    let result = api.render(scene, &request).expect("custom render failed");
    assert!(result.rendered_pixels == 320 * 180);
}
