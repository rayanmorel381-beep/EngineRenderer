
//! EngineRenderer main binary: selects CLI, realtime, or video generation mode.

mod utils;
mod realtime;
mod generator;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() {
        return utils::terminal_mode::run_interactive();
    }

    if generator::is_video_mode(&args) {
        return generator::run(&args);
    }

    if realtime::is_realtime_mode(&args) {
        return realtime::run(&args);
    }

    utils::terminal_mode::run_cli(&args)
}
