use std::collections::HashMap;
use std::error::Error;
use std::io::{self, IsTerminal, Write};
use std::path::PathBuf;

use enginerenderer::api::engine::Engine;
use enginerenderer::api::engine::EngineApi;
use enginerenderer::api::engine::diagnostics::{
    ComputeArch, ComputeOs, ComputeVendor, DiagnosticComponent, DiagnosticOverrides,
    DiagnosticsOptions,
};

const VERSION: &str = env!("CARGO_PKG_VERSION");
const NAME: &str = env!("CARGO_PKG_NAME");

pub fn run_cli(args: &[String]) -> Result<(), Box<dyn Error>> {
    let Some(first) = args.first().map(|s| s.as_str()) else {
        print_help();
        return Ok(());
    };

    let tail: Vec<&str> = args.iter().skip(1).map(|s| s.as_str()).collect();

    match first {
        "gallery" | "--gallery" => run_command_with_args("gallery", &tail)?,
        "test" | "--test" => run_command_with_args("test", &tail)?,
        "run" | "--run" => run_command_with_args("run", &tail)?,
        "render" | "--render" => run_command_with_args("render", &tail)?,
        "gpu-render" | "--gpu-render" => run_command_with_args("gpu-render", &tail)?,
        "validate" | "--validate" => run_command_with_args("validate", &tail)?,
        "version" | "--version" | "-V" => {
            println!("{} {}", NAME, VERSION);
        }
        "detect" | "--detect" | "debug" | "--debug" => {
            run_command_with_args("detect", &tail)?;
        }
        "help" | "--help" | "-h" => print_help(),
        other => {
            eprintln!("unknown argument: {}", other);
            print_help();
        }
    }
    Ok(())
}

pub(crate) fn print_help() {
    eprintln!("Usage: cargo run -- <command> [options]");
    eprintln!();
    eprintln!("Commands:");
    eprintln!("  help                       show this help");
    eprintln!("  version                    print version");
    eprintln!("  render [--out PATH]        standard CPU render");
    eprintln!("  gallery                    render the gallery showcase");
    eprintln!("  test                       fast smoke render");
    eprintln!(
        "  validate                   smoke render + (linux) GPU context probe; non-zero exit on failure"
    );
    eprintln!("  run [--seconds N --fps N --width N --height N]");
    eprintln!("                             realtime preview; flags skip prompts (CI-friendly)");
    eprintln!("  gpu-render [options]       GPU path-traced single frame (linux only)");
    eprintln!("    --width N --height N --spp N --bounces N --seed HEX|DEC");
    eprintln!("    --exposure F --hdri PRESET --clouds F --out PATH");
    eprintln!("    PRESET: cinematic_space | none");
    eprintln!("  detect [--json --verbose --bench --component cpu|gpu|ram|display]");
    eprintln!("         [--override arch=x86,os=linux,vendor=amd]");
    eprintln!();
    eprintln!("Interactive mode: launch with no arguments.");
}

pub(crate) fn run_command(cmd: &str) -> Result<(), Box<dyn Error>> {
    run_command_with_args(cmd, &[])
}

