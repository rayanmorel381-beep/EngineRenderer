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
├── quickstat.md
├── ChangeLog.md
├── License
├── arborescence.md
├── src/
│   ├── main.rs
│   ├── lib.rs
│   ├── generator.rs
│   ├── realtime.rs
│   ├── api/
│   │   ├── mod.rs
│   │   ├── scene_descriptor.rs
│   │   ├── generator.rs
│   │   ├── ai/
│   │   │   ├── mod.rs
│   │   │   ├── ai_manager.rs
│   │   │   ├── capabilities.rs
│   │   │   ├── prompt.rs
│   │   │   └── renderer.rs
│   │   ├── animation/
│   │   │   ├── mod.rs
│   │   │   └── animation_api.rs
│   │   ├── camera/
│   │   │   ├── mod.rs
│   │   │   ├── controller.rs
│   │   │   └── presets.rs
│   │   ├── engine/
│   │   │   ├── mod.rs
│   │   │   ├── cameras.rs
│   │   │   ├── descriptor.rs
│   │   │   ├── diagnostics.rs
│   │   │   ├── engine_api.rs
│   │   │   ├── objects.rs
│   │   │   ├── rendering.rs
│   │   │   └── scenes.rs
│   │   ├── materials/
│   │   │   ├── mod.rs
│   │   │   ├── builder.rs
│   │   │   ├── catalog.rs
│   │   │   ├── physics.rs
│   │   │   ├── shortcuts.rs
│   │   │   └── spectrum.rs
│   │   ├── objects/
│   │   │   ├── mod.rs
│   │   │   ├── composites.rs
│   │   │   ├── primitives.rs
│   │   │   └── scene_object.rs
│   │   ├── scenes/
│   │   │   ├── mod.rs
│   │   │   ├── builder.rs
│   │   │   └── presets.rs
│   │   └── types/
│   │       ├── mod.rs
│   │       ├── color.rs
│   │       ├── config.rs
│   │       ├── core.rs
│   │       └── transform.rs
│   ├── core/
│   │   ├── mod.rs
│   │   ├── animation/
│   │   ├── coremanager/
│   │   ├── debug/
│   │   │   ├── mod.rs
│   │   │   ├── logger.rs
│   │   │   ├── profiling.rs
│   │   │   ├── serialization.rs
│   │   │   └── tools.rs
│   │   ├── engine/
│   │   │   ├── mod.rs
│   │   │   ├── acces_hardware/
│   │   │   ├── config/
│   │   │   ├── engineloop/
│   │   │   ├── event/
│   │   │   ├── physics/
│   │   │   ├── rendering/
│   │   │   └── scene/
│   │   ├── input/
│   │   ├── scheduler/
│   │   └── simulation/
│   └── utils/
│       ├── mod.rs
│       └── terminal_mode/
```

## 🧭 Main Directory Roles

- src/api: High-level API for building scenes, materials, cameras, animation, and AI integration.
- src/core: Internal engine systems (rendering, physics, simulation, scheduler, and managers).
- src/utils: Utility layer, including the interactive terminal mode.
- .github/workflows: CI pipelines for x86 and ARM architectures.

## 🚪 Entry Points

- src/main.rs: CLI dispatcher — routes `video`, `run`, interactive, and other commands.
- src/lib.rs: Public library exports.
- src/generator.rs: Generic video/animation rendering (CLI `video` mode).
- src/realtime.rs: Realtime preview mode (CLI `run` mode).
- src/api/generator.rs: `GeneratorRequest` API type and builder.
- src/api/engine/diagnostics.rs: Hardware diagnostics and compute environment API.
