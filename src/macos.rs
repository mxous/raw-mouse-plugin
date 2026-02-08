// Stub for macOS implementations (CGEventTap / IOHID)
// TODO: implement CGEventTap-based backend and optional IOHIDManager backend

use serde_json::json;
use crate::{DeviceEvent, DeviceEventKind};

pub async fn start_raw_input<R>(_app: tauri::AppHandle<R>) -> Result<(), String>
where
    R: tauri::Runtime,
{
    Err("macOS raw input not implemented yet; enable macos-cgevent feature".into())
}

pub fn stop_raw_input() -> Result<(), String> {
    Err("not implemented".into())
}
