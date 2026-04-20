use enginerenderer::api::types::core::{OutputFormat, RenderRequest};

#[test]
fn output_format_extension_values_are_stable() {
    assert_eq!(OutputFormat::Ppm.extension(), "ppm");
    assert_eq!(OutputFormat::Png.extension(), "png");
    assert_eq!(OutputFormat::Exr.extension(), "exr");
}

#[test]
fn with_format_rewrites_file_extension_only() {
    let req = RenderRequest::preview()
        .with_output("output", "shot.custom")
        .with_format(OutputFormat::Exr);
    assert_eq!(req.file_name, "shot.exr");
    assert_eq!(req.output_path().to_string_lossy(), "output/shot.exr");
}
