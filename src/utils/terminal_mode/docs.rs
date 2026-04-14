use std::fs;
use std::path::{Path, PathBuf};

use super::ui;

pub fn print_docs(args: &[&str], current_module: Option<&str>) {
    let section_from_first = args
        .first()
        .copied()
        .filter(|value| is_doc_section(value));
    let target = if section_from_first.is_some() {
        current_module
    } else {
        args.first().copied().or(current_module)
    };
    let section = if section_from_first.is_some() {
        section_from_first
    } else {
        args
            .get(1)
            .copied()
            .filter(|value| is_doc_section(value))
    };

    if args.is_empty() && current_module.is_none() {
        print_docs_topics();
        return;
    }

    if let Some(raw_target) = target {
        let normalized_target = normalize_target(raw_target);
        match normalized_target.as_str() {
            "readme" => {
                print_doc_excerpt("README", include_str!("../../../ReadMe.md"), 220);
            }
            "tree" | "arborescence" => {
                print_doc_excerpt("ARBORESCENCE", include_str!("../../../arborescence.md"), 260);
            }
            "changelog" => {
                print_doc_excerpt("CHANGELOG", include_str!("../../../ChangeLog.md"), 220);
            }
            "runtime" => {
                print_runtime_docs(section);
            }
            "rendering" => {
                print_rendering_docs(section);
            }
            _ => {
                print_code_docs(&normalized_target, section);
            }
        }
    } else {
        print_docs_topics();
    }
}

pub(crate) fn print_docs_topics() {
    let topic_lines = vec![
        "readme | tree | changelog | runtime | rendering".to_string(),
        "ai | animation | camera | engine | materials | objects | scenes | types".to_string(),
        "ai.renderer | ai.prompt | ai.capabilities | ai.ai_manager".to_string(),
    ];
    ui::print_console_block("Documentation Topics", &topic_lines);

    let section_lines = vec!["overview | constraints | implementation | tips".to_string()];
    ui::print_console_block("Sections", &section_lines);

    let example_lines = vec![
        "docs ai".to_string(),
        "docs ai.renderer".to_string(),
        "docs ai.prompt constraints".to_string(),
        "docs implementation       (when a module is open)".to_string(),
    ];
    ui::print_console_block("Examples", &example_lines);
}

fn print_runtime_docs(section: Option<&str>) {
    match section.map(|s| s.to_ascii_lowercase()) {
        Some(s) if s == "constraints" => {
            eprintln!("runtime constraints:");
            eprintln!("  interactive mode starts only with no CLI input");
            eprintln!("  generation actions are explicit (call render/gallery/test)");
            eprintln!("  root namespace model is enforced by open/back flow");
        }
        Some(s) if s == "implementation" => {
            eprintln!("runtime implementation:");
            eprintln!("  file: src/main.rs");
            eprintln!("  parser: run_interactive() -> split_whitespace -> op dispatch");
            eprintln!("  state: current_module Option<&'static str>");
        }
        Some(s) if s == "tips" => {
            eprintln!("runtime tips:");
            eprintln!("  use open <module> then docs implementation");
            eprintln!("  use call test before call render for faster feedback");
            eprintln!("  use clear to keep logs readable");
        }
        _ => {
            eprintln!("runtime overview:");
            eprintln!("  entrypoint: src/main.rs");
            eprintln!("  flow: args -> run_interactive() -> command parser -> module context");
            eprintln!("  commands: help, open, call, inspect, docs, animate, back, clear, exit");
        }
    }
}

fn print_rendering_docs(section: Option<&str>) {
    match section.map(|s| s.to_ascii_lowercase()) {
        Some(s) if s == "constraints" => {
            eprintln!("rendering constraints:");
            eprintln!("  keep deterministic defaults for reproducible tests");
            eprintln!("  avoid implicit generation outside explicit calls");
        }
        Some(s) if s == "implementation" => {
            eprintln!("rendering implementation:");
            eprintln!("  path: src/core/engine/rendering");
            eprintln!("  core: renderer + raytracing + scheduler + post-process");
        }
        Some(s) if s == "tips" => {
            eprintln!("rendering tips:");
            eprintln!("  start with call test to iterate quickly");
            eprintln!("  profile bvh/scheduler before micro-optimizing shading");
        }
        _ => {
            eprintln!("rendering overview:");
            eprintln!("  layers: api -> coremanager -> rendering -> raytracing -> postprocess");
            eprintln!("  key dirs: src/core/engine/rendering, src/core/engine/rendering/raytracing");
        }
    }
}

fn print_code_docs(target: &str, section: Option<&str>) {
    let files = resolve_doc_files(target);
    if files.is_empty() {
        eprintln!("unknown docs topic: {}", target);
        eprintln!("type 'docs' to list topics");
        return;
    }

    eprintln!("docs for '{}':", target);
    for file in &files {
        let relative = file
            .strip_prefix(env!("CARGO_MANIFEST_DIR"))
            .ok()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| file.to_string_lossy().to_string());
        eprintln!("  source: {}", relative);
    }
    eprintln!();

    let mut doc_lines = Vec::new();
    let mut signature_lines = Vec::new();
    for file in files {
        if let Ok(source) = fs::read_to_string(&file) {
            doc_lines.extend(extract_doc_lines(&source));
            signature_lines.extend(extract_signatures(&source));
        }
    }

    print_doc_section(section, &doc_lines, &signature_lines);
}

