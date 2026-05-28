//! EngineRenderer main binary: selects CLI, realtime, or video generation mode.

mod generator;
mod realtime;
mod utils;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() {
        return utils::terminal_mode::run_interactive();
    }

    let first = args[0].as_str();
    let cli_commands = [
        "help",
        "--help",
        "-h",
        "version",
        "--version",
        "-V",
        "render",
        "--render",
        "gallery",
        "--gallery",
        "test",
        "--test",
        "validate",
        "--validate",
        "gpu-render",
        "--gpu-render",
        "detect",
        "--detect",
        "debug",
        "--debug",
    ];
    if cli_commands.contains(&first) {
        return utils::terminal_mode::run_cli(&args);
    }

    if generator::is_video_mode(&args) {
        return generator::run(&args);
    }

    if realtime::is_realtime_mode(&args) {
        return realtime::run(&args);
    }

    utils::terminal_mode::run_cli(&args)
}
