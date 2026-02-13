use tauri::{AppHandle, command, Emitter};
use tauri::Wry;
use serde_json::json;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::thread::{self, JoinHandle};
use std::os::unix::io::AsRawFd;
use once_cell::sync::Lazy;
use evdev::{Device, EventSummary, KeyCode, RelativeAxisCode};

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

/// Map evdev KeyCode to rdev-compatible key name string.
/// Returns None for keys we don't need to handle.
fn evdev_key_to_rdev_name(key: KeyCode) -> Option<&'static str> {
    Some(match key {
        // Letters
        KeyCode::KEY_A => "KeyA",
        KeyCode::KEY_B => "KeyB",
        KeyCode::KEY_C => "KeyC",
        KeyCode::KEY_D => "KeyD",
        KeyCode::KEY_E => "KeyE",
        KeyCode::KEY_F => "KeyF",
        KeyCode::KEY_G => "KeyG",
        KeyCode::KEY_H => "KeyH",
        KeyCode::KEY_I => "KeyI",
        KeyCode::KEY_J => "KeyJ",
        KeyCode::KEY_K => "KeyK",
        KeyCode::KEY_L => "KeyL",
        KeyCode::KEY_M => "KeyM",
        KeyCode::KEY_N => "KeyN",
        KeyCode::KEY_O => "KeyO",
        KeyCode::KEY_P => "KeyP",
        KeyCode::KEY_Q => "KeyQ",
        KeyCode::KEY_R => "KeyR",
        KeyCode::KEY_S => "KeyS",
        KeyCode::KEY_T => "KeyT",
        KeyCode::KEY_U => "KeyU",
        KeyCode::KEY_V => "KeyV",
        KeyCode::KEY_W => "KeyW",
        KeyCode::KEY_X => "KeyX",
        KeyCode::KEY_Y => "KeyY",
        KeyCode::KEY_Z => "KeyZ",

        // Number row
        KeyCode::KEY_0 => "Num0",
        KeyCode::KEY_1 => "Num1",
        KeyCode::KEY_2 => "Num2",
        KeyCode::KEY_3 => "Num3",
        KeyCode::KEY_4 => "Num4",
        KeyCode::KEY_5 => "Num5",
        KeyCode::KEY_6 => "Num6",
        KeyCode::KEY_7 => "Num7",
        KeyCode::KEY_8 => "Num8",
        KeyCode::KEY_9 => "Num9",

        // Function keys
        KeyCode::KEY_F1 => "F1",
        KeyCode::KEY_F2 => "F2",
        KeyCode::KEY_F3 => "F3",
        KeyCode::KEY_F4 => "F4",
        KeyCode::KEY_F5 => "F5",
        KeyCode::KEY_F6 => "F6",
        KeyCode::KEY_F7 => "F7",
        KeyCode::KEY_F8 => "F8",
        KeyCode::KEY_F9 => "F9",
        KeyCode::KEY_F10 => "F10",
        KeyCode::KEY_F11 => "F11",
        KeyCode::KEY_F12 => "F12",
        KeyCode::KEY_F13 => "F13",
        KeyCode::KEY_F14 => "F14",
        KeyCode::KEY_F15 => "F15",
        KeyCode::KEY_F16 => "F16",
        KeyCode::KEY_F17 => "F17",
        KeyCode::KEY_F18 => "F18",
        KeyCode::KEY_F19 => "F19",
        KeyCode::KEY_F20 => "F20",
        KeyCode::KEY_F21 => "F21",
        KeyCode::KEY_F22 => "F22",
        KeyCode::KEY_F23 => "F23",
        KeyCode::KEY_F24 => "F24",

        // Modifiers
        KeyCode::KEY_LEFTSHIFT => "ShiftLeft",
        KeyCode::KEY_RIGHTSHIFT => "ShiftRight",
        KeyCode::KEY_LEFTCTRL => "ControlLeft",
        KeyCode::KEY_RIGHTCTRL => "ControlRight",
        KeyCode::KEY_LEFTALT => "Alt",
        KeyCode::KEY_RIGHTALT => "AltGr",
        KeyCode::KEY_LEFTMETA => "MetaLeft",
        KeyCode::KEY_RIGHTMETA => "MetaRight",

        // Special keys
        KeyCode::KEY_ESC => "Escape",
        KeyCode::KEY_BACKSPACE => "Backspace",
        KeyCode::KEY_TAB => "Tab",
        KeyCode::KEY_ENTER => "Return",
        KeyCode::KEY_SPACE => "Space",
        KeyCode::KEY_CAPSLOCK => "CapsLock",
        KeyCode::KEY_DELETE => "Delete",
        KeyCode::KEY_INSERT => "Insert",
        KeyCode::KEY_HOME => "Home",
        KeyCode::KEY_END => "End",
        KeyCode::KEY_PAGEUP => "PageUp",
        KeyCode::KEY_PAGEDOWN => "PageDown",
        KeyCode::KEY_SYSRQ => "PrintScreen",
        KeyCode::KEY_SCROLLLOCK => "ScrollLock",
        KeyCode::KEY_PAUSE => "Pause",
        KeyCode::KEY_NUMLOCK => "NumLock",

        // Arrow keys
        KeyCode::KEY_UP => "UpArrow",
        KeyCode::KEY_DOWN => "DownArrow",
        KeyCode::KEY_LEFT => "LeftArrow",
        KeyCode::KEY_RIGHT => "RightArrow",

        // Symbol keys
        KeyCode::KEY_GRAVE => "BackQuote",
        KeyCode::KEY_MINUS => "Minus",
        KeyCode::KEY_EQUAL => "Equal",
        KeyCode::KEY_LEFTBRACE => "LeftBracket",
        KeyCode::KEY_RIGHTBRACE => "RightBracket",
        KeyCode::KEY_BACKSLASH => "BackSlash",
        KeyCode::KEY_SEMICOLON => "SemiColon",
        KeyCode::KEY_APOSTROPHE => "Quote",
        KeyCode::KEY_COMMA => "Comma",
        KeyCode::KEY_DOT => "Dot",
        KeyCode::KEY_SLASH => "Slash",
        KeyCode::KEY_102ND => "IntlBackslash",

        // Keypad
        KeyCode::KEY_KP0 => "Kp0",
        KeyCode::KEY_KP1 => "Kp1",
        KeyCode::KEY_KP2 => "Kp2",
        KeyCode::KEY_KP3 => "Kp3",
        KeyCode::KEY_KP4 => "Kp4",
        KeyCode::KEY_KP5 => "Kp5",
        KeyCode::KEY_KP6 => "Kp6",
        KeyCode::KEY_KP7 => "Kp7",
        KeyCode::KEY_KP8 => "Kp8",
        KeyCode::KEY_KP9 => "Kp9",
        KeyCode::KEY_KPENTER => "KpReturn",
        KeyCode::KEY_KPMINUS => "KpMinus",
        KeyCode::KEY_KPPLUS => "KpPlus",
        KeyCode::KEY_KPASTERISK => "KpMultiply",
        KeyCode::KEY_KPSLASH => "KpDivide",
        KeyCode::KEY_KPDOT => "KpDecimal",
        KeyCode::KEY_KPEQUAL => "KpEqual",
        KeyCode::KEY_KPCOMMA => "KpComma",

        // Volume
        KeyCode::KEY_VOLUMEUP => "VolumeUp",
        KeyCode::KEY_VOLUMEDOWN => "VolumeDown",
        KeyCode::KEY_MUTE => "VolumeMute",

        // International
        KeyCode::KEY_RO => "IntlRo",
        KeyCode::KEY_YEN => "IntlYen",

        _ => return None,
    })
}

