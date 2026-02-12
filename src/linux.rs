use tauri::{AppHandle, command, Emitter};
use tauri::Wry;
use serde_json::json;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::thread::{self, JoinHandle};
use std::os::unix::io::AsRawFd;
use once_cell::sync::Lazy;
use evdev::{Device, EventSummary, RelativeAxisCode};

use crate::{DeviceEvent, DeviceEventKind};

static IS_LISTENING: AtomicBool = AtomicBool::new(false);

struct PluginState {
    app: Option<AppHandle<Wry>>,
    threads: Vec<JoinHandle<()>>,
}

static STATE: Lazy<Mutex<PluginState>> = Lazy::new(|| {
    Mutex::new(PluginState {
        app: None,
        threads: Vec::new(),
    })
});

/// Find all evdev devices that support relative X/Y axes (mice, trackballs, etc.)
fn find_mouse_devices() -> Result<Vec<Device>, String> {
    let mut mice = Vec::new();
    let entries = std::fs::read_dir("/dev/input")
        .map_err(|e| format!("Failed to read /dev/input: {e}"))?;

    for entry in entries.flatten() {
        let name = match entry.file_name().into_string() {
            Ok(n) => n,
            Err(_) => continue,
        };
        if !name.starts_with("event") {
            continue;
        }

        let path = entry.path();
        let device = match Device::open(&path) {
            Ok(d) => d,
            Err(_) => continue,
        };

        let has_rel = device.supported_relative_axes().map_or(false, |axes| {
            axes.contains(RelativeAxisCode::REL_X) && axes.contains(RelativeAxisCode::REL_Y)
        });

        if has_rel {
            eprintln!("Found mouse device: {} ({})", path.display(), device.name().unwrap_or("unknown"));
            mice.push(device);
        }
    }

    if mice.is_empty() {
        return Err(
            "No mouse devices found. Ensure your user is in the 'input' group: sudo usermod -aG input $USER (then log out/in)".into()
        );
    }

    Ok(mice)
}

/// Spawn a reader thread for a single evdev device.
/// Each thread loops on fetch_events() and emits relative deltas back to Tauri.
fn spawn_device_reader(mut device: Device, app: AppHandle<Wry>) -> JoinHandle<()> {
    thread::spawn(move || {
        let dev_name = device.name().unwrap_or("unknown").to_string();
        eprintln!("Listening on device: {dev_name}");

        let fd = device.as_raw_fd();

        while IS_LISTENING.load(Ordering::SeqCst) {
            // Poll with 100ms timeout so the thread can check IS_LISTENING periodically
            // instead of blocking indefinitely on fetch_events().
            let mut pollfd = libc::pollfd {
                fd,
                events: libc::POLLIN,
                revents: 0,
            };
            let ret = unsafe { libc::poll(&mut pollfd, 1, 100) };
            if ret <= 0 {
                continue;
            }

            let events = match device.fetch_events() {
                Ok(evs) => evs,
                Err(e) => {
                    use std::io::ErrorKind;
                    match e.kind() {
                        ErrorKind::NotFound => {
                            eprintln!("Device removed: {dev_name}");
                            break;
                        }
                        ErrorKind::WouldBlock | ErrorKind::Interrupted => continue,
                        _ => {
                            eprintln!("Error reading {dev_name}: {e}");
                            break;
                        }
                    }
                }
            };

            for ev in events {
                if !IS_LISTENING.load(Ordering::SeqCst) {
                    break;
                }
                match ev.destructure() {
                    EventSummary::RelativeAxis(_raw, axis, value) => {
                        let (dx, dy) = match axis {
                            RelativeAxisCode::REL_X => (value, 0),
                            RelativeAxisCode::REL_Y => (0, value),
                            _ => continue,
                        };

                        let event = DeviceEvent {
                            kind: DeviceEventKind::MouseMove,
                            value: json!({ "x": dx, "y": dy }),
                        };
                        let _ = app.emit("device-changed", event);
                    }
                    _ => {}
                }
            }
        }

        eprintln!("Stopped listening on device: {dev_name}");
    })
}

#[command]
pub fn start_raw_input(app: AppHandle<Wry>) -> Result<(), String> {
    if IS_LISTENING.load(Ordering::SeqCst) {
        return Ok(());
    }

    let devices = find_mouse_devices()?;

    IS_LISTENING.store(true, Ordering::SeqCst);

    let mut s = STATE.lock().map_err(|e| e.to_string())?;
    s.app = Some(app.clone());

    for device in devices {
        let handle = spawn_device_reader(device, app.clone());
        s.threads.push(handle);
    }

    Ok(())
}

#[command]
pub fn stop_raw_input() -> Result<(), String> {
    if !IS_LISTENING.load(Ordering::SeqCst) {
        return Ok(());
    }

    IS_LISTENING.store(false, Ordering::SeqCst);

    let mut s = STATE.lock().map_err(|e| e.to_string())?;

    // The threads will exit on their own once IS_LISTENING is false,
    // but fetch_events() blocks until the next event arrives.
    // We drain the handles here; they'll finish once the user moves the mouse.
    for handle in s.threads.drain(..) {
        let _ = handle.join();
    }

    s.app = None;
    Ok(())
}
