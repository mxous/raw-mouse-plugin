Examples for raw-mouse-plugin

This folder demonstrates how to integrate `raw-mouse-plugin` into a Tauri application's `src-tauri` crate.

1. Add the crate as a path dependency in your Tauri `src-tauri/Cargo.toml`:

```toml
raw-mouse-plugin = { path = "../../raw-mouse-plugin", version = "0.1", features = ["windows"] }
```

2. Add plugin commands to your `generate_handler!` list in `src-tauri/src/lib.rs`:

```rust
use raw_mouse_plugin::{start_raw_input, stop_raw_input, set_rumble_value};

.invoke_handler(generate_handler![
    start_raw_input,
    stop_raw_input,
    set_rumble_value,
    // other commands...
])
```

3. On the frontend, the plugin emits `device-changed` events with the same payload shape used previously, so existing listeners should continue working.

Notes:
- On Linux, you will need to enable appropriate features and provide udev permissions for raw device access.
- On macOS, the plugin will need Accessibility permission to capture global events when using `CGEventTap`.