pub(crate) fn run_command_with_args(cmd: &str, args: &[&str]) -> Result<(), Box<dyn Error>> {
    match cmd {
        "help" => print_help(),
        "version" => println!("{} {}", NAME, VERSION),
        "gallery" => {
            let reports = Engine::production_reference().render_gallery()?;
            eprintln!("gallery renders:");
            for report in reports {
                eprintln!("  {}", report.output_path.display());
            }
        }
        "test" => {
            let report = Engine::test_minimal().run()?;
            eprintln!("test render -> {}", report.output_path.display());
        }
        "run" => {
            let flags = parse_flags(args)?;
            let has_flags = !flags.is_empty();
            let interactive_input = !has_flags && io::stdin().is_terminal();

            let seconds = if interactive_input {
                prompt_u32("duration (seconds)", 10)?
            } else {
                flag_u32(&flags, "seconds", 10)?
            };
            let fps = if interactive_input {
                prompt_u32("target fps", 30)?
            } else {
                flag_u32(&flags, "fps", 30)?
            };
            let width = if interactive_input {
                prompt_u32("width", 1280)?
            } else {
                flag_u32(&flags, "width", 1280)?
            };
            let height = if interactive_input {
                prompt_u32("height", 720)?
            } else {
                flag_u32(&flags, "height", 720)?
            };

            Engine::realtime_with_resolution(width as usize, height as usize)
                .run_realtime(seconds, fps)?;
        }
        "render" => {
            let flags = parse_flags(args)?;
            #[cfg(target_os = "linux")]
            {
                let width = flag_u32(&flags, "width", 1280)?;
                let height = flag_u32(&flags, "height", 720)?;
                let samples = flag_u32(&flags, "spp", 64)?;
                let bounces = flag_u32(&flags, "bounces", 4)?;
                let exposure = flag_f32(&flags, "exposure", 0.9)?;
                let out = match flags.get("out") {
                    Some(p) => PathBuf::from(p),
                    None => {
                        let mut dir = std::env::temp_dir();
                        dir.push("enginerenderer_render");
                        std::fs::create_dir_all(&dir)?;
                        dir.join(format!("render_{width}x{height}.png"))
                    }
                };
                gpu_render_default(width, height, samples, bounces, exposure, &out)?;
            }
            #[cfg(not(target_os = "linux"))]
            {
                let report = Engine::default().run()?;
                if let Some(out) = flags.get("out") {
                    let dst = PathBuf::from(out);
                    if let Some(parent) = dst.parent()
                        && !parent.as_os_str().is_empty()
                    {
                        std::fs::create_dir_all(parent)?;
                    }
                    std::fs::copy(&report.output_path, &dst)?;
                    eprintln!(
                        "rendered {}x{} in {} ms -> {}",
                        report.width,
                        report.height,
                        report.duration_ms,
                        dst.display(),
                    );
                } else {
                    eprintln!(
                        "rendered {}x{} in {} ms -> {}",
                        report.width,
                        report.height,
                        report.duration_ms,
                        report.output_path.display(),
                    );
                }
            }
        }
        "validate" => {
            run_validate()?;
        }
        "gpu-render" => {
            run_gpu_render(args)?;
        }
        "detect" => {
            let options = parse_detect_options(args)?;
            let api = EngineApi::new();
            api.diagnose_compute_environment(&options);
        }
        other => {
            eprintln!("unknown command: {}", other);
            print_help();
        }
    }
    Ok(())
}

fn run_validate() -> Result<(), Box<dyn Error>> {
    let report = Engine::realtime_with_resolution(640, 360).run()?;
    eprintln!(
        "[validate] cpu smoke ok: {}x{} in {} ms -> {}",
        report.width,
        report.height,
        report.duration_ms,
        report.output_path.display(),
    );

    #[cfg(target_os = "linux")]
    {
        use enginerenderer::api::engine::rendering::gpu_try_new_desktop;
        match gpu_try_new_desktop(64, 64) {
            Ok((tracer, _ctx)) => {
                let dev = tracer.device();
                eprintln!(
                    "[validate] gpu context ok: vendor={:?} renderer={:?} version={:?}",
                    dev.vendor, dev.renderer, dev.version
                );
            }
            Err(e) => {
                eprintln!("[validate] gpu context failed: {}", e);
                return Err(format!("gpu probe failed: {}", e).into());
            }
        }
    }
    #[cfg(not(target_os = "linux"))]
    {
        eprintln!("[validate] gpu probe skipped (linux only)");
    }
    Ok(())
}

#[cfg(not(target_os = "linux"))]
fn run_gpu_render(_args: &[&str]) -> Result<(), Box<dyn Error>> {
    Err("gpu-render is only supported on Linux desktop (GLX 4.3+)".into())
}

#[cfg(target_os = "linux")]
fn run_gpu_render(args: &[&str]) -> Result<(), Box<dyn Error>> {
    use enginerenderer::api::engine::rendering::ProceduralEnvironment;

    let flags = parse_flags(args)?;

    let width = flag_u32(&flags, "width", 1280)?;
    let height = flag_u32(&flags, "height", 720)?;
    let samples = flag_u32(&flags, "spp", 64)?;
    let max_bounces = flag_u32(&flags, "bounces", 4)?;
    let seed = flag_u32_radix(&flags, "seed", 0xDECA_FC0D)?;
    let exposure = flag_f32(&flags, "exposure", 0.9)?;
    let _clouds = flag_f32(&flags, "clouds", 0.45)?.clamp(0.0, 1.0);

    let hdri = match flags
        .get("hdri")
        .map(|s| s.as_str())
        .unwrap_or("cinematic_space")
    {
        "none" | "off" => None,
        "cinematic_space" | "default" => Some(ProceduralEnvironment::cinematic_space()),
        other => return Err(format!("unknown hdri preset: {}", other).into()),
    };

    let out_path = match flags.get("out") {
        Some(p) => PathBuf::from(p),
        None => {
            let mut dir = std::env::temp_dir();
            dir.push("enginerenderer_gpu_demo");
            std::fs::create_dir_all(&dir)?;
            dir.join(format!("gpu_render_{width}x{height}.png"))
        }
    };

    gpu_render_into(
        width,
        height,
        samples,
        max_bounces,
        seed,
        exposure,
        hdri,
        &out_path,
    )
}

#[cfg(target_os = "linux")]
fn gpu_render_default(
    width: u32,
    height: u32,
    samples: u32,
    bounces: u32,
    exposure: f32,
    out_path: &std::path::Path,
) -> Result<(), Box<dyn Error>> {
    use enginerenderer::api::engine::rendering::ProceduralEnvironment;
    gpu_render_into(
        width,
        height,
        samples,
        bounces,
        0xDECA_FC0D,
        exposure,
        Some(ProceduralEnvironment::cinematic_space()),
        out_path,
    )
}

#[cfg(target_os = "linux")]
#[allow(clippy::too_many_arguments)]
fn gpu_render_into(
    width: u32,
    height: u32,
    samples: u32,
    max_bounces: u32,
    seed: u32,
    exposure: f32,
    hdri: Option<enginerenderer::api::engine::rendering::ProceduralEnvironment>,
    out_path: &std::path::Path,
) -> Result<(), Box<dyn Error>> {
    use enginerenderer::api::engine::rendering::{
        AreaLight, DirectionalLight, GpuRenderConfig, Image, Material, RenderCamera, Scene, Sphere,
        Vec3, VolumetricMedium, gpu_try_new_desktop,
    };
    use std::time::Instant;

    let workload = width as u64 * height as u64 * samples as u64;
    const MAX_WORKLOAD: u64 = 5_000_000_000;
    if workload > MAX_WORKLOAD {
        return Err(format!(
            "workload {} exceeds MAX_WORKLOAD {} (GPU TDR risk)",
            workload, MAX_WORKLOAD
        )
        .into());
    }

    eprintln!("==> creating offscreen GLX 4.3 context {width}x{height}");
    let (mut tracer, _ctx) = gpu_try_new_desktop(width, height)
        .map_err(|e| format!("could not create GPU context: {e}"))?;
    let dev = tracer.device();
    eprintln!(
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
        hdri,
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
        seed,
        exposure,
        denoise: false,
    };

    eprintln!(
        "==> dispatching compute: {width}x{height}, {samples} spp, {max_bounces} bounces, {} primary rays",
        workload
    );
    let start = Instant::now();
    let fb = tracer
        .render(&scene, &camera, cfg)
        .map_err(|e| format!("GPU render failed: {e}"))?;
    let elapsed = start.elapsed();
    let rays_per_sec = workload as f64 / elapsed.as_secs_f64();
    eprintln!(
        "==> render done in {:.3}s  ({:.2} M primary rays/s)",
        elapsed.as_secs_f64(),
        rays_per_sec / 1.0e6
    );

    if let Some(parent) = out_path.parent()
        && !parent.as_os_str().is_empty()
    {
        std::fs::create_dir_all(parent)?;
    }
    let image = Image {
        width: fb.width,
        height: fb.height,
        pixels: fb.color.clone(),
    };
    image.save(out_path)?;
    eprintln!("==> wrote {}", out_path.display());
    Ok(())
}

fn parse_flags(args: &[&str]) -> Result<HashMap<String, String>, Box<dyn Error>> {
    let mut out: HashMap<String, String> = HashMap::new();
    let mut i = 0usize;
    while i < args.len() {
        let arg = args[i];
        if let Some(rest) = arg.strip_prefix("--") {
            if let Some(eq) = rest.find('=') {
                out.insert(rest[..eq].to_string(), rest[eq + 1..].to_string());
                i += 1;
            } else {
                let key = rest.to_string();
                let Some(value) = args.get(i + 1) else {
                    return Err(format!("missing value for --{}", key).into());
                };
                out.insert(key, (*value).to_string());
                i += 2;
            }
        } else {
            return Err(format!("unexpected positional argument: {}", arg).into());
        }
    }
    Ok(out)
}

fn flag_u32(
    flags: &HashMap<String, String>,
    name: &str,
    default_value: u32,
) -> Result<u32, Box<dyn Error>> {
    match flags.get(name) {
        Some(v) => Ok(v.parse::<u32>()?.max(1)),
        None => Ok(default_value),
    }
}

