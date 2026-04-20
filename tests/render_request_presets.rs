use enginerenderer::api::types::core::{Quality, RenderRequest};

#[test]
fn render_request_presets_have_expected_dimensions() {
    let p = RenderRequest::preview();
    let h = RenderRequest::hd();
    let pr = RenderRequest::production();

    assert_eq!((p.width, p.height, p.quality), (1280, 720, Quality::Preview));
    assert_eq!((h.width, h.height, h.quality), (1920, 1080, Quality::Hd));
    assert_eq!((pr.width, pr.height, pr.quality), (2560, 1440, Quality::Production));
}

#[test]
fn render_request_default_is_hd() {
    let d = RenderRequest::default();
    let h = RenderRequest::hd();
    assert_eq!(d.width, h.width);
    assert_eq!(d.height, h.height);
    assert_eq!(d.quality, h.quality);
}
