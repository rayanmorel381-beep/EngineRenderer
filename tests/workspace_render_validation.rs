//! Campagne de validation de rendu au niveau workspace.

use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};

use enginerenderer::api::engine::diagnostics::{ComputeArch, ComputeOs, ComputeVendor, DiagnosticOverrides, DiagnosticsOptions};
use enginerenderer::api::engine::rendering::Vec3;
use enginerenderer::api::EngineApi;
use enginerenderer::api::types::core::RenderRequest;

fn unique_dir(tag: &str) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock drift")
        .as_nanos();
    std::env::temp_dir().join(format!("enginerenderer_{tag}_{stamp}"))
}

fn percentile(sorted: &[u128], p: f64) -> u128 {
    let idx = ((sorted.len() - 1) as f64 * p.clamp(0.0, 1.0)).round() as usize;
    sorted[idx]
}

fn build_scene_a(api: &EngineApi) -> enginerenderer::api::scenes::builder::SceneBuilder {
    api.scene()
    .add_sphere_named(Vec3::new(0.0, 0.0, 0.0), 1.2, "plastic-red")
    .add_sphere_named(Vec3::new(-2.0, 0.3, 1.5), 0.7, "glass")
    .add_sphere_named(Vec3::new(2.2, -0.2, -1.4), 0.9, "metal-brushed")
        .camera_position([6.0, 3.0, 7.0], [0.0, 0.0, 0.0])
        .camera_fov(48.0)
        .auto_frame()
}

fn build_scene_b(api: &EngineApi) -> enginerenderer::api::scenes::builder::SceneBuilder {
    api.scene()
    .add_sphere_named(Vec3::new(-1.0, 0.0, 0.0), 1.0, "plastic-blue")
    .add_sphere_named(Vec3::new(1.4, 0.1, 0.5), 0.55, "metal-gold")
    .add_sphere_named(Vec3::new(0.6, 1.2, -1.0), 0.45, "emissive")
        .camera_position([5.0, 4.0, 6.0], [0.0, 0.2, 0.0])
        .camera_fov(42.0)
        .auto_frame()
}

fn build_scene_c(api: &EngineApi) -> enginerenderer::api::scenes::builder::SceneBuilder {
    api.scene()
    .add_sphere_named(Vec3::new(-1.8, -0.1, 0.8), 0.8, "plastic-green")
    .add_sphere_named(Vec3::new(0.0, 0.0, 0.0), 1.25, "metal-silver")
    .add_sphere_named(Vec3::new(1.7, 0.4, -0.9), 0.65, "glass")
        .camera_position([7.5, 3.4, 5.5], [0.0, 0.0, 0.0])
        .camera_fov(50.0)
        .auto_frame()
}

#[test]
fn visual_regression_preview_is_deterministic() {
    let api = EngineApi::new();
    let out_dir = unique_dir("visual_regression");
    fs::create_dir_all(&out_dir).expect("create output dir");

    let request_a = RenderRequest::preview()
        .with_resolution(96, 54)
        .with_output(out_dir.clone(), "visual_a.ppm");
    let request_b = request_a.clone().with_output(out_dir.clone(), "visual_b.ppm");

    let scene_a = build_scene_a(&api);
    let scene_b = build_scene_a(&api);

    let result_a = api.render(scene_a, &request_a).expect("render first frame");
    let result_b = api.render(scene_b, &request_b).expect("render second frame");

    let bytes_a = fs::read(&result_a.output_path).expect("read first frame");
    let bytes_b = fs::read(&result_b.output_path).expect("read second frame");

    assert_eq!(bytes_a, bytes_b);

    let _ = fs::remove_dir_all(&out_dir);
}

