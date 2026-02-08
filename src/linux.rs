// Stub for Linux implementations (evdev / libinput / x11)
// TODO: implement libinput/evdev/XInput2 backends behind feature flags

use serde_json::json;
use crate::{DeviceEvent, DeviceEventKind};

pub async fn start_raw_input<R>(_app: tauri::AppHandle<R>) -> Result<(), String>
where
    R: tauri::Runtime,
{
    Err("Linux raw input not implemented yet; enable linux-evdev/linux-libinput features".into())
}

pub fn stop_raw_input() -> Result<(), String> {
    Err("not implemented".into())
}
