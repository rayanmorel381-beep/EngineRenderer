<div align="center">

# 🎥 EngineRenderer 🎥

### A zero-dependency offline rendering engine in pure Rust

<br>

`CPU Path Tracing` · `BVH Acceleration` · `16-Band Spectral Rendering` · `PBR Materials` · `Animation & Video`

<br>

[![Rust](https://img.shields.io/badge/Rust-edition_2024-f74c00?style=for-the-badge&logo=rust&logoColor=white)](#-building)
[![Platforms](https://img.shields.io/badge/Linux_·_macOS_·_Windows_·_Android-0078d4?style=for-the-badge&logo=linux&logoColor=white)](#-cross-platform-support)
[![Build Matrix](https://img.shields.io/badge/All_listed_architectures-compiling-success?style=for-the-badge&logo=rust&logoColor=white&color=2ea44f)](#-cross-platform-support)
[![Crates](https://img.shields.io/badge/external_crates-0-2ea44f?style=for-the-badge)](#-stats)
[![License](https://img.shields.io/badge/MIT-license-yellow?style=for-the-badge)](License)

<br>

**244 source files** · **~26,350 lines** · **17 material presets** · **18 interpolation curves** · **9+ targets**

</div>

<br>

## 🎨 Features

<table>
<tr><td>

### Rendering Pipeline

| | |
|:--|:--|
| 🔦 **Path Tracing** | Configurable SPP (1–64+), multi-bounce, BVH-accelerated |
| 🌈 **Spectral** | 16-band wavelength dispatch, CIE 1931 XYZ conversion |
| 🪩 **PBR Materials** | Albedo · roughness · metallic · emission · IOR · subsurface · anisotropy · iridescence · clearcoat · transmission |
| 🎭 **17 Presets** | Stellar · planetary · manufactured · celestial · natural |
| 💡 **Lighting** | Area lights · soft shadows · cascaded shadow maps (PCF + contact) |
| 🌫️ **Volumetrics** | God rays · participating media · atmospheric scattering · clouds |
| ✨ **Post-Processing** | Bloom · blur · depth-of-field (physical lens) · tone mapping |
| 🏎️ **Motion Blur** | Time-sampled primitives |
| 🎯 **Optimization** | Adaptive sampling · firefly rejection · denoising · frustum culling · LOD |

</td></tr>
</table>

### Render Presets

```
AnimationFast ·········  1 SPP · 1 bounce · denoised previews
PreviewCpu ············  4 SPP · 1 bounce · adaptive sampling
UltraHdCpu ············ 16 SPP · 3 bounces · high-quality stills
ProductionReference ··· 64 SPP · max bounces · reference quality
```

### Realtime 120 FPS Profile

EngineRenderer includes a unified ultra-constrained realtime profile for `--fps 120`.

- Scene-side budget: proxy showcase geometry, reduced light budget, optional panorama removal
- Runtime-side budget: cached showcase meshes, static realtime scene reuse, BVH reuse
- Loop-side budget: deadline-based frame pacing with drift compensation

Validated runtime benchmarks (release, 3 seconds, `--fps 120 --width 1280 --height 720`):

- Linux x86_64: achieved FPS above target 120
- Android ARM64: achieved FPS above target 120

### 🎬 Animation

| | |
|:--|:--|
| ⏱️ Timeline keyframes | Camera · sun · sky · exposure |
| 📐 18 interpolations | Linear · Step · SmoothStep · Hermite · EaseIn/Out/InOut (Quad, Cubic, Sine, Expo) · Back · Bounce |
| 🎞️ Frame sequencer | Batch rendering with caching |
| 📹 Video export | MP4 via system FFmpeg |

### 🏗️ Scene Building

| | |
|:--|:--|
| 🔗 Fluent API | `SceneBuilder` · `MaterialBuilder` · `CameraController` |
| 📄 Scene format | Text-based `.scene` files with serialization & parsing |
| 📦 Mesh import | OBJ · glTF/GLB |
| 🔧 Procedural | Geometry generators · mesh operations · composite objects |

### ⚛️ Physics & Simulation

| | |
|:--|:--|
| 💥 Rigid bodies | Constraint solving · collision detection · raycasting |
| 🪐 N-body | Gravitational simulation for celestial mechanics |

### 🤖 AI Integration

| | |
|:--|:--|
| 🔍 Introspection | `Capabilities::discover()` → JSON feature description |
| 💬 Prompt-to-scene | Natural language → renderable scene |
| 📷 Camera presets | Front · top-down · dramatic · cinematic · closeup |

<br>

## 🏛️ Architecture

```
╔══════════════════════════════════════════════════════════════════╗
║                          API Layer                              ║
║   SceneBuilder · MaterialCatalog · CameraController             ║
║   AnimationClip · Timeline · FrameSequencer                     ║
║   AiRenderer · Capabilities · SceneDescriptor                   ║
╠══════════════════════════════════════════════════════════════════╣
║                         Core Engine                             ║
║   Path Tracer · BVH · Spectral Shading · Shadow Maps            ║
║   Volumetrics · Post-Processing · Scene Graph · Physics         ║
║   Animation Sequencer · Video Exporter · Scheduler              ║
╠══════════════════════════════════════════════════════════════════╣
║                  Hardware Abstraction Layer                      ║
║   CPU (topology, SIMD) · GPU (DRM/D3D12) · Display · DMA       ║
╚══════════════════════════════════════════════════════════════════╝
```

📋 Full module breakdown: [arborescence.md](arborescence.md)

<br>

## 🌍 Cross-Platform Support

<table>
<tr>
<td width="50%">

### Targets

| Target | Arch | OS |
|:-------|:-----|:---|
| `x86_64-unknown-linux-gnu` | x86_64 | Linux |
| `i686-unknown-linux-gnu` | x86 | Linux |
| `aarch64-unknown-linux-gnu` | ARM64 | Linux |
| `armv7-unknown-linux-gnueabihf` | ARMv7 | Linux |
| `x86_64-apple-darwin` | x86_64 | macOS Intel |
| `aarch64-apple-darwin` | ARM64 | macOS Silicon |
| `x86_64-pc-windows-gnu` | x86_64 | Windows |
| `i686-pc-windows-gnu` | x86 | Windows |
| `aarch64-linux-android` | ARM64 | Android |

> + musl variants for static linking

✅ All listed targets compile successfully.

✅ `cargo clippy --target x86_64-pc-windows-gnu` passes cleanly.

</td>
<td width="50%">

### SIMD Detection

| ISA | Extensions |
|:----|:-----------|
| x86 / x86_64 | SSE4.2 · AVX · AVX-512 |
| ARM / AArch64 | NEON · SVE |
| *Fallback* | Scalar paths |

### GPU Drivers

| OS | Drivers |
|:---|:--------|
| Linux | `amdgpu` `radeon` `i915` `xe` `nouveau` `mali` `msm` |
| Windows | D3D12 / DXGI |
| macOS | AppleGPU · Intel iGPU |

> Automatic CPU fallback if no GPU available

</td>
</tr>
</table>

<br>

## 🔨 Building

| Dependency | Required | Notes |
|:-----------|:--------:|:------|
| **Rust** | ✅ | Edition 2024 |
| **FFmpeg** | ❌ | Only for MP4 export |

```bash
cargo build --release
```

Quick start: [quickstat.md](quickstat.md)

<details>
<summary>🔀 <strong>Cross-compilation targets</strong></summary>

```bash
cargo build --release --target aarch64-apple-darwin         # Apple Silicon
cargo build --release --target aarch64-unknown-linux-gnu    # ARM64 Linux
cargo build --release --target x86_64-pc-windows-gnu        # Windows (MinGW)
cargo build --release --target aarch64-linux-android         # Android
cargo build --release --target x86_64-unknown-linux-musl    # Static Linux
```

</details>

<br>

## 🚀 Usage

### CLI Mode

```bash
cargo run --release -- render    # Standard render → output file
cargo run --release -- gallery   # Full gallery showcase
cargo run --release -- test      # Quick smoke test
cargo run --release -- detect    # Hardware/compute diagnostics
cargo run --release -- video     # Render animation → MP4
cargo run --release -- run       # Realtime preview mode
cargo run --release -- help      # Show commands
```

#### Detect / Debug options

```bash
cargo run --release -- detect --verbose
cargo run --release -- detect --json --component gpu
cargo run --release -- detect --bench --component cpu
cargo run --release -- detect --override arch=arm,os=windows,vendor=amd
```

Supported values:

- `--component`: `cpu`, `gpu`, `ram`, `display`, `all`
- `--override`: `arch=x86|arm`, `os=linux|windows|macos`, `vendor=amd|intel|apple|unknown`

### Interactive Terminal

Launch with **no arguments** for a full REPL with module navigation and built-in docs:

```bash
cargo run --release
```

```
╭───────── Root Namespace ─────────╮
│ 🚀Welcome in Helper🚀           │
│                                  │
│ 📦 Modules:                     │
│   🧠 ai         🎬 animation   │
│   📷 camera     ⚙ engine       │
│   🎨 materials  📦 objects     │
│   🌌 scenes     🧩 types       │
╰──────────────────────────────────╯
```

<details>
<summary>📖 <strong>All REPL commands</strong></summary>
<br>

| Command | Description |
|:--------|:------------|
| `open <module>` | Enter a module namespace |
| `call <fn>` | Execute a function |
| `inspect <item>` | Show details |
| `docs [target] [section]` | Browse source documentation |
| `animate [topic]` | Show execution flow |
| `back` | Return to root |
| `clear` | Clear screen |
| `exit` | Quit |

**engine** → `call render` · `call gallery` · `call test` · `call run`

**animation** → `call spheres` · `call cubes` · `call house` · `call city` · `call car` · `call world` · `call blackhole`

</details>

<details>
<summary>📚 <strong>Built-in documentation browser</strong></summary>
<br>

The REPL reads source code directly:

```
engine> docs                        # List all topics
engine> docs readme                 # README
engine> docs tree                   # Project structure
engine> docs changelog              # Changelog
engine> docs rendering              # Rendering pipeline
engine> docs ai.renderer            # Sub-module docs
engine> docs ai.prompt constraints  # Section filter
```

Sections: `overview` · `constraints` · `implementation` · `tips`

</details>

<br>

### Render Examples

```bash
cargo run --example render_spheres     # → output/SPHERES/spheres.ppm
cargo run --example render_cubes       # → output/CUBES/cubes.ppm
cargo run --example render_house       # → output/HOUSES/house.ppm
cargo run --example render_city        # → output/CITY/city.ppm
cargo run --example render_car         # → output/CAR/car.ppm
cargo run --example render_world       # → output/WORLD/world.ppm
cargo run --example render_blackhole   # → output/BLACKHOLE/blackhole.ppm
```

### 🌀 Rendu vidéo / Animation

Le mode `video` génère une animation MP4 à partir d'une scène :

```bash
cargo run --release -- video --duration=10 --fps=30 --width=1920 --height=1080 --output-mp4=output/animation.mp4
```

Chargement d'une scène existante :

```bash
cargo run --release -- video --scene-file=my_scene.scene --output-mp4=output/animation.mp4
```

| Option | Défaut | Description |
|:-------|:------:|:------------|
| `--duration=N` | `5` | Durée en secondes |
| `--fps=N` | `30` | Images par seconde |
| `--width=N` | `1280` | Largeur en pixels |
| `--height=N` | `720` | Hauteur en pixels |
| `--quality=preview\|hd\|production` | `preview` | Qualité de rendu |
| `--scene-file=PATH` | *(défaut)* | Scène `.scene` à charger |
| `--output-dir=PATH` | `output/video` | Dossier frames temporaires |
| `--output-mp4=PATH` | `output/video/animation.mp4` | Fichier MP4 final |
| `--prefix=NAME` | `frame` | Préfixe des frames |

<br>

### 📦 Library API

```rust
use enginerenderer::api::engine::Engine;
use enginerenderer::api::scenes::builder::SceneBuilder;
use enginerenderer::api::materials::catalog::MaterialCatalog;
use enginerenderer::api::objects::primitives::RenderPreset;
use enginerenderer::api::animation::{AnimationClip, Timeline, Interpolation, FrameSequencer};
```

<details>
<summary><strong>Code examples</strong></summary>
<br>

**Default render:**
```rust
let report = Engine::default().run()?;
println!("Rendered {}x{} in {} ms", report.width, report.height, report.duration_ms);
```

**Production gallery:**
```rust
let reports = Engine::production_reference().render_gallery()?;
for report in reports {
    println!("{}", report.output_path.display());
}
```

**Smoke test:**
```rust
let report = Engine::test_minimal().run()?;
```

**AI capabilities:**
```rust
let caps = enginerenderer::api::ai::capabilities::Capabilities::discover();
```

**Hardware diagnostics:**
```rust
enginerenderer::api::diagnose_compute_environment();
```

</details>

<br>

## 📁 Output Formats

| Format | Extension | Notes |
|:------:|:---------:|:------|
| PPM | `.ppm` | Default still image |
| EXR | `.exr` | HDR framebuffer |
| PNG | `.png` | Compressed stills |
| MP4 | `.mp4` | Video (FFmpeg) |

<br>

## 📊 Stats

<div align="center">

| Source files | Lines of Rust | External crates | Material presets | Interpolation curves | Render presets | Targets | Edition |
|:---:|:---:|:---:|:---:|:---:|:---:|:---:|:---:|
| **244** | **~26,350** | **0** | **17** | **18** | **4** | **9+** | **2024** |

</div>

<br>

## 🗂️ Repository Structure

```
src/
├── main.rs                     CLI dispatcher (video · run · interactive)
├── lib.rs                      Public library exports
├── generator.rs                Generic video rendering (CLI video mode)
├── realtime.rs                 Realtime preview mode (CLI run mode)
├── api/                        Public API layer
│   ├── scene_descriptor.rs       Serializable scene description
│   ├── generator.rs              GeneratorRequest API type
│   ├── ai/                       AI capabilities, prompt parsing, renderer
│   ├── animation/                Clips, timelines, sequencer
│   ├── camera/                   Controller and presets
│   ├── engine/                   EngineApi · diagnostics · cameras ·
│   │                             descriptor · objects · rendering · scenes
│   ├── materials/                Builder, catalog, spectrum, physics
│   ├── objects/                  Primitives, composites, scene objects
│   ├── scenes/                   Builder and presets
│   └── types/                    Color, config, core types, transforms
│
├── core/                       Internal engine
│   ├── animation/                Clip, timeline, easing, video export
│   ├── coremanager/              All managers (engine, camera, config,
│   │                             time, input, audio, resource, LOD, net)
│   ├── debug/                    Logger, profiling, serialization
│   ├── engine/
│   │   ├── acces_hardware/       CPU · GPU · display · compute · DMA
│   │   ├── config/               Engine configuration
│   │   ├── engineloop/           Main render loop
│   │   ├── event/                Event system
│   │   ├── physics/              Rigid body, collision, raycasting
│   │   ├── rendering/            Raytracing · BVH · shading · shadows
│   │   │                         volumetrics · post-processing · culling
│   │   │                         LOD · framebuffer · texture · loaders
│   │   └── scene/                Graph, world, objects, celestial
│   ├── input/                    Input, camera control, events, audio
│   ├── scheduler/                Tile scheduler, loop control, profiling
│   └── simulation/               N-body gravitational simulation
│
└── utils/                      CLI and interactive terminal
```

<br>

<div align="center">

## License

[MIT](License)

</div>