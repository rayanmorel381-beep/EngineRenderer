//! Vérifie l'accès au catalogue de matériaux.

use enginerenderer::api::EngineApi;

#[test]
fn material_catalog_known_name_is_accessible() {
    let api = EngineApi::new();
    let m = api.materials().by_name("stellar_surface");
    assert!(m.roughness >= 0.0);
    assert!(m.roughness <= 1.0);
}

#[test]
fn material_catalog_unknown_name_falls_back() {
    let api = EngineApi::new();
    let m = api.materials().by_name("__unknown__");
    assert!((m.albedo.x - 0.5).abs() < 1e-9);
    assert!((m.albedo.y - 0.5).abs() < 1e-9);
    assert!((m.albedo.z - 0.5).abs() < 1e-9);
}
