use std::error::Error;

use enginerenderer::api::engine::Engine;

pub fn run_cli(arg: &str) -> Result<(), Box<dyn Error>> {
    match arg {
        "gallery" | "--gallery" => run_command("gallery")?,
        "test" | "--test" => run_command("test")?,
        "run" | "--run" => run_command("run")?,
        "render" | "--render" => run_command("render")?,
        "detect" | "--detect" | "debug" | "--debug" => run_command("detect")?,
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
    eprintln!("  cargo run -- render       -> standard render (writes a file)");
    eprintln!("  cargo run -- gallery      -> gallery_*.ppm showcase set");
    eprintln!("  cargo run -- test         -> tiny smoke render (direct)");
    eprintln!("  cargo run -- detect       -> print hardware/compute detection diagnostics");
    eprintln!("Interactive commands:");
    eprintln!("  help | open <module> | call <fn> | inspect <item> | docs [target] [section] | animate [topic] | back | clear | exit");
}

pub(crate) fn run_command(cmd: &str) -> Result<(), Box<dyn Error>> {
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
            eprintln!("Already in run");
            eprintln!("Waiting for input");
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
            enginerenderer::api::diagnose_compute_environment();
        }
        other => {
            eprintln!("unknown command: {}", other);
            print_help();
        }
    }
    Ok(())
}
