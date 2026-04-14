use std::error::Error;
use std::io::{self, Write};
use std::thread;
use std::time::Duration;

pub fn play_animation(topic: Option<&str>, current_module: Option<&str>) -> Result<(), Box<dyn Error>> {
    let selected = topic.or(current_module).unwrap_or("pipeline");
    match selected {
        "pipeline" | "engine" => {
            run_animation(
                "engine pipeline",
                &[
                    "scene build      [#.....]",
                    "bvh build        [##....]",
                    "tile schedule    [###...]",
                    "ray trace        [####..]",
                    "post process     [#####.]",
                    "output/report    [######]",
                ],
                110,
            )?;
        }
        "animation" | "frames" => {
            run_animation(
                "animation workflow",
                &[
                    "descriptor        [#.....]",
                    "clip evaluate      [##....]",
                    "frame render       [###...]",
                    "frame export       [####..]",
                    "video encode       [#####.]",
                    "done               [######]",
                ],
                110,
            )?;
        }
        "bvh" => {
            run_animation(
                "bvh build",
                &[
                    "collect primitives [#.....]",
                    "sort centroids     [##....]",
                    "split nodes        [###...]",
                    "build leaves       [####..]",
                    "link hierarchy      [#####.]",
                    "ready              [######]",
                ],
                100,
            )?;
        }
        "scheduler" => {
            run_animation(
                "tile scheduler",
                &[
                    "tile partition     [#.....]",
                    "worker assign      [##....]",
                    "queue dispatch     [###...]",
                    "collect buffers    [####..]",
                    "merge image        [#####.]",
                    "ready              [######]",
                ],
                100,
            )?;
        }
        _ => {
            eprintln!("unknown animation topic: {}", selected);
            eprintln!("animation topics: pipeline | engine | animation | frames | bvh | scheduler");
        }
    }
    Ok(())
}

fn run_animation(name: &str, frames: &[&str], delay_ms: u64) -> Result<(), Box<dyn Error>> {
    eprintln!("animation: {}", name);
    for frame in frames {
        eprint!("\r{}", frame);
        io::stderr().flush()?;
        thread::sleep(Duration::from_millis(delay_ms));
    }
    eprintln!();
    eprintln!("animation complete");
    Ok(())
}
