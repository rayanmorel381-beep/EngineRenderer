<div align="center">

# 📋 Changelog

All notable changes to **EngineRenderer** are documented here.

Format: [Keep a Changelog](https://keepachangelog.com/en/1.1.0/)

</div>

---

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
- CLI mode — `render` · `gallery` · `test` · `help`
- 7 render examples — spheres · cubes · house · city · car · world · blackhole
- Standalone `generate_mp4.rs` black hole accretion disk animation

### ⚙️ Infrastructure

- Engine logging — circular buffer · 3 severity levels
- Adaptive tile-based work scheduler with profiling
- Output formats — PPM · EXR · PNG · MP4