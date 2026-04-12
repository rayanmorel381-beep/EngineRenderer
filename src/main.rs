use std::error::Error;

use enginerenderer::api::engine::Engine;

fn main() -> Result<(), Box<dyn Error>> {
    match std::env::args().nth(1).as_deref() {
        Some("gallery") | Some("--gallery") => {
            let reports = Engine::production_reference().render_gallery()?;
            eprintln!("gallery renders:");
            for report in reports {
                eprintln!("  {}", report.output_path.display());
            }
        }
        Some("test") | Some("--test") => {
            let report = Engine::test_minimal().run()?;
            eprintln!("test render -> {}", report.output_path.display());
        }
        Some("help") | Some("--help") | Some("-h") => {
            eprintln!("Usage:");
            eprintln!("  cargo run                 -> output/render_output.ppm");
            eprintln!("  cargo run -- gallery      -> gallery_*.ppm showcase set");
            eprintln!("  cargo run -- test         -> tiny smoke render");
            eprintln!("Examples:");
            eprintln!("  cargo run --example render_spheres    -> output/SPHERES/spheres.ppm");
            eprintln!("  cargo run --example render_cubes      -> output/CUBES/cubes.ppm");
            eprintln!("  cargo run --example render_house      -> output/HOUSES/house.ppm");
            eprintln!("  cargo run --example render_city       -> output/CITY/city.ppm");
            eprintln!("  cargo run --example render_car        -> output/CAR/car.ppm");
            eprintln!("  cargo run --example render_world      -> output/WORLD/world.ppm");
            eprintln!("  cargo run --example render_blackhole  -> output/BLACKHOLE/blackhole.ppm");
        }
        _ => {
            let report = Engine::default().run()?;
            eprintln!(
                "rendered {}x{} in {} ms -> {}",
                report.width,
                report.height,
                report.duration_ms,
                report.output_path.display(),
            );
        }
    }

    Ok(())
}