fn flag_u32_radix(
    flags: &HashMap<String, String>,
    name: &str,
    default_value: u32,
) -> Result<u32, Box<dyn Error>> {
    match flags.get(name) {
        Some(v) => {
            let s = v.trim();
            let parsed = if let Some(hex) = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
                u32::from_str_radix(hex, 16)?
            } else {
                s.parse::<u32>()?
            };
            Ok(parsed)
        }
        None => Ok(default_value),
    }
}

fn flag_f32(
    flags: &HashMap<String, String>,
    name: &str,
    default_value: f32,
) -> Result<f32, Box<dyn Error>> {
    match flags.get(name) {
        Some(v) => Ok(v.parse::<f32>()?),
        None => Ok(default_value),
    }
}

fn prompt_u32(label: &str, default_value: u32) -> Result<u32, Box<dyn Error>> {
    eprint!("{} [{}]: ", label, default_value);
    io::stderr().flush()?;

    let mut line = String::new();
    io::stdin().read_line(&mut line)?;

    let trimmed = line.trim();
    if trimmed.is_empty() {
        return Ok(default_value);
    }

    let parsed = trimmed.parse::<u32>()?;
    Ok(parsed.max(1))
}

fn parse_detect_options(args: &[&str]) -> Result<DiagnosticsOptions, Box<dyn Error>> {
    let mut json = false;
    let mut verbose = false;
    let mut bench = false;
    let mut component: Option<DiagnosticComponent> = None;
    let mut overrides = DiagnosticOverrides::default();

    let mut i = 0usize;
    while i < args.len() {
        let arg = args[i];
        if arg == "--json" || arg == "-j" {
            json = true;
            i += 1;
            continue;
        }
        if arg == "--verbose" || arg == "-v" {
            verbose = true;
            i += 1;
            continue;
        }
        if arg == "--bench" {
            bench = true;
            i += 1;
            continue;
        }

        if let Some(value) = arg.strip_prefix("--component=") {
            component = parse_component(value)?;
            i += 1;
            continue;
        }
        if arg == "--component" {
            let Some(value) = args.get(i + 1) else {
                return Err("missing value for --component".into());
            };
            component = parse_component(value)?;
            i += 2;
            continue;
        }

        if let Some(value) = arg.strip_prefix("--override=") {
            parse_overrides(value, &mut overrides)?;
            i += 1;
            continue;
        }
        if arg == "--override" {
            let Some(value) = args.get(i + 1) else {
                return Err("missing value for --override".into());
            };
            parse_overrides(value, &mut overrides)?;
            i += 2;
            continue;
        }

        return Err(format!("unknown detect option: {}", arg).into());
    }

    Ok(DiagnosticsOptions {
        json,
        verbose,
        bench,
        component,
        overrides,
    })
}

fn parse_component(value: &str) -> Result<Option<DiagnosticComponent>, Box<dyn Error>> {
    if value.eq_ignore_ascii_case("all") {
        return Ok(None);
    }
    DiagnosticComponent::parse(value)
        .map(Some)
        .ok_or_else(|| format!("invalid component: {}", value).into())
}

fn parse_overrides(value: &str, overrides: &mut DiagnosticOverrides) -> Result<(), Box<dyn Error>> {
    for token in value.split(',').filter(|s| !s.trim().is_empty()) {
        let mut parts = token.splitn(2, '=');
        let Some(key) = parts.next().map(|k| k.trim().to_ascii_lowercase()) else {
            continue;
        };
        let Some(val) = parts.next().map(|v| v.trim()) else {
            return Err(format!("invalid override token: {}", token).into());
        };

        match key.as_str() {
            "arch" => {
                let parsed = ComputeArch::parse(val)
                    .ok_or_else(|| format!("invalid arch override: {}", val))?;
                overrides.arch = Some(parsed);
            }
            "os" => {
                let parsed =
                    ComputeOs::parse(val).ok_or_else(|| format!("invalid os override: {}", val))?;
                overrides.os = Some(parsed);
            }
            "vendor" => {
                let parsed = ComputeVendor::parse(val)
                    .ok_or_else(|| format!("invalid vendor override: {}", val))?;
                overrides.vendor = Some(parsed);
            }
            _ => {
                return Err(format!("invalid override key: {}", key).into());
            }
        }
    }
    Ok(())
}
