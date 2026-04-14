mod utils;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    match std::env::args().nth(1) {
        None => utils::terminal_mode::run_interactive(),
        Some(arg) => utils::terminal_mode::run_cli(&arg),
    }
}
