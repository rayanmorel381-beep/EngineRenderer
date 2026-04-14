use std::error::Error;
use std::io::{self, Write};

use super::animation;
use super::cli;
use super::docs;
use super::ui;

pub fn run_interactive() -> Result<(), Box<dyn Error>> {
    const MODULES: [&str; 8] = [
        "ai",
        "animation",
        "camera",
        "engine",
        "materials",
        "objects",
        "scenes",
        "types",
    ];

    eprintln!("Waiting for input");
    eprintln!("Type 'help' for further commands, 'quit' to EXIT");
    let stdin = io::stdin();
    let mut current_module: Option<&'static str> = None;

    loop {
        eprint!("engine> ");
        io::stderr().flush()?;

        let mut line = String::new();
        let read = stdin.read_line(&mut line)?;
        if read == 0 {
            eprintln!("exit (EOF)");
            break;
        }

        let cmd = line.trim();
        if cmd.is_empty() {
            continue;
        }

        if cmd.eq_ignore_ascii_case("quit") || cmd.eq_ignore_ascii_case("exit") || cmd.eq_ignore_ascii_case("qui") {
            eprintln!("bye");
            break;
        }

        let mut parts = cmd.split_whitespace();
        let op = parts.next().unwrap_or("");

        match op {
            "help" | "?" => {
                if let Some(module) = current_module {
                    print_module_help(module);
                } else {
                    print_root_help(&MODULES);
                }
            }
            "docs" => {
                let args: Vec<&str> = parts.collect();
                docs::print_docs(&args, current_module);
            }
            "animate" => {
                let topic = parts.next();
                animation::play_animation(topic, current_module)?;
            }
            "open" => {
                let Some(module) = parts.next() else {
                    eprintln!("usage: open <module>");
                    continue;
                };
                if let Some(found) = MODULES.iter().copied().find(|m| *m == module) {
                    current_module = Some(found);
                    eprintln!("opened module '{}'.", module);
                } else {
                    eprintln!("unknown module: {}", module);
                }
            }
            "call" => {
                let Some(function_name) = parts.next() else {
                    eprintln!("usage: call <fn>");
                    continue;
                };
                match current_module {
                    None => {
                        eprintln!("open a module first: open <module>");
                    }
                    Some("engine") => match function_name {
                        "render" => cli::run_command("render")?,
                        "gallery" => cli::run_command("gallery")?,
                        "test" => cli::run_command("test")?,
                        "run" => cli::run_command("run")?,
                        "detect" | "debug" => cli::run_command("detect")?,
                        other => eprintln!("unknown engine function: {}", other),
                    },
                    Some("animation") => match function_name {
                        "pipeline" | "engine" | "frames" | "bvh" | "scheduler" => {
                            animation::play_animation(Some(function_name), current_module)?;
                        }
                        other => eprintln!("unknown animation function: {}", other),
                    },
                    Some(module) => {
                        eprintln!("call is not mapped for module '{}'.", module);
                    }
                }
            }
            "inspect" => {
                let item = parts.next().unwrap_or("<item>");
                match current_module {
                    None => {
                        if MODULES.contains(&item) {
                            print_module_help(item);
                        } else {
                            eprintln!("root item '{}': use open {}", item, item);
                        }
                    }
                    Some(module) => {
                        eprintln!("module '{}' item '{}': no extended metadata yet", module, item);
                    }
                }
            }
            "back" | ".." => {
                current_module = None;
                eprintln!("back to root namespace");
            }
            "clear" => {
                eprint!("\x1B[2J\x1B[H");
                io::stderr().flush()?;
            }
            other => {
                eprintln!("unknown command: {}", other);
                eprintln!("type: help");
            }
        }
    }

    Ok(())
}

fn print_root_help(_modules: &[&str]) {
    let width = ui::terminal_columns().saturating_sub(2).clamp(60, 79);
    let namespace_lines = vec![
        "🚀Welcome in Helper🚀".to_string(),
        String::new(),
        "📦 Modules:".to_string(),
        "  🧠 ai         🎬 animation    📷 camera     ⚙ engine".to_string(),
        "  🎨 materials  📦 objects      🌌 scenes     🧩 types".to_string(),
    ];
    ui::print_framed_panel("Root Namespace", &namespace_lines, width);

    let command_lines = vec![
        "open <module>     📂 Explore a module".to_string(),
        "call <fn>         ⚡ Execute a function".to_string(),
        "inspect <item>    🔍 Show details".to_string(),
        "docs [target]     📚 Show code documentation".to_string(),
        "animate [topic]   🎞 Show execution flow".to_string(),
        "back              ⬅ Go up".to_string(),
        "clear             🧹 Clear screen".to_string(),
        "exit              🚪 Quit".to_string(),
    ];
    ui::print_framed_panel("Commands", &command_lines, width);
}

fn print_module_help(module: &str) {
    let module_header = vec![format!("Module: {}", module)];
    ui::print_console_block("Module", &module_header);

    let command_lines = vec![
        "call <fn>         Execute a function in this module".to_string(),
        "inspect <item>    Show details".to_string(),
        "docs [target] [section]  Show module docs from code".to_string(),
        "animate [topic]   Show module animation".to_string(),
        "back              Go up to root".to_string(),
        "clear             Clear screen".to_string(),
        "exit              Quit".to_string(),
    ];
    ui::print_console_block("Commands", &command_lines);

    match module {
        "engine" => {
            let callable = vec!["render | gallery | test | run | detect".to_string()];
            ui::print_console_block("Callable Functions", &callable);
        }
        "animation" => {
            let callable = vec!["pipeline | frames | bvh | scheduler".to_string()];
            ui::print_console_block("Callable Functions", &callable);
        }
        _ => {}
    }
}
