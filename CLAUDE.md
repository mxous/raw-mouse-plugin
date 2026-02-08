# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

A Rust library crate providing cross-platform raw mouse input capture for Tauri 2 applications. Currently only the Windows implementation is complete; Linux and macOS modules are stubs with planned backends (evdev/libinput and CGEventTap respectively).

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

- **`lib.rs`** — Entry point. Defines shared types (`DeviceEvent`, `DeviceEventKind`) and conditionally re-exports platform modules based on `#[cfg(windows)]` / feature flags.
- **`windows.rs`** — Complete Windows implementation using Win32 Raw Input API. Registers `RAWINPUTDEVICE` for mouse input, installs a `WH_GETMESSAGE` hook on the main window's thread, and processes `WM_INPUT` messages. Emits `"device-changed"` events to the Tauri frontend via `app.emit()`.
- **`linux.rs`** / **`macos.rs`** — Stubs returning "not implemented" errors.

### Windows Implementation Key Details

- **Global state**: `PluginState` in a `Lazy<Mutex<>>` holds the Tauri `AppHandle`, hook handle, mouse position, and tracking mode. `IS_LISTENING` is an `AtomicBool`.
- **Hook handle**: Stored in `thread_local!` (`HOOK_HANDLE`) because `SetWindowsHookExW` must be called and unhooked from the same thread.
- **Tracking modes**: `Relative` emits raw dx/dy deltas from `RAWINPUT`; `Absolute` calls `GetCursorPos()` for screen coordinates. Mode switching dynamically installs/uninstalls the hook.
- **Tauri commands** (all `#[command]`): `start_raw_input`, `stop_raw_input`, `get_mouse_position`, `set_tracking_mode`, `get_tracking_mode`.

### Example App (`examples/tauri-app/`)

A minimal Tauri 2 app demonstrating the plugin. Frontend is plain HTML/TS served by Vite (port 1430). Backend in `src-tauri/` wraps the library commands via `generate_handler![]` and references the library as a path dependency.

## Feature Flags

Defined in `Cargo.toml` but only `windows` (implicitly via `#[cfg(windows)]`) is functional:

| Feature | Status |
|---------|--------|
| `windows` | Functional (compiles via `cfg(windows)` target) |
| `linux-evdev`, `linux-libinput`, `x11` | Planned stubs |
| `macos-cgevent`, `macos-iohid` | Planned stubs |
| `absolute` | Declared but unused |

## Key Conventions

- All Tauri commands return `Result<T, String>` — errors are stringified.
- Event payload format: `DeviceEvent { kind: DeviceEventKind, value: serde_json::Value }` emitted on the `"device-changed"` channel.
- Unsafe blocks are used for Win32 API calls (`SetWindowsHookExW`, `GetRawInputData`, `RegisterRawInputDevices`, etc.) and are isolated to `windows.rs`.
- The `tauri` dependency is only compiled on the Windows target (`[target.'cfg(windows)'.dependencies]`), meaning this crate currently cannot compile on non-Windows platforms without feature/target adjustments.
