//! Smoke test for the GPU compute path tracer.
//!
//! Renders a 64×64 scene with three spheres (lambert, metal, glass) lit by
//! a directional sun on a Linux GLX 4.3+ desktop, reads the framebuffer
//! back, asserts every pixel is finite, and asserts the image is not
//! constant (i.e. the shader actually shaded something).
//!
//! The test is skipped automatically when no offscreen GL context is
//! available (headless CI, no X server, no DRI driver supporting GL 4.3).

#![cfg(target_os = "linux")]

use enginerenderer::api::engine::rendering::{
    gpu_try_new_desktop, DirectionalLight, GpuRenderConfig, Image, Material, RenderCamera,
    Scene, Sphere, Vec3, VolumetricMedium,
};

#[test]
fn gpu_raytracer_renders_three_sphere_scene() {
    let (mut tracer, _ctx) = match gpu_try_new_desktop(64, 64) {
        Ok(pair) => pair,
        Err(e) => {
            eprintln!("skipping GPU smoke test: {e}");
            return;
        }
    };

    println!(
        "GPU device: vendor={:?} renderer={:?} version={:?}",
        tracer.device().vendor,
        tracer.device().renderer,
        tracer.device().version
    );

    let lambert = Material::new(Vec3::new(0.85, 0.32, 0.28), 0.85, 0.0, 0.05, Vec3::ZERO);
    let metal = Material::new(Vec3::new(0.95, 0.85, 0.55), 0.15, 1.0, 0.95, Vec3::ZERO);
    let glass = Material::new(Vec3::new(0.95, 0.97, 0.99), 0.05, 0.0, 0.50, Vec3::ZERO)
        .with_transmission(1.0, 1.5);
    let ground = Material::new(Vec3::new(0.40, 0.42, 0.45), 1.0, 0.0, 0.02, Vec3::ZERO);

    let scene = Scene {
        objects: vec![
            Sphere {
                center: Vec3::new(-1.2, 0.0, -2.5),
                radius: 0.7,
                material: lambert,
            },
            Sphere {
                center: Vec3::new(0.0, 0.0, -2.5),
                radius: 0.7,
                material: metal,
            },
            Sphere {
                center: Vec3::new(1.2, 0.0, -2.5),
                radius: 0.7,
                material: glass,
            },
            Sphere {
                center: Vec3::new(0.0, -101.0, -2.5),
                radius: 100.0,
                material: ground,
            },
        ],
        triangles: vec![],
        sun: DirectionalLight {
            direction: Vec3::new(-0.5, -1.0, -0.4).normalize(),
            color: Vec3::new(1.0, 0.97, 0.92),
            intensity: 4.0,
            angular_radius: 0.0095,
        },
        area_lights: vec![],
        sky_top: Vec3::new(0.45, 0.65, 0.95),
        sky_bottom: Vec3::new(0.92, 0.94, 0.97),
        exposure: 1.0,
        volume: VolumetricMedium::vacuum(),
        hdri: None,
        solar_elevation: 0.6,
    };

    let camera = RenderCamera::look_at(
        Vec3::new(0.0, 0.4, 0.0),
        Vec3::new(0.0, 0.0, -2.5),
        Vec3::new(0.0, 1.0, 0.0),
        45.0,
        1.0,
    );

    let cfg = GpuRenderConfig {
        width: 64,
        height: 64,
        samples: 4,
        max_bounces: 2,
        seed: 0xCAFEBABE,
        exposure: 1.0,
        denoise: false,
    };

    let fb = tracer.render(&scene, &camera, cfg).expect("GPU render failed");

    assert_eq!(fb.width, 64);
    assert_eq!(fb.height, 64);
    assert_eq!(fb.color.len(), 64 * 64);

    let mut min_lum = f64::INFINITY;
    let mut max_lum = f64::NEG_INFINITY;
    let mut nan_count = 0u32;
    let mut sum = 0.0;
    for c in &fb.color {
        if !c.x.is_finite() || !c.y.is_finite() || !c.z.is_finite() {
            nan_count += 1;
            continue;
        }
        let lum = 0.2126 * c.x + 0.7152 * c.y + 0.0722 * c.z;
        if lum < min_lum {
            min_lum = lum;
        }
        if lum > max_lum {
            max_lum = lum;
        }
        sum += lum;
    }
    let mean = sum / fb.color.len() as f64;
    println!(
        "GPU render stats: min_lum={min_lum:.4} max_lum={max_lum:.4} mean_lum={mean:.4} nan_pixels={nan_count}"
    );

    assert_eq!(nan_count, 0, "GPU produced NaN/inf pixels");
    assert!(mean > 0.01, "image is essentially black, shader did not shade");
    assert!(
        max_lum - min_lum > 0.05,
        "image has no contrast (min={min_lum}, max={max_lum}), shader produced flat output"
    );

    let out_dir = std::env::temp_dir().join("enginerenderer_gpu_smoke");
    std::fs::create_dir_all(&out_dir).ok();
    let out_path = out_dir.join("gpu_three_spheres.ppm");
    let image = Image {
        width: fb.width,
        height: fb.height,
        pixels: fb.color.clone(),
    };
    if let Err(e) = image.save_ppm(&out_path) {
        eprintln!("warning: could not save preview PPM: {e}");
    } else {
        println!("GPU preview saved to {}", out_path.display());
    }
}
