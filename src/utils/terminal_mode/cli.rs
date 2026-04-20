
use std::error::Error;
use std::io::{self, Write};

use enginerenderer::api::engine::Engine;
use enginerenderer::api::engine::EngineApi;
use enginerenderer::api::engine::diagnostics::{
    ComputeArch,
    ComputeOs,
    ComputeVendor,
    DiagnosticComponent,
    DiagnosticOverrides,
    DiagnosticsOptions,
};

pub fn run_cli(args: &[String]) -> Result<(), Box<dyn Error>> {
    let Some(first) = args.first().map(|s| s.as_str()) else {
        print_help();
        return Ok(());
    };

    match first {
        "gallery" | "--gallery" => run_command_with_args("gallery", &[])? ,
        "test" | "--test" => run_command_with_args("test", &[])? ,
        "run" | "--run" => run_command_with_args("run", &[])? ,
        "render" | "--render" => run_command_with_args("render", &[])? ,
        "detect" | "--detect" | "debug" | "--debug" => {
            let tail: Vec<&str> = args.iter().skip(1).map(|s| s.as_str()).collect();
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
    eprintln!("Usage:");
    eprintln!("  cargo run -- help         -> show this help");
    eprintln!("  cargo run -- run          -> real-time mode with input prompts");
    eprintln!("  cargo run -- render       -> standard render (writes a file)");
    eprintln!("  cargo run -- gallery      -> gallery_*.ppm showcase set");
    eprintln!("  cargo run -- test         -> tiny smoke render (direct)");
    eprintln!("  cargo run -- detect [--json] [--verbose] [--bench] [--component cpu|gpu|ram|display] [--override arch=x86,os=linux,vendor=amd]");
    eprintln!("Interactive commands:");
    eprintln!("  help | open <module> | call <fn> | inspect <item> | docs [target] [section] | animate [topic] | back | clear | exit");
}

pub(crate) fn run_command(cmd: &str) -> Result<(), Box<dyn Error>> {
    run_command_with_args(cmd, &[])
}

pub(crate) fn run_command_with_args(cmd: &str, args: &[&str]) -> Result<(), Box<dyn Error>> {
    match cmd {
        "help" => print_help(),
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
            let seconds = prompt_u32("duration (seconds)", 10)?;
            let fps = prompt_u32("target fps", 30)?;
            let width = prompt_u32("width", 1280)?;
            let height = prompt_u32("height", 720)?;

            Engine::realtime_with_resolution(width as usize, height as usize)
                .run_realtime(seconds, fps)?;
        }
        "render" => {
            let report = Engine::default().run()?;
            eprintln!(
                "rendered {}x{} in {} ms -> {}",
                report.width,
                report.height,
                report.duration_ms,
                report.output_path.display(),
            );
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
                let parsed = ComputeOs::parse(val)
                    .ok_or_else(|| format!("invalid os override: {}", val))?;
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
