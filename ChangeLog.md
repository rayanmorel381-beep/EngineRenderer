<div align="center">

# 📋 Changelog

All notable changes to **EngineRenderer** are documented here.

Format: [Keep a Changelog](https://keepachangelog.com/en/1.1.0/)

</div>

---

## `0.0.2` — 2026-04-16

> Realtime 120 FPS profile hardening, cross-architecture runtime validation, and Windows target Clippy cleanup.

### ⚡ Realtime Performance

- Unified ultra-constrained 120 FPS profile across architectures
- Realtime scene complexity budget added (proxy meshes, light budget, optional panorama)
- Showcase mesh caching and BVH reuse moved outside the realtime frame loop
- Deadline-based frame pacing with drift compensation for stable high-frequency loops
- Ultra-constrained presentation path tuned to reduce per-frame overhead

### 📊 Runtime Validation

- Linux x86_64 release realtime benchmark (3s, target 120 FPS): achieved > 120 FPS
- Android ARM64 release realtime benchmark (3s, target 120 FPS): achieved > 120 FPS
- Runtime tuning now scales internal resolution and scene cost to preserve target cadence

### 🪟 Windows Target Quality

- Windows x86_64 vendor paths wired so CPU/GPU detection code is actively used
- Dead-code warning sources on Windows vendor modules eliminated through integration
- Clippy warnings fixed on `x86_64-pc-windows-gnu`:
	- collapsed nested `if`
	- replaced manual `div_ceil` patterns
	- replaced manual clamp patterns with `clamp`
	- renamed acronym aliases (`HKEY` → `Hkey`) to satisfy lint rules
- `cargo clippy --target x86_64-pc-windows-gnu` now passes cleanly

### 🧱 Build & Docs

- Strict docs check kept clean (`RUSTFLAGS='-W missing-docs' cargo check`)
- Cross-target checks revalidated after perf/runtime refactors

## `0.0.1` — 2026-04-14

> Initial release — full rendering pipeline, animation system, multi-platform HAL.

### 🎨 Rendering

- CPU path-tracing renderer with BVH acceleration structure
- 16-band spectral rendering with CIE 1931 XYZ conversion
- PBR material system — albedo · roughness · metallic · emission · IOR · subsurface · anisotropy · iridescence · clearcoat · transmission
- 17 built-in material presets — stellar · planetary · manufactured · celestial · natural
- 4 render presets — `AnimationFast` · `PreviewCpu` · `UltraHdCpu` · `ProductionReference`
- Adaptive sampling with firefly rejection and denoising
- Cascaded shadow maps — PCF filtering · contact shadows
- Volumetrics — god rays · participating media · atmospheric scattering · procedural clouds
- Post-processing — bloom · Gaussian blur · depth-of-field · tone mapping
- Motion blur via time-sampled primitives
- Frustum culling and LOD management with hysteresis

### 🎬 Animation

- Timeline-based keyframe system with 18 interpolation methods
- Frame sequencer for batch rendering with caching
- MP4 video export via FFmpeg

### 🏗️ Scene

- Fluent builder API — `SceneBuilder` · `MaterialBuilder` · `CameraController`
- Text-based `.scene` format with serialization and parsing
- OBJ and glTF/GLB mesh import
- Procedural geometry generators and mesh operations
- Composite objects with hierarchical scene graphs

### ⚛️ Physics & Simulation

- Rigid body dynamics · collision detection · raycasting
- N-body gravitational simulation for celestial mechanics

### 🤖 AI

- Capability introspection · prompt-to-scene parsing · AI-friendly camera presets

### 🖥️ Hardware Abstraction

- Per-OS CPU/GPU/display detection — Linux · macOS · Windows · Android
- Runtime SIMD — SSE4.2 · AVX · AVX-512 (x86) — NEON · SVE (ARM)
- GPU drivers — `amdgpu` `radeon` `i915` `xe` `nouveau` `mali` `msm` — D3D12/DXGI — AppleGPU
- Automatic CPU fallback
- Cross-compilation for 9+ targets
- Compilation successful on all listed architectures

### 🛠️ Tools & CLI

- Interactive terminal REPL — module navigation · documentation browser · framed UI panels
- CLI mode — `render` · `gallery` · `test` · `video` · `run` · `detect` · `help`
- 7 render examples — spheres · cubes · house · city · car · world · blackhole
- `video` mode — animation MP4 générique avec chargement de scène `.scene` via `--scene-file`
- `run` mode — prévisualisation realtime paramétrable (`--seconds`, `--fps`, `--width`, `--height`)

### ⚙️ Infrastructure

- Engine logging — circular buffer · 3 severity levels
- Adaptive tile-based work scheduler with profiling
- Output formats — PPM · EXR · PNG · MP4