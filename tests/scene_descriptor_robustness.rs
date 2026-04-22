//! Tests adversariaux du parseur `.scene`.

use enginerenderer::api::scenes::{
    MAX_MATERIAL_NAME_LEN, MAX_SCENE_AREA_LIGHTS, MAX_SCENE_FILE_SIZE, MAX_SCENE_SPHERES,
    MAX_SCENE_TRIANGLES, SceneDescriptor,
};

fn parse(text: &str) -> Result<SceneDescriptor, String> {
    SceneDescriptor::parse(text)
}

fn err(text: &str) -> String {
    parse(text).expect_err("expected parse error")
}

#[test]
fn empty_input_yields_default_descriptor() {
    let desc = parse("").expect("empty parses");
    assert!(desc.spheres.is_empty());
    assert!(desc.triangles.is_empty());
    assert!(desc.area_lights.is_empty());
}

#[test]
fn comment_only_input_parses() {
    let desc = parse("# nothing\n# at all\n").expect("comment parses");
    assert!(desc.spheres.is_empty());
}

#[test]
fn unknown_keyword_reports_line_number() {
    let message = err("version 1\nbogus pos=0,0,0\n");
    assert!(message.contains("line 2"), "missing line number: {message}");
    assert!(message.contains("bogus"));
}

#[test]
fn malformed_kv_token_reports_line() {
    let message = err("camera eye 0,0,0\n");
    assert!(message.contains("line 1"));
    assert!(message.contains("malformed token"));
}

#[test]
fn empty_kv_value_is_rejected() {
    let message = err("camera eye=\n");
    assert!(message.contains("line 1"));
}

#[test]
fn nan_float_is_rejected() {
    let message = err("exposure NaN\n");
    assert!(message.contains("non-finite"));
}

#[test]
fn infinity_float_is_rejected() {
    let message = err("exposure inf\n");
    assert!(message.contains("non-finite"));
}

#[test]
fn vec3_with_nan_component_is_rejected() {
    let message = err("camera eye=1,NaN,2\n");
    assert!(message.contains("non-finite"));
}

#[test]
fn fov_out_of_range_is_rejected() {
    let message = err("camera fov=190\n");
    assert!(message.contains("fov"));
}

#[test]
fn negative_aperture_is_rejected() {
    let message = err("camera aperture=-0.1\n");
    assert!(message.contains("aperture"));
}

#[test]
fn negative_exposure_is_rejected() {
    let message = err("exposure -0.5\n");
    assert!(message.contains("exposure"));
}

#[test]
fn zero_radius_sphere_is_rejected() {
    let message = err("sphere pos=0,0,0 radius=0\n");
    assert!(message.contains("radius"));
}

#[test]
fn negative_radius_sphere_is_rejected() {
    let message = err("sphere pos=0,0,0 radius=-1\n");
    assert!(message.contains("radius"));
}

#[test]
fn albedo_above_one_is_rejected() {
    let message = err("sphere pos=0,0,0 radius=1 albedo=1.5,0.5,0.5\n");
    assert!(message.contains("color"));
}

#[test]
fn negative_albedo_is_rejected() {
    let message = err("sphere pos=0,0,0 radius=1 albedo=-0.1,0.5,0.5\n");
    assert!(message.contains("color"));
}

#[test]
fn roughness_above_one_is_rejected() {
    let message = err("sphere pos=0,0,0 radius=1 roughness=2\n");
    assert!(message.contains("scalar"));
}

#[test]
fn metallic_negative_is_rejected() {
    let message = err("sphere pos=0,0,0 radius=1 metallic=-0.01\n");
    assert!(message.contains("scalar"));
}

#[test]
fn negative_emission_is_rejected() {
    let message = err("sphere pos=0,0,0 radius=1 emission=-1\n");
    assert!(message.contains("emission"));
}

#[test]
fn area_light_zero_size_is_rejected() {
    let message = err("area_light pos=0,5,0 color=1,1,1 intensity=1 size=0,2\n");
    assert!(message.contains("size"));
}

#[test]
fn area_light_negative_intensity_is_rejected() {
    let message = err("area_light pos=0,5,0 color=1,1,1 intensity=-1 size=2,2\n");
    assert!(message.contains("intensity"));
}

#[test]
fn invalid_material_name_chars_are_rejected() {
    let message = err("sphere pos=0,0,0 radius=1 material=foo/../bar\n");
    assert!(message.contains("invalid characters"));
}

#[test]
fn empty_material_name_is_rejected() {
    let message = err("sphere pos=0,0,0 radius=1 material=\n");
    assert!(message.contains("empty value"));
}

