//! Standalone GPU render demo: 1280x720, 256 SPP, 8 bounces, area lights,
//! ACES tonemap + sRGB gamma, optional bilateral denoise.
//!
//! Run with: `cargo run --release --example gpu_render_demo`.

#[cfg(not(target_os = "linux"))]
fn main() {
    eprintln!("gpu_render_demo only supports Linux desktop (GLX 4.3+)");
}

#[cfg(target_os = "linux")]
fn main() {
    use enginerenderer::api::engine::rendering::{
        gpu_try_new_desktop, AreaLight, DirectionalLight, GpuRenderConfig, Image, Material,
        ProceduralEnvironment, RenderCamera, Scene, Sphere, Vec3, VolumetricMedium,
    };
    use std::time::Instant;

    let width: u32 = 1280;
    let height: u32 = 720;
    let samples: u32 = 64;
    let max_bounces: u32 = 4;
    let exposure: f32 = 0.9;
    let denoise: bool = false;

    println!("==> creating offscreen GLX 4.3 context {width}x{height}");
    let (mut tracer, _ctx) = gpu_try_new_desktop(width, height)
        .unwrap_or_else(|e| panic!("could not create GPU context: {e}"));
    let dev = tracer.device();
    println!(
        "GPU device: vendor={:?} renderer={:?} version={:?}",
        dev.vendor, dev.renderer, dev.version
    );

    let lambert = Material::new(Vec3::new(0.85, 0.32, 0.28), 0.85, 0.0, 0.05, Vec3::ZERO);
    let metal = Material::new(Vec3::new(0.95, 0.85, 0.55), 0.10, 1.0, 0.95, Vec3::ZERO);
    let glass = Material::new(Vec3::new(0.95, 0.97, 0.99), 0.02, 0.0, 0.50, Vec3::ZERO)
        .with_transmission(1.0, 1.5);
    let chrome = Material::new(Vec3::new(0.78, 0.82, 0.88), 0.05, 1.0, 1.0, Vec3::ZERO);
    let emerald = Material::new(Vec3::new(0.18, 0.78, 0.42), 0.30, 0.0, 0.30, Vec3::ZERO);
    let ground = Material::new(Vec3::new(0.18, 0.20, 0.22), 1.0, 0.0, 0.02, Vec3::ZERO);
    let lamp = Material::new(
        Vec3::new(1.0, 0.92, 0.78),
        1.0,
        0.0,
        0.0,
        Vec3::new(8.0, 6.5, 4.5),
    );

    let scene = Scene {
        objects: vec![
            Sphere {
                center: Vec3::new(-2.6, 0.0, -3.2),
                radius: 0.7,
                material: lambert,
            },
            Sphere {
                center: Vec3::new(-1.1, 0.0, -3.2),
                radius: 0.7,
                material: metal,
            },
            Sphere {
                center: Vec3::new(0.4, 0.0, -3.2),
                radius: 0.7,
                material: glass,
            },
            Sphere {
                center: Vec3::new(1.9, 0.0, -3.2),
                radius: 0.7,
                material: chrome,
            },
            Sphere {
                center: Vec3::new(3.4, 0.0, -3.2),
                radius: 0.7,
                material: emerald,
            },
            Sphere {
                center: Vec3::new(0.4, 2.4, -2.6),
                radius: 0.35,
                material: lamp,
            },
            Sphere {
                center: Vec3::new(0.0, -101.0, -3.2),
                radius: 100.3,
                material: ground,
            },
        ],
        triangles: vec![],
        sun: DirectionalLight {
            direction: Vec3::new(-0.45, -1.0, -0.30).normalize(),
            color: Vec3::new(1.0, 0.97, 0.92),
            intensity: 4.0,
            angular_radius: 0.0095,
        },
        area_lights: vec![
            AreaLight {
                position: Vec3::new(-2.5, 3.2, -1.0),
                u: Vec3::new(1.4, 0.0, 0.0),
                v: Vec3::new(0.0, 0.0, 1.4),
                color: Vec3::new(1.0, 0.85, 0.65),
                intensity: 5.0,
            },
            AreaLight {
                position: Vec3::new(3.0, 2.8, -1.2),
                u: Vec3::new(1.0, 0.0, 0.0),
                v: Vec3::new(0.0, 0.0, 1.0),
                color: Vec3::new(0.55, 0.75, 1.0),
                intensity: 4.0,
            },
        ],
        sky_top: Vec3::new(0.18, 0.32, 0.62),
        sky_bottom: Vec3::new(0.55, 0.65, 0.78),
        exposure: 1.0,
        volume: VolumetricMedium::vacuum(),
        hdri: Some(ProceduralEnvironment::cinematic_space()),
        solar_elevation: 0.6,
    };

    let aspect = width as f64 / height as f64;
    let camera = RenderCamera::look_at(
        Vec3::new(0.4, 0.6, 1.0),
        Vec3::new(0.4, 0.0, -3.2),
        Vec3::new(0.0, 1.0, 0.0),
        45.0,
        aspect,
    );

    let cfg = GpuRenderConfig {
        width,
        height,
        samples,
        max_bounces,
        seed: 0xDECAFC0D,
        exposure,
        denoise,
    };

    println!(
        "==> dispatching compute: {width}x{height}, {samples} spp, {max_bounces} bounces, {} pixels, {} primary rays",
        width as u64 * height as u64,
        width as u64 * height as u64 * samples as u64
    );
    let start = Instant::now();
    let fb = tracer
        .render(&scene, &camera, cfg)
        .unwrap_or_else(|e| panic!("GPU render failed: {e}"));
    let elapsed = start.elapsed();
    let total_rays = width as u64 * height as u64 * samples as u64;
    let rays_per_sec = total_rays as f64 / elapsed.as_secs_f64();
    println!(
        "==> render done in {:.3}s  ({:.2} M primary rays/s, includes upload + dispatch + readback)",
        elapsed.as_secs_f64(),
        rays_per_sec / 1.0e6
    );

    let out_dir = std::env::temp_dir().join("enginerenderer_gpu_demo");
    std::fs::create_dir_all(&out_dir).ok();
    let ppm = out_dir.join(format!("gpu_render_{width}x{height}.ppm"));
    let image = Image {
        width: fb.width,
        height: fb.height,
        pixels: fb.color.clone(),
    };
    image.save_ppm(&ppm).expect("save_ppm failed");
    println!("==> wrote {}", ppm.display());

    let png = out_dir.join(format!("gpu_render_{width}x{height}.png"));
    let convert = std::process::Command::new("convert")
        .arg(&ppm)
        .arg(&png)
        .status();
    match convert {
        Ok(s) if s.success() => println!("==> wrote {}", png.display()),
        Ok(_) => eprintln!("convert exited non-zero; PPM is still available"),
        Err(e) => eprintln!("ImageMagick `convert` not found ({e}); use the PPM directly"),
    }
}