#[test]
fn stress_parallel_preview_renders_complete() {
    let workers = 4usize;
    let passes_per_worker = 4usize;
    let out_dir = unique_dir("stress_parallel");
    fs::create_dir_all(&out_dir).expect("create output dir");

    let failures = Arc::new(Mutex::new(Vec::new()));
    let mut handles = Vec::new();

    for worker in 0..workers {
        let out_dir = out_dir.clone();
        let failures = Arc::clone(&failures);
        handles.push(thread::spawn(move || {
            let api = EngineApi::new();
            for pass in 0..passes_per_worker {
                let name = format!("stress_w{worker}_p{pass}.ppm");
                let request = RenderRequest::preview()
                    .with_resolution(96, 54)
                    .with_output(out_dir.clone(), name);
                let scene = if pass % 3 == 0 {
                    build_scene_a(&api)
                } else if pass % 3 == 1 {
                    build_scene_b(&api)
                } else {
                    build_scene_c(&api)
                };
                if let Err(err) = api.render(scene, &request) {
                    failures.lock().expect("lock failures").push(err.to_string());
                }
            }
        }));
    }

    for handle in handles {
        handle.join().expect("worker join");
    }

    let failures = failures.lock().expect("lock failures");
    assert!(failures.is_empty(), "stress failures: {failures:?}");

    let _ = fs::remove_dir_all(&out_dir);
}

#[test]
#[ignore]
fn long_run_frame_pacing_campaign_has_controlled_tail() {
    let api = EngineApi::new();
    let out_dir = unique_dir("long_run_pacing");
    fs::create_dir_all(&out_dir).expect("create output dir");

    let mut samples: Vec<u128> = Vec::new();

    for i in 0..20usize {
        let scene = match i % 3 {
            0 => build_scene_a(&api),
            1 => build_scene_b(&api),
            _ => build_scene_c(&api),
        };

        let request = RenderRequest::preview()
            .with_resolution(128, 72)
            .with_output(out_dir.clone(), format!("campaign_{i:03}.ppm"));

        let result = api.render(scene, &request).expect("campaign render");
        samples.push(result.duration_ms.max(1));
    }

    samples.sort_unstable();

    let p50 = percentile(&samples, 0.50) as f64;
    let p95 = percentile(&samples, 0.95) as f64;
    let p99 = percentile(&samples, 0.99) as f64;

    let mean = samples.iter().copied().map(|v| v as f64).sum::<f64>() / samples.len() as f64;
    let variance = samples
        .iter()
        .map(|v| {
            let d = *v as f64 - mean;
            d * d
        })
        .sum::<f64>()
        / samples.len() as f64;
    let jitter = variance.sqrt();

    assert!(p95 <= p50 * 2.2, "p95 too high: p50={p50:.2} p95={p95:.2}");
    assert!(p99 <= p50 * 3.0, "p99 too high: p50={p50:.2} p99={p99:.2}");
    assert!(jitter <= mean * 0.95, "jitter too high: mean={mean:.2} jitter={jitter:.2}");

    let _ = fs::remove_dir_all(&out_dir);
}

#[test]
#[ignore]
fn diagnostics_benchmark_campaign_matrix_is_available() {
    let api = EngineApi::new();
    let overrides = [
        DiagnosticOverrides {
            arch: Some(ComputeArch::X86),
            os: Some(ComputeOs::Linux),
            vendor: Some(ComputeVendor::Amd),
        },
        DiagnosticOverrides {
            arch: Some(ComputeArch::X86),
            os: Some(ComputeOs::Windows),
            vendor: Some(ComputeVendor::Intel),
        },
        DiagnosticOverrides {
            arch: Some(ComputeArch::Arm),
            os: Some(ComputeOs::Macos),
            vendor: Some(ComputeVendor::Apple),
        },
    ];

    for item in overrides {
        let report = api.compute_environment_report(&DiagnosticsOptions {
            bench: true,
            overrides: item,
            ..DiagnosticsOptions::default()
        });

        let bench = report.benchmark.expect("benchmark present");
        assert!(bench.iterations >= 128);
        assert!(bench.total_ms > 0);
        assert!(bench.avg_us > 0);
    }
}
