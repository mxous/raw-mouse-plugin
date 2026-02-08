use raw_mouse_plugin;
use tauri::{generate_handler, command, AppHandle};
use tauri::Wry;

#[command]
fn start_raw_input(app: AppHandle<Wry>) -> Result<(), String> {
    raw_mouse_plugin::start_raw_input(app)
}

#[command]
fn stop_raw_input() -> Result<(), String> {
    raw_mouse_plugin::stop_raw_input()
}

#[command]
fn set_tracking_mode(mode: String) -> Result<(), String> {
    raw_mouse_plugin::set_tracking_mode(mode)
}

#[command]
fn get_tracking_mode() -> Result<String, String> {
    raw_mouse_plugin::get_tracking_mode()
}

#[command]
fn get_mouse_position() -> Result<(i32, i32), String> {
    raw_mouse_plugin::get_mouse_position()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|_app| {
            println!("Raw Mouse Plugin Example starting...");
            Ok(())
        })
        .invoke_handler(generate_handler![
            start_raw_input,
            stop_raw_input,
            set_tracking_mode,
            get_tracking_mode,
            get_mouse_position
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
