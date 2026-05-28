<div align="center">

# ⚡ Quickstart

Fastest way to build and run EngineRenderer.

</div>

## 1. Build

```bash
cargo build --release
```

## 2. First render

```bash
cargo run --release -- render
```

Expected result: an image is generated in the output directory.

## 3. Mode vidéo (animation → MP4)

```bash
cargo run --release -- video --duration=10 --fps=30 --width=1920 --height=1080 --output-mp4=output/animation.mp4
```

Expected result: `output/animation.mp4` généré via FFmpeg.

## 4. Mode realtime

```bash
cargo run --release -- run --seconds=10 --fps=30
```

Expected result: rendu séquentiel en mode preview.

## 5. Render example scene

```bash
cargo run --example render_spheres     # → output/SPHERES/spheres.ppm
cargo run --example render_cubes       # → output/CUBES/cubes.ppm
cargo run --example render_blackhole   # → output/BLACKHOLE/blackhole.ppm
```

Expected result: output files generated in their respective directories.

## 6. Interactive terminal mode

```bash
cargo run --release
```

Expected result: the REPL opens with module navigation.

## 7. Optional cross-target builds

```bash
cargo build --release --target aarch64-apple-darwin
cargo build --release --target aarch64-unknown-linux-gnu
cargo build --release --target x86_64-pc-windows-gnu
cargo build --release --target aarch64-linux-android
cargo build --release --target x86_64-unknown-linux-musl
```

## 8. Workspace validation

```bash
cargo check --all-targets
cargo clippy --all-targets
cargo test --test workspace_render_validation
```

Long campaigns (ignored by default):

```bash
cargo test --test workspace_render_validation -- --ignored
```
