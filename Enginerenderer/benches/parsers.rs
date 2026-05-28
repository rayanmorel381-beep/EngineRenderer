//! Zero-dependency benchmark harness for parser hot paths.
//!
//! Run with: `cargo bench --bench parsers`
//!
//! Each benchmark is a closure executed `iters` times after a warmup of
//! `warmup` iterations. We report the median, mean, min and max wall-clock
//! time per iteration, plus the throughput in items/s when applicable.

use std::hint::black_box;
use std::time::{Duration, Instant};

use enginerenderer::api::loaders::glb::iter_glb_chunks;
use enginerenderer::api::loaders::obj::ObjLoader;
use enginerenderer::api::scenes::{SceneDescriptor, SphereEntry};

const WARMUP_ITERS: u32 = 16;
const MEASURE_ITERS: u32 = 64;

struct Sample {
    name: &'static str,
    items: u64,
    timings_ns: Vec<u128>,
}

impl Sample {
    fn report(&self) {
        let mut sorted = self.timings_ns.clone();
        sorted.sort_unstable();
        let n = sorted.len();
        let min = sorted[0];
        let max = sorted[n - 1];
        let median = sorted[n / 2];
        let sum: u128 = sorted.iter().sum();
        let mean = sum / n as u128;
        let throughput = if median > 0 {
            (self.items as f64) * 1_000_000_000.0 / median as f64
        } else {
            f64::INFINITY
        };
        println!(
            "{:<48} median={:>10}ns  mean={:>10}ns  min={:>10}ns  max={:>10}ns  ~{:>10.0} items/s",
            self.name, median, mean, min, max, throughput
        );
    }
}

fn bench<F: FnMut()>(name: &'static str, items: u64, mut body: F) -> Sample {
    for _ in 0..WARMUP_ITERS {
        body();
    }
    let mut timings = Vec::with_capacity(MEASURE_ITERS as usize);
    for _ in 0..MEASURE_ITERS {
        let start = Instant::now();
        body();
        timings.push(start.elapsed().as_nanos());
    }
    Sample {
        name,
        items,
        timings_ns: timings,
    }
}

fn make_obj_source(triangles: usize) -> String {
    let mut s = String::with_capacity(triangles * 80);
    for i in 0..triangles {
        let x = i as f64 * 0.001;
        s.push_str(&format!("v {x} 0.0 0.0\n"));
        s.push_str(&format!("v {x} 1.0 0.0\n"));
        s.push_str(&format!("v {x} 0.0 1.0\n"));
    }
    for i in 0..triangles {
        let base = i * 3 + 1;
        s.push_str(&format!("f {} {} {}\n", base, base + 1, base + 2));
    }
    s
}

fn make_scene_source(spheres: usize) -> String {
    let mut s = String::with_capacity(spheres * 80);
    s.push_str("camera position=0,0,5 target=0,0,0 fov=60 aperture=0.0\n");
    s.push_str("sun direction=-0.3,-1.0,-0.2 color=1.0,1.0,1.0 intensity=1.5\n");
    s.push_str("sky top=0.5,0.7,1.0 bottom=0.05,0.1,0.2\n");
    for i in 0..spheres {
        let x = (i as f64) * 0.5;
        s.push_str(&format!(
            "sphere center={x},0.0,0.0 radius=0.25 albedo=0.8,0.4,0.2 roughness=0.5 metallic=0.0\n"
        ));
    }
    s
}

fn make_glb_source(chunks: usize) -> Vec<u8> {
    let mut tail = Vec::new();
    let payload = vec![0x42u8; 32];
    for _ in 0..chunks {
        tail.extend_from_slice(&(payload.len() as u32).to_le_bytes());
        tail.extend_from_slice(&0x004E_4942u32.to_le_bytes());
        tail.extend_from_slice(&payload);
    }
    let total = 12 + tail.len();
    let mut buf = Vec::with_capacity(total);
    buf.extend_from_slice(b"glTF");
    buf.extend_from_slice(&2u32.to_le_bytes());
    buf.extend_from_slice(&(total as u32).to_le_bytes());
    buf.extend(tail);
    buf
}

