# Raw Mouse Plugin Example App

A minimal Tauri 2 application demonstrating the raw-mouse-plugin on Windows.

## Running the example

From the repo root:

```bash
cd examples/tauri-app
pnpm install
pnpm dev
```

## How it works

1. **Start Listening** — calls `start_raw_input`, which registers raw input devices and begins emitting `device-changed` events with relative mouse deltas.
2. **Stop Listening** — calls `stop_raw_input` to unhook and stop event emission.
3. **Event Display** — listens for `device-changed` events and displays mouse movement data in real-time.

## Integration guide

1. Add the crate as a path dependency in your Tauri `src-tauri/Cargo.toml`:

```toml
raw-mouse-plugin = { path = "../../raw-mouse-plugin", version = "0.1", features = ["windows"] }
```

2. Add plugin commands to your `generate_handler!` list in `src-tauri/src/lib.rs`:

```rust
use raw_mouse_plugin::{start_raw_input, stop_raw_input};

.invoke_handler(generate_handler![
    start_raw_input,
    stop_raw_input,
])
```

3. On the frontend, listen for `device-changed` events to receive relative `(dx, dy)` mouse deltas.

## Troubleshooting

- If you get a "command not found" error, ensure the plugin path in `Cargo.toml` is correct.
- If no events appear after clicking Start, check the browser console (F12) for errors.
- Ensure your Tauri environment is set up correctly (Node.js, Rust toolchain, etc.).
