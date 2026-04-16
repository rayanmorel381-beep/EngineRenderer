//! Terminal mode entry points and submodules.

mod animation;
mod cli;
mod docs;
mod interactive;
mod ui;

pub use cli::run_cli;
/// Interactive REPL entry point used when no CLI argument is provided.
pub use interactive::run_interactive;