fn bench_obj_parsing() -> Sample {
    let source = make_obj_source(1000);
    let triangles = 1000u64;
    bench("obj_parse_1000_triangles", triangles, || {
        let mesh = ObjLoader::parse_str(black_box(&source), "bench".to_string()).expect("parse ok");
        black_box(mesh);
    })
}

fn bench_obj_parsing_heavy() -> Sample {
    let source = make_obj_source(50_000);
    let triangles = 50_000u64;
    bench("obj_parse_50k_triangles", triangles, || {
        let mesh = ObjLoader::parse_str(black_box(&source), "bench".to_string()).expect("parse ok");
        black_box(mesh);
    })
}

fn bench_scene_parsing() -> Sample {
    let source = make_scene_source(500);
    let spheres = 500u64;
    bench("scene_parse_500_spheres", spheres, || {
        let desc = SceneDescriptor::parse(black_box(&source)).expect("parse ok");
        black_box(desc);
    })
}

fn bench_scene_serialize() -> Sample {
    let mut desc = SceneDescriptor::default();
    for i in 0..500 {
        desc.spheres.push(SphereEntry {
            position: [i as f64 * 0.1, 0.0, 0.0],
            radius: 0.25,
            material_name: None,
            albedo: [0.8, 0.4, 0.2],
            roughness: 0.5,
            metallic: 0.0,
            emission: 0.0,
        });
    }
    let spheres = 500u64;
    bench("scene_serialize_500_spheres", spheres, || {
        let s = black_box(&desc).serialize();
        black_box(s);
    })
}

fn bench_scene_serialize_heavy() -> Sample {
    let mut desc = SceneDescriptor::default();
    for i in 0..5_000 {
        desc.spheres.push(SphereEntry {
            position: [i as f64 * 0.1, 0.0, 0.0],
            radius: 0.25,
            material_name: None,
            albedo: [0.8, 0.4, 0.2],
            roughness: 0.5,
            metallic: 0.0,
            emission: 0.0,
        });
    }
    let spheres = 5_000u64;
    bench("scene_serialize_5k_spheres", spheres, || {
        let s = black_box(&desc).serialize();
        black_box(s);
    })
}

fn bench_glb_validation() -> Sample {
    let buf = make_glb_source(100);
    let chunks = 100u64;
    bench("glb_iter_chunks_100_payloads", chunks, || {
        let chunks = iter_glb_chunks(black_box(&buf)).expect("iter ok");
        black_box(chunks);
    })
}

fn bench_scene_roundtrip() -> Sample {
    let source = make_scene_source(500);
    let spheres = 500u64;
    bench("scene_parse_then_serialize_500", spheres, || {
        let desc = SceneDescriptor::parse(black_box(&source)).expect("parse ok");
        let s = desc.serialize();
        black_box(s);
    })
}

fn main() {
    println!("== enginerenderer micro-benchmarks ==");
    println!(
        "warmup={WARMUP_ITERS}  measure={MEASURE_ITERS}  total per bench wall-clock budget = ~{}ms",
        MEASURE_ITERS * 10
    );
    println!();
    let benches: Vec<Sample> = vec![
        bench_obj_parsing(),
        bench_obj_parsing_heavy(),
        bench_scene_parsing(),
        bench_scene_serialize(),
        bench_scene_serialize_heavy(),
        bench_glb_validation(),
        bench_scene_roundtrip(),
    ];
    for s in &benches {
        s.report();
    }
    let total_ns: u128 = benches
        .iter()
        .map(|s| s.timings_ns.iter().sum::<u128>())
        .sum();
    println!();
    println!(
        "total measurement wall-clock: {:.2}s",
        Duration::from_nanos(total_ns as u64).as_secs_f64()
    );
}
