//! Heavy GPU render demo: thousands of triangles to demonstrate BVH win.
//!
//! Run with: `cargo run --release --example gpu_render_heavy`.

#[cfg(not(target_os = "linux"))]
fn main() {
    eprintln!("gpu_render_heavy only supports Linux desktop (GLX 4.3+)");
}

#[cfg(target_os = "linux")]
fn main() {
    use enginerenderer::api::engine::rendering::{
        gpu_try_new_desktop, AreaLight, DirectionalLight, GpuRenderConfig, Image, Material,
        RenderCamera, Scene, Sphere, Triangle, Vec3, VolumetricMedium,
    };
    use std::time::Instant;

    let width: u32 = 512;
    let height: u32 = 384;
    let samples: u32 = 16;
    let max_bounces: u32 = 3;

    println!("==> creating offscreen GLX 4.3 context {width}x{height}");
    let (mut tracer, _ctx) = gpu_try_new_desktop(width, height)
        .unwrap_or_else(|e| panic!("could not create GPU context: {e}"));
    let dev = tracer.device();
    println!(
        "GPU device: vendor={:?} renderer={:?} version={:?}",
        dev.vendor, dev.renderer, dev.version
    );

    let lambert_red = Material::new(Vec3::new(0.85, 0.32, 0.28), 0.85, 0.0, 0.05, Vec3::ZERO);
    let lambert_green = Material::new(Vec3::new(0.32, 0.78, 0.42), 0.85, 0.0, 0.05, Vec3::ZERO);
    let lambert_blue = Material::new(Vec3::new(0.30, 0.45, 0.85), 0.85, 0.0, 0.05, Vec3::ZERO);
    let lambert_yellow = Material::new(Vec3::new(0.92, 0.85, 0.30), 0.85, 0.0, 0.05, Vec3::ZERO);
    let metal = Material::new(Vec3::new(0.90, 0.90, 0.92), 0.05, 1.0, 1.0, Vec3::ZERO);
    let ground = Material::new(Vec3::new(0.18, 0.20, 0.22), 1.0, 0.0, 0.02, Vec3::ZERO);

    let mut triangles: Vec<Triangle> = Vec::new();
    let grid: i32 = 8;
    let cell: f64 = 0.55;
    let palette = [lambert_red, lambert_green, lambert_blue, lambert_yellow];
    for gz in 0..grid {
        for gx in 0..grid {
            let cx = (gx as f64 - grid as f64 * 0.5) * cell;
            let cz = -3.0 - (gz as f64) * cell;
            let h = 0.15 + 0.4 * ((gx + gz * 7) as f64 * 0.37).sin().abs();
            let m = palette[((gx * 31 + gz * 17) as usize) % palette.len()];
            push_box(
                &mut triangles,
                Vec3::new(cx, -0.7, cz),
                Vec3::new(cell * 0.4, h, cell * 0.4),
                m,
            );
        }
    }

    let mut spheres = vec![
        Sphere {
            center: Vec3::new(-2.0, 0.6, -3.0),
            radius: 0.6,
            material: metal,
        },
        Sphere {
            center: Vec3::new(2.0, 0.6, -3.0),
            radius: 0.6,
            material: metal,
        },
        Sphere {
            center: Vec3::new(0.0, -101.0, -3.0),
            radius: 100.3,
            material: ground,
        },
    ];
    spheres.shrink_to_fit();

    let triangle_count = triangles.len();
    let sphere_count = spheres.len();
    let prim_count = triangle_count + sphere_count;

    let scene = Scene {
        objects: spheres,
        triangles,
        sun: DirectionalLight {
            direction: Vec3::new(-0.45, -1.0, -0.30).normalize(),
            color: Vec3::new(1.0, 0.97, 0.92),
            intensity: 4.0,
            angular_radius: 0.0095,
        },
        area_lights: vec![AreaLight {
            position: Vec3::new(-1.5, 3.5, -1.0),
            u: Vec3::new(1.5, 0.0, 0.0),
            v: Vec3::new(0.0, 0.0, 1.5),
            color: Vec3::new(1.0, 0.85, 0.65),
            intensity: 6.0,
        }],
        sky_top: Vec3::new(0.18, 0.32, 0.62),
        sky_bottom: Vec3::new(0.55, 0.65, 0.78),
        exposure: 1.0,
        volume: VolumetricMedium::vacuum(),
        hdri: None,
        solar_elevation: 0.6,
    };

    let aspect = width as f64 / height as f64;
    let camera = RenderCamera::look_at(
        Vec3::new(0.0, 1.5, 1.5),
        Vec3::new(0.0, 0.0, -3.0),
        Vec3::new(0.0, 1.0, 0.0),
        50.0,
        aspect,
    );

    let cfg = GpuRenderConfig {
        width,
        height,
        samples,
        max_bounces,
        seed: 0xBADBEEFu32,
        exposure: 0.9,
        denoise: false,
    };

    println!(
        "==> scene: {sphere_count} spheres + {triangle_count} triangles ({prim_count} prims)\n==> dispatching: {samples} spp, {max_bounces} bounces, {} primary rays",
        width as u64 * height as u64 * samples as u64
    );
    let start = Instant::now();
    let fb = tracer
        .render(&scene, &camera, cfg)
        .unwrap_or_else(|e| panic!("GPU render failed: {e}"));
    let elapsed = start.elapsed();
    let total_rays = width as u64 * height as u64 * samples as u64;
    println!(
        "==> render done in {:.3}s  ({:.2} M primary rays/s)",
        elapsed.as_secs_f64(),
        total_rays as f64 / elapsed.as_secs_f64() / 1.0e6
    );

    let out_dir = std::env::temp_dir().join("enginerenderer_gpu_demo");
    std::fs::create_dir_all(&out_dir).ok();
    let ppm = out_dir.join(format!("gpu_render_heavy_{width}x{height}.ppm"));
    let image = Image {
        width: fb.width,
        height: fb.height,
        pixels: fb.color.clone(),
    };
    image.save_ppm(&ppm).expect("save_ppm failed");
    println!("==> wrote {}", ppm.display());

    let png = out_dir.join(format!("gpu_render_heavy_{width}x{height}.png"));
    let _ = std::process::Command::new("convert")
        .arg(&ppm)
        .arg(&png)
        .status()
        .map(|s| {
            if s.success() {
                println!("==> wrote {}", png.display());
            }
        });
}