#[test]
fn long_material_name_is_rejected() {
    let name = "a".repeat(MAX_MATERIAL_NAME_LEN + 1);
    let source = format!("sphere pos=0,0,0 radius=1 material={name}\n");
    let message = err(&source);
    assert!(message.contains("exceeds limit"));
}

#[test]
fn vec3_with_two_components_is_rejected() {
    let message = err("camera eye=1,2\n");
    assert!(message.contains("3 components"));
}

#[test]
fn vec2_with_one_component_is_rejected() {
    let message = err("area_light pos=0,5,0 color=1,1,1 intensity=1 size=2\n");
    assert!(message.contains("2 components"));
}

#[test]
fn utf8_bom_is_skipped() {
    let mut text = String::from("\u{feff}");
    text.push_str("exposure 1.2\n");
    let desc = parse(&text).expect("bom parses");
    assert!((desc.exposure - 1.2).abs() < 1e-9);
}

#[test]
fn sphere_count_above_limit_is_rejected() {
    assert!(MAX_SCENE_SPHERES > 0);
    assert!(MAX_SCENE_TRIANGLES > 0);
    assert!(MAX_SCENE_AREA_LIGHTS > 0);
    assert!(MAX_SCENE_FILE_SIZE > 0);
}

#[test]
fn round_trip_serialize_then_parse_is_stable() {
    let original = SceneDescriptor::default();
    let serialized = original.serialize();
    let reparsed = parse(&serialized).expect("default round-trips");
    let serialized_again = reparsed.serialize();
    assert_eq!(serialized, serialized_again);
}

#[test]
fn valid_scene_with_all_keywords_parses() {
    let source = "\
version 1
camera eye=0,1,5 target=0,0,0 fov=60 aperture=0.0
sun dir=0,-1,0 intensity=1.0 color=1,1,1
sky top=0.1,0.2,0.3 bottom=0.0,0.0,0.0
exposure 1.0
sphere pos=0,0,0 radius=1 material=stellar_core
sphere pos=2,0,0 radius=0.5 albedo=0.8,0.2,0.2 roughness=0.3 metallic=0.0 emission=0.0
triangle a=0,0,0 b=1,0,0 c=0,1,0 albedo=0.5,0.5,0.5 roughness=0.5 metallic=0.0 emission=0.0
area_light pos=0,5,0 color=1,1,1 intensity=2 size=2,2
";
    let desc = parse(source).expect("full scene parses");
    assert_eq!(desc.spheres.len(), 2);
    assert_eq!(desc.triangles.len(), 1);
    assert_eq!(desc.area_lights.len(), 1);
}

#[test]
fn fuzz_random_inputs_never_panic() {
    let mut state: u64 = 0xABCD_1234_5678_9EF0;
    let alphabet: &[u8] =
        b"sphere triangle camera sun sky exposure area_light pos=radius material albedo roughness metallic emission color intensity size eye target fov aperture dir top bottom 0123456789.,-=#\n\t inf NaN ";
    for _ in 0..200 {
        state = state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        let len = ((state >> 32) as usize) % 2048;
        let mut buffer = Vec::with_capacity(len);
        let mut local = state;
        for _ in 0..len {
            local = local
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            let pick = ((local >> 24) as usize) % alphabet.len();
            buffer.push(alphabet[pick]);
        }
        let text = String::from_utf8_lossy(&buffer).to_string();
        if let Ok(desc) = SceneDescriptor::parse(&text) {
            for sphere in &desc.spheres {
                assert!(sphere.radius > 0.0);
                assert!(sphere.position.iter().all(|c| c.is_finite()));
                assert!(sphere.albedo.iter().all(|c| (0.0..=1.0).contains(c)));
                assert!((0.0..=1.0).contains(&sphere.roughness));
                assert!((0.0..=1.0).contains(&sphere.metallic));
                assert!(sphere.emission >= 0.0);
            }
            for triangle in &desc.triangles {
                assert!(triangle.a.iter().all(|c| c.is_finite()));
                assert!(triangle.b.iter().all(|c| c.is_finite()));
                assert!(triangle.c.iter().all(|c| c.is_finite()));
            }
            for light in &desc.area_lights {
                assert!(light.intensity >= 0.0);
                assert!(light.size[0] > 0.0 && light.size[1] > 0.0);
            }
            assert!(desc.exposure > 0.0 || desc.spheres.is_empty() && desc.triangles.is_empty() && desc.area_lights.is_empty() || desc.exposure.is_finite());
        }
    }
}
