use enginerenderer::api::engine::Engine;

pub fn is_realtime_mode(args: &[String]) -> bool {
    args.iter().any(|arg| {
        arg == "run"
            || arg == "--run"
            || arg.starts_with("--seconds")
            || arg.starts_with("--fps")
            || arg.starts_with("--width")
            || arg.starts_with("--height")
    })
}

pub fn run(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let seconds = parse_u32_arg(args, "--seconds", 10).max(1);
    let fps = parse_u32_arg(args, "--fps", 30).max(1);
    let width = parse_optional_u32_arg(args, "--width");
    let height = parse_optional_u32_arg(args, "--height");

    match (width, height) {
        (Some(w), Some(h)) => {
            Engine::realtime_with_resolution(w.max(1) as usize, h.max(1) as usize)
                .run_realtime(seconds, fps)
        }
        _ => Engine::realtime().run_realtime(seconds, fps),
    }
}

fn parse_u32_arg(args: &[String], key: &str, default_value: u32) -> u32 {
    if let Some(value) = parse_string_arg(args, key)
        && let Ok(parsed) = value.parse::<u32>()
    {
        return parsed;
    }
    default_value
}

fn parse_string_arg(args: &[String], key: &str) -> Option<String> {
    for (index, arg) in args.iter().enumerate() {
        if let Some(value) = arg.strip_prefix(&(String::from(key) + "=")) {
            return Some(value.to_string());
        }
        if arg == key {
            return args.get(index + 1).cloned();
        }
    }
    None
}

fn parse_optional_u32_arg(args: &[String], key: &str) -> Option<u32> {
    let value = parse_string_arg(args, key)?;
    value.parse::<u32>().ok()
}
