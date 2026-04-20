use enginerenderer::api::EngineApi;

#[test]
fn engine_api_new_and_default_have_materials() {
    let a = EngineApi::new();
    let b = EngineApi::default();
    assert!(!a.material_names().is_empty());
    assert_eq!(a.material_names(), b.material_names());
}

#[test]
fn engine_api_material_names_are_unique() {
    let api = EngineApi::new();
    let names = api.material_names();
    let mut sorted = names.to_vec();
    sorted.sort_unstable();
    sorted.dedup();
    assert_eq!(sorted.len(), names.len());
}