#[cfg(target_os = "linux")]
fn push_box(
    out: &mut Vec<enginerenderer::api::engine::rendering::Triangle>,
    center: enginerenderer::api::engine::rendering::Vec3,
    half_extents: enginerenderer::api::engine::rendering::Vec3,
    mat: enginerenderer::api::engine::rendering::Material,
) {
    use enginerenderer::api::engine::rendering::{Triangle, Vec3};
    let xm = center.x - half_extents.x;
    let xp = center.x + half_extents.x;
    let ym = center.y - half_extents.y;
    let yp = center.y + half_extents.y;
    let zm = center.z - half_extents.z;
    let zp = center.z + half_extents.z;
    let v = [
        Vec3::new(xm, ym, zm),
        Vec3::new(xp, ym, zm),
        Vec3::new(xp, yp, zm),
        Vec3::new(xm, yp, zm),
        Vec3::new(xm, ym, zp),
        Vec3::new(xp, ym, zp),
        Vec3::new(xp, yp, zp),
        Vec3::new(xm, yp, zp),
    ];
    let faces = [
        [0, 2, 1],
        [0, 3, 2],
        [4, 5, 6],
        [4, 6, 7],
        [0, 1, 5],
        [0, 5, 4],
        [2, 3, 7],
        [2, 7, 6],
        [1, 2, 6],
        [1, 6, 5],
        [3, 0, 4],
        [3, 4, 7],
    ];
    for f in &faces {
        out.push(Triangle::flat(v[f[0]], v[f[1]], v[f[2]], mat));
    }
}
