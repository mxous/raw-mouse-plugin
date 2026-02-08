# Raw Mouse Plugin Example App

A minimal Tauri application to test the raw-mouse-plugin on Windows.

## Running the example

From this directory (`examples/tauri-app/`), run:

```bash
cd src-tauri
cargo tauri dev
```

This will launch the development server and open the example app window.

## How it works

1. **Start Listening** — calls `start_raw_input` command, which registers raw input devices and begins emitting `device-changed` events
2. **Stop Listening** — calls `stop_raw_input` command to unhook and stop event emission
3. **Event Display** — listens for `device-changed` events and displays mouse movement and button press data in real-time

## Expected behavior on Windows

- Mouse movements should appear as `MouseMove` events with relative `(x, y)` deltas
- Click the Start button and move your mouse to see events appear
- Events show the most recent 50 events (oldest scroll off)

## Troubleshooting

- If you get a "command not found" error, ensure the plugin path in `Cargo.toml` is correct (`path = "../../"`)
- If no events appear after clicking Start, check the browser console (F12) for errors
- Ensure your Tauri environment is set up correctly (NodeJS, Rust toolchain, etc.)
