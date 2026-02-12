#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[cfg(windows)]
pub mod windows;

#[cfg(target_os = "linux")]
pub mod linux;

#[cfg(any(feature = "macos-cgevent", target_os = "macos"))]
pub mod macos;

use serde::Serialize;
use serde_json::Value;

#[derive(Debug, Clone, Serialize)]
pub enum DeviceEventKind {
    MousePress,
    MouseRelease,
    MouseMove,
    KeyboardPress,
    KeyboardRelease,
}

#[derive(Debug, Clone, Serialize)]
pub struct DeviceEvent {
    pub kind: DeviceEventKind,
    pub value: Value,
}

#[cfg(windows)]
pub use windows::{start_raw_input, stop_raw_input};

#[cfg(target_os = "linux")]
pub use linux::{start_raw_input, stop_raw_input};

#[cfg(feature = "macos-cgevent")]
pub use macos::{start_raw_input, stop_raw_input};
