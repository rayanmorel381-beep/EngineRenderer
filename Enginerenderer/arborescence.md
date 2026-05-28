<div align="center">

# 🗂️ EngineRenderer Tree

Project structure overview and module map.

</div>

## 🌲 Project Overview

```text
EngineRenderer/
├── .github/
│   └── workflows/
│       ├── ci-arm.yml
│       └── ci-x86.yml
├── Cargo.toml
├── Cargo.lock
├── ReadMe.md
├── quickstart.md
├── ChangeLog.md
├── CONTRIBUTING.md
├── License
├── arborescence.md
├── benches/
│   ├── android_gpu.rs
│   ├── desktop_gpu.rs
│   ├── parsers.rs
│   └── run_android_bench.sh
├── examples/
│   ├── gpu_render_demo.rs
│   ├── gpu_render_heavy.rs
│   └── gpu_window.rs
├── fpstests/
│   ├── aarch64-linux-android.md
│   └── x86_64-unknown-linux-gnu.md
├── src/
│   ├── main.rs
│   ├── lib.rs
│   ├── generator.rs
│   ├── realtime.rs
│   ├── api/
│   │   ├── mod.rs
│   │   ├── display.rs
│   │   ├── loaders.rs
│   │   ├── rendering.rs
│   │   ├── ai/
│   │   ├── animation/
│   │   ├── camera/
│   │   ├── engine/
│   │   ├── materials/
│   │   ├── objects/
│   │   ├── scenes/
│   │   └── types/
│   ├── core/
│   │   ├── mod.rs
│   │   ├── animation/
│   │   ├── coremanager/
│   │   │   ├── mod.rs
│   │   │   ├── audio_device.rs
│   │   │   ├── audio_manager.rs
│   │   │   ├── camera_manager.rs
│   │   │   ├── config_manager.rs
│   │   │   ├── engine_manager.rs
│   │   │   ├── hot_reload.rs
│   │   │   ├── hrtf.rs
│   │   │   ├── input_manager.rs
│   │   │   ├── lod_manager.rs
│   │   │   ├── mixer.rs
│   │   │   ├── netcode.rs
│   │   │   ├── network_manager.rs
│   │   │   ├── resource_manager.rs
│   │   │   ├── reverb.rs
│   │   │   └── time_manager.rs
│   │   ├── debug/
│   │   ├── ecs/
│   │   │   ├── mod.rs
│   │   │   ├── component.rs
│   │   │   ├── entity.rs
│   │   │   ├── system.rs
│   │   │   └── world.rs
│   │   ├── engine/
│   │   ├── input/
│   │   ├── scheduler/
│   │   ├── scripting/
│   │   │   ├── mod.rs
│   │   │   ├── bytecode.rs
│   │   │   ├── compiler.rs
│   │   │   └── vm.rs
│   │   └── simulation/
│   └── utils/
│       ├── mod.rs
│       └── terminal_mode/
├── tests/
│   ├── android_benchmark_contract.rs
│   ├── android_gpu_backend.rs
│   ├── diagnose_smoke.rs
│   ├── engine_api_basics.rs
│   ├── generator_request.rs
│   ├── gpu_raytracer_smoke.rs
│   ├── material_catalog.rs
│   ├── math_utils.rs
│   ├── output_format.rs
│   ├── realtime_profiles.rs
│   ├── render_request_clamp.rs
│   ├── render_request_presets.rs
│   ├── run_android_tests.sh
│   ├── scene_descriptor_robustness.rs
│   └── workspace_render_validation.rs
```

## 🧭 Main Directory Roles

- src/api: High-level API for building scenes, materials, cameras, animation, and AI integration.
- src/core: Internal engine systems (rendering, physics, simulation, scheduler, managers, ECS, scripting VM).
- src/core/ecs: Entity/Component/System core with sparse-set `TypedStorage` and `ComponentRegistry`.
- src/core/scripting: Stack-based scripting VM (`Vm`), bytecode (`Opcode`, `Instruction`), and `ScriptRunner`.
- src/core/coremanager: Engine-wide managers including `HotReloader` for script hot reload.
- src/utils: Utility layer, including the interactive terminal mode.
- benches/: Criterion-style benchmark binaries for desktop and Android GPU paths.
- examples/: Standalone GPU render and windowing demos.
- fpstests/: Per-target realtime FPS measurement reports.
- .github/workflows: CI pipelines for x86 and ARM architectures.

## 🚪 Entry Points

- src/main.rs: CLI dispatcher — routes `video`, `run`, interactive, and other commands.
- src/lib.rs: Public library exports.
- src/generator.rs: Generic video/animation rendering (CLI `video` mode).
- src/realtime.rs: Realtime preview mode (CLI `run` mode).
- src/api/loaders.rs: Mesh/scene loaders (OBJ, glTF, `.scene`).
- src/api/rendering.rs: Public rendering entry points.
- src/api/engine/diagnostics.rs: Hardware diagnostics and compute environment API.
