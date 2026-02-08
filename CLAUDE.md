# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

A Rust library crate providing cross-platform raw mouse input capture for Tauri 2 applications. The plugin is relative-mode-only, always emitting raw dx/dy deltas. Currently only the Windows implementation is complete; Linux and macOS modules are stubs with planned backends (evdev/libinput and CGEventTap respectively).

## Build Commands

```bash
# Build the library
cargo build

# Check compilation without producing binaries
cargo check

# Run the example Tauri app (from repo root)
cd examples/tauri-app && pnpm install && pnpm dev

# Build example app backend only
cd examples/tauri-app/src-tauri && cargo build
```

There are no tests or linting configured in this project yet.

## Architecture

### Library Crate (`src/`)

- **`lib.rs`** — Entry point. Defines shared types (`DeviceEvent`, `DeviceEventKind`) and conditionally re-exports platform modules based on `#[cfg(windows)]` / feature flags. Public API is `start_raw_input` and `stop_raw_input`.
- **`windows.rs`** — Complete Windows implementation using Win32 Raw Input API. Registers `RAWINPUTDEVICE` for mouse input, installs a `WH_GETMESSAGE` hook on the main window's thread, and processes `WM_INPUT` messages. Always emits relative dx/dy deltas via `"device-changed"` events to the Tauri frontend.
- **`linux.rs`** / **`macos.rs`** — Stubs returning "not implemented" errors.

### Windows Implementation Key Details

- **Global state**: `PluginState` in a `Lazy<Mutex<>>` holds the Tauri `AppHandle` and hook handle. `IS_LISTENING` is an `AtomicBool`.
- **Hook lifecycle**: The hook is always installed when `start_raw_input` is called and uninstalled when `stop_raw_input` is called. The hook always emits relative mouse deltas from `RAWINPUT`.
- **Tauri commands** (all `#[command]`): `start_raw_input`, `stop_raw_input`.

### Example App (`examples/tauri-app/`)

A minimal Tauri 2 app demonstrating the plugin. Frontend is plain HTML/TS served by Vite (port 1430). Backend in `src-tauri/` wraps the library commands via `generate_handler![]` and references the library as a path dependency. Displays accumulated relative mouse deltas in real time.

## Feature Flags

Defined in `Cargo.toml` but only `windows` (implicitly via `#[cfg(windows)]`) is functional:

| Feature | Status |
|---------|--------|
| `windows` | Functional (compiles via `cfg(windows)` target) |
| `linux-evdev`, `linux-libinput`, `x11` | Planned stubs |
| `macos-cgevent`, `macos-iohid` | Planned stubs |

## Key Conventions

- All Tauri commands return `Result<T, String>` — errors are stringified.
- Event payload format: `DeviceEvent { kind: DeviceEventKind, value: serde_json::Value }` emitted on the `"device-changed"` channel.
- Unsafe blocks are used for Win32 API calls (`SetWindowsHookExW`, `GetRawInputData`, `RegisterRawInputDevices`, etc.) and are isolated to `windows.rs`.
- The `tauri` dependency is only compiled on the Windows target (`[target.'cfg(windows)'.dependencies]`), meaning this crate currently cannot compile on non-Windows platforms without feature/target adjustments.
