use enginerenderer::api::EngineApi;

#[test]
fn realtime_profile_values_match_contract() {
    let api = EngineApi::new();
    let hd = api.realtime_hd();
    let mobile = api.realtime_mobile();
    let ultra = api.realtime_ultra();

    assert_eq!((hd.width, hd.height, hd.target_fps, hd.duration_seconds), (1920, 1080, 60, 10));
    assert_eq!((mobile.width, mobile.height, mobile.target_fps, mobile.duration_seconds), (640, 360, 120, 8));
    assert_eq!((ultra.width, ultra.height, ultra.target_fps, ultra.duration_seconds), (3840, 2160, 30, 5));
}