/// Map evdev mouse button KeyCode to rdev-compatible button name string.
fn evdev_button_to_rdev_name(key: KeyCode) -> Option<&'static str> {
    Some(match key {
        KeyCode::BTN_LEFT => "Left",
        KeyCode::BTN_RIGHT => "Right",
        KeyCode::BTN_MIDDLE => "Middle",
        _ => return None,
    })
}

/// Find all relevant evdev input devices (mice and keyboards).
fn find_input_devices() -> Result<Vec<Device>, String> {
    let mut devices = Vec::new();
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

        let has_keys = device.supported_keys().map_or(false, |keys| {
            keys.contains(KeyCode::KEY_A) && keys.contains(KeyCode::KEY_Z)
        });

        if has_rel || has_keys {
            eprintln!("Found input device: {} ({}) [mouse={}, keyboard={}]",
                path.display(),
                device.name().unwrap_or("unknown"),
                has_rel,
                has_keys,
            );
            devices.push(device);
        }
    }

    if devices.is_empty() {
        return Err(
            "No input devices found. Ensure your user is in the 'input' group: sudo usermod -aG input $USER (then log out/in)".into()
        );
    }

    Ok(devices)
}

/// Spawn a reader thread for a single evdev device.
fn spawn_device_reader(mut device: Device, app: AppHandle<Wry>) -> JoinHandle<()> {
    thread::spawn(move || {
        let dev_name = device.name().unwrap_or("unknown").to_string();
        eprintln!("Listening on device: {dev_name}");

        let fd = device.as_raw_fd();

        while IS_LISTENING.load(Ordering::SeqCst) {
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
                    EventSummary::Key(_raw, key, value) => {
                        // value: 1 = press, 0 = release, 2 = repeat (ignore repeats)
                        if value == 2 {
                            continue;
                        }
                        let pressed = value == 1;

                        // Check mouse buttons first
                        if let Some(button_name) = evdev_button_to_rdev_name(key) {
                            let event = DeviceEvent {
                                kind: if pressed { DeviceEventKind::MousePress } else { DeviceEventKind::MouseRelease },
                                value: json!(button_name),
                            };
                            let _ = app.emit("device-changed", event);
                            continue;
                        }

                        // Then keyboard keys
                        if let Some(key_name) = evdev_key_to_rdev_name(key) {
                            let event = DeviceEvent {
                                kind: if pressed { DeviceEventKind::KeyboardPress } else { DeviceEventKind::KeyboardRelease },
                                value: json!(key_name),
                            };
                            let _ = app.emit("device-changed", event);
                        }
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

    let devices = find_input_devices()?;

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

    for handle in s.threads.drain(..) {
        let _ = handle.join();
    }

    s.app = None;
    Ok(())
}
