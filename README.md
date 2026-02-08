# raw-mouse-plugin

A small Rust crate providing cross-platform mouse tracking (relative and optional absolute) designed for Tauri apps.

Features:
- Windows raw input (feature `windows`) — ports existing raw input implementation.
- Linux: planned `libinput`/`evdev` backends (feature gated).
- macOS: planned `CGEventTap` backend (feature gated).
- Optional `absolute` feature to wrap `rdev`.

Quick usage (in a Tauri app `src-tauri/Cargo.toml`):

Add as a path dependency:

```toml
[dependencies]
raw-mouse-plugin = { path = "../../raw-mouse-plugin", version = "0.1", features = ["windows"] }
```

In your Tauri backend (example `src-tauri/src/lib.rs`):

```rust
use raw_mouse_plugin::{start_raw_input, stop_raw_input, set_rumble_value};

// include in the invoke handler
.invoke_handler(generate_handler![
    start_raw_input,
    stop_raw_input,
    set_rumble_value,
    // ...other handlers
])
```

The plugin emits the same `device-changed` event payload as the original implementation, so no frontend changes should be required.

Permissions & platform notes
--

- Linux (evdev/libinput): capturing raw device events typically requires read access to `/dev/input/event*`. Provide a udev rule to grant your user access, for example:

```ini
# /etc/udev/rules.d/99-raw-mouse.rules
KERNEL=="event*", SUBSYSTEM=="input", ATTRS{name}=="Your Device Name", MODE="0660", GROUP="input"
```

You may need to add your user to the `input` group or run the app with elevated privileges for raw `evdev` access. When running under Wayland, many compositors do not allow clients to open input devices directly — prefer `libinput` integration where possible.

- macOS (CGEventTap / IOHID): global event taps require Accessibility permission. The user must allow the app under System Settings → Privacy & Security → Accessibility. When distributing a signed app, ensure code-signing and proper entitlements for capturing global events.

- Windows (Raw Input): no extra privileges are required for registering raw input devices with `RIDEV_INPUTSINK`, but the plugin must correctly register/unregister hooks to avoid resource leaks.

Documentation and examples in this repository will show how to enable the platform-specific feature flags and how to set up the required permissions on each OS.