fn resolve_doc_files(target: &str) -> Vec<PathBuf> {
    let mut files = Vec::new();
    let api_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/api");
    let parts: Vec<&str> = target.split('.').filter(|p| !p.is_empty()).collect();
    if parts.is_empty() {
        return files;
    }

    if parts.len() == 1 {
        let module_dir = api_root.join(parts[0]);
        let mod_file = module_dir.join("mod.rs");
        if mod_file.exists() {
            files.push(mod_file);
            if let Ok(entries) = fs::read_dir(&module_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().and_then(|e| e.to_str()) == Some("rs")
                        && path.file_name().and_then(|n| n.to_str()) != Some("mod.rs")
                    {
                        files.push(path);
                    }
                }
            }
        } else {
            let file = api_root.join(format!("{}.rs", parts[0]));
            if file.exists() {
                files.push(file);
            }
        }
        return files;
    }

    let module_dir = api_root.join(parts[0]);
    if !module_dir.exists() {
        return files;
    }
    let sub = parts[1..].join("/");
    let sub_file = module_dir.join(format!("{}.rs", sub));
    if sub_file.exists() {
        files.push(sub_file);
        return files;
    }
    let nested_mod = module_dir.join(sub).join("mod.rs");
    if nested_mod.exists() {
        files.push(nested_mod);
    }
    files
}

fn extract_doc_lines(source: &str) -> Vec<String> {
    source
        .lines()
        .filter_map(|line| line.trim_start().strip_prefix("///"))
        .map(|line| line.trim().to_string())
        .collect()
}

fn extract_signatures(source: &str) -> Vec<String> {
    source
        .lines()
        .map(str::trim)
        .filter(|line| {
            line.starts_with("pub fn ")
                || line.starts_with("pub struct ")
                || line.starts_with("pub enum ")
                || line.starts_with("impl ")
                || line.starts_with("pub mod ")
        })
        .map(|line| line.to_string())
        .collect()
}

fn print_doc_section(section: Option<&str>, doc_lines: &[String], signatures: &[String]) {
    let section_name = section.unwrap_or("overview").to_ascii_lowercase();
    match section_name.as_str() {
        "implementation" => {
            eprintln!("implementation:");
            if signatures.is_empty() {
                eprintln!("  no public signatures detected");
            } else {
                for line in signatures.iter().take(80) {
                    eprintln!("  {}", line);
                }
            }
        }
        "constraints" => {
            eprintln!("constraints:");
            let keywords = [
                "must",
                "only",
                "avoid",
                "unknown",
                "ignored",
                "stable",
                "deterministic",
                "fallback",
                "no ",
            ];
            let mut count = 0usize;
            for line in doc_lines {
                let lower = line.to_ascii_lowercase();
                if keywords.iter().any(|k| lower.contains(k)) {
                    eprintln!("  {}", line);
                    count += 1;
                    if count >= 60 {
                        break;
                    }
                }
            }
            if count == 0 {
                eprintln!("  no explicit constraints found in rust doc comments");
            }
        }
        "tips" => {
            eprintln!("tips:");
            let keywords = ["tip", "quick start", "workflow", "example", "fallback"];
            let mut count = 0usize;
            for line in doc_lines {
                let lower = line.to_ascii_lowercase();
                if keywords.iter().any(|k| lower.contains(k)) {
                    eprintln!("  {}", line);
                    count += 1;
                    if count >= 60 {
                        break;
                    }
                }
            }
            if count == 0 {
                eprintln!("  no explicit tips found in rust doc comments");
            }
        }
        _ => {
            eprintln!("overview:");
            if doc_lines.is_empty() {
                eprintln!("  no rust doc comments found in selected source files");
            } else {
                for line in doc_lines.iter().take(80) {
                    eprintln!("  {}", line);
                }
            }
        }
    }
}

fn print_doc_excerpt(title: &str, content: &str, max_lines: usize) {
    eprintln!("{}", title);
    eprintln!();
    for line in content.lines().take(max_lines) {
        eprintln!("{}", line);
    }
    let total = content.lines().count();
    if total > max_lines {
        eprintln!();
        eprintln!("... truncated: showing {}/{} lines", max_lines, total);
    }
}

fn is_doc_section(value: &str) -> bool {
    matches!(
        value.to_ascii_lowercase().as_str(),
        "overview" | "constraints" | "implementation" | "tips"
    )
}

fn normalize_target(target: &str) -> String {
    target
        .trim()
        .trim_start_matches("api.")
        .trim_start_matches("src/api/")
        .replace('/', ".")
        .to_ascii_lowercase()
}
