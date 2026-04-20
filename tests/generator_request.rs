use enginerenderer::api::engine::rendering::GeneratorRequest;
use enginerenderer::api::types::core::Quality;

#[test]
fn generator_preview_defaults_are_stable() {
    let g = GeneratorRequest::preview();
    assert_eq!(g.width, 1280);
    assert_eq!(g.height, 720);
    assert_eq!(g.quality, Quality::Preview);
    assert_eq!(g.frame_prefix, "frame");
}

#[test]
fn generator_to_render_request_preserves_resolution_and_quality() {
    let mut g = GeneratorRequest::preview();
    g.width = 1920;
    g.height = 1080;
    g.quality = Quality::Hd;

    let r = g.to_render_request();
    assert_eq!(r.width, 1920);
    assert_eq!(r.height, 1080);
    assert_eq!(r.quality, Quality::Hd);
}
