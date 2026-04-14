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

## 3. Interactive terminal mode

```bash
cargo run --release
```

Expected result: the REPL opens with module navigation.

## 4. Render example scene

```bash
cargo run --example render_spheres
```

Expected result: output/SPHERES/spheres.ppm

## Optional cross-target builds

```bash
cargo build --release --target aarch64-apple-darwin
cargo build --release --target aarch64-unknown-linux-gnu
cargo build --release --target x86_64-pc-windows-gnu
cargo build --release --target aarch64-linux-android
cargo build --release --target x86_64-unknown-linux-musl
```
