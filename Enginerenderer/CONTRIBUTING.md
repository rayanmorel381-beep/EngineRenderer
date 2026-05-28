# Contributing

## Pull Request Requirements

Every pull request **must** include the full output of both commands run against the target architecture of the change:

```bash
cargo check --target <your-arch>
cargo clippy --target <your-arch>
```

Paste the complete terminal output (stdout + stderr) directly in the PR description, even if it is clean. PRs without these logs will not be reviewed.

Any warning or error reported by `cargo check` or `cargo clippy` must be resolved before the PR can be accepted. No CI actions, bots, or automated workflows are used — verification is manual and based solely on the provided logs.

## Why

This project targets multiple architectures and platforms. Each architecture can surface different compilation issues, type mismatches, or lint violations. Providing the logs for your specific target makes it possible to verify correctness without running the build locally.

## Supported Targets

| Target | Arch | OS | Status |
|:-------|:-----|:---|:------:|
| `x86_64-unknown-linux-gnu` | x86_64 | Linux | ✅ Stable |
| `aarch64-unknown-linux-gnu` | ARM64 | Linux | ✅ Stable |
| `aarch64-linux-android` | ARM64 | Android | ✅ Stable |
| `x86_64-pc-windows-gnu` | x86_64 | Windows | 🔜 Arriving soon |
| `i686-pc-windows-gnu` | x86 | Windows | 🔜 Arriving soon |
| `x86_64-pc-windows-msvc` | x86_64 | Windows | 🔜 Arriving soon |
| `i686-pc-windows-msvc` | x86 | Windows | 🔜 Arriving soon |
| `aarch64-pc-windows-msvc` | ARM64 | Windows | 🔜 Arriving soon |
| `i686-unknown-linux-gnu` | x86 | Linux | ⚠️ Not tested |
| `armv7-unknown-linux-gnueabihf` | ARMv7 | Linux | ⚠️ Not tested |
| `x86_64-unknown-linux-musl` | x86_64 | Linux musl | ⚠️ Not tested |
| `i686-unknown-linux-musl` | x86 | Linux musl | ⚠️ Not tested |
| `aarch64-unknown-linux-musl` | ARM64 | Linux musl | ⚠️ Not tested |
| `x86_64-apple-darwin` | x86_64 | macOS Intel | ⚠️ Not tested |
| `aarch64-apple-darwin` | ARM64 | macOS Silicon | ⚠️ Not tested |
| `aarch64-apple-ios` | ARM64 | iOS | ⚠️ Not tested |

If your change touches platform-specific code, include logs for all affected targets.

## Realtime 120 FPS Stability Test

If your change touches the render loop, realtime profile, scene building, or any performance-sensitive path, you must also include the output of the following benchmark:

```bash
cargo run --release -- run --fps 120 --width 1280 --height 720 --seconds 3
```

The achieved FPS must be above 120 for the test to pass. Include the full terminal output in the PR description alongside the check and clippy logs.

### Example PR Description Format (with FPS test)

```
## cargo check

Target: aarch64-unknown-linux-gnu

<paste full output here>

## cargo clippy

Target: aarch64-unknown-linux-gnu

<paste full output here>

## realtime 120 FPS benchmark

Target: aarch64-unknown-linux-gnu
Command: cargo run --release -- run --fps 120 --width 1280 --height 720 --seconds 3

<paste full output here>
```
