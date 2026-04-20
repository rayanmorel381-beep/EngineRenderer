use enginerenderer::api::types::core::{Quality, RenderRequest};

#[test]
fn with_resolution_clamps_min_and_max() {
    let min_clamped = RenderRequest::preview().with_resolution(1, 2);
    let max_clamped = RenderRequest::preview().with_resolution(8000, 9000);

    assert_eq!((min_clamped.width, min_clamped.height), (64, 64));
    assert_eq!((max_clamped.width, max_clamped.height), (3840, 2160));
}

#[test]
fn with_output_and_quality_updates_values() {
    let r = RenderRequest::preview()
        .with_quality(Quality::Production)
        .with_output("build/out", "frame.ppm");

    assert_eq!(r.quality, Quality::Production);
    assert_eq!(r.output_path().to_string_lossy(), "build/out/frame.ppm");
}
