# raw-mouse-plugin

A Rust library crate providing cross-platform raw mouse input capture for Tauri 2 applications. The plugin operates in relative mode only, emitting raw dx/dy deltas from the OS input system.

Currently only the Windows implementation is complete; Linux and macOS modules are stubs with planned backends.

## Features

- **Windows** — Raw Input API via `WH_GETMESSAGE` hook, emitting relative mouse deltas.
- **Linux** — Planned `libinput`/`evdev` backends (feature gated, currently stubs).
- **macOS** — Planned `CGEventTap` backend (feature gated, currently stub).

## Quick Usage

Add as a dependency in your Tauri app's `src-tauri/Cargo.toml`:

```toml
[dependencies]
raw-mouse-plugin = { path = "../../raw-mouse-plugin", version = "0.1" }
```

In your Tauri backend (`src-tauri/src/lib.rs`):

```rust
use raw_mouse_plugin::{start_raw_input, stop_raw_input};

// include in the invoke handler
.invoke_handler(generate_handler![
    start_raw_input,
    stop_raw_input,
])
```

The plugin emits `device-changed` events with a `DeviceEvent` payload:

```json
{ "kind": "MouseMove", "value": { "x": <dx>, "y": <dy> } }
```

Listen for these events on the frontend via Tauri's `listen()` API.

## API

| Command | Description |
|---------|-------------|
| `start_raw_input` | Registers raw input devices and installs the hook. Begins emitting `device-changed` events with relative mouse deltas. |
| `stop_raw_input` | Uninstalls the hook and stops emitting events. |

## Platform Notes

- **Windows (Raw Input):** No extra privileges required. The plugin registers `RAWINPUTDEVICE` with `RIDEV_INPUTSINK` and hooks the main window's thread.
- **Linux (evdev/libinput):** Capturing raw device events typically requires read access to `/dev/input/event*`. A udev rule or `input` group membership may be needed. Under Wayland, prefer `libinput` integration.
- **macOS (CGEventTap / IOHID):** Global event taps require Accessibility permission under System Settings > Privacy & Security > Accessibility.

## Example App

See `examples/tauri-app/` for a minimal Tauri 2 app demonstrating the plugin. It displays accumulated relative mouse deltas in real time.

```bash
cd examples/tauri-app && pnpm install && pnpm dev
```
