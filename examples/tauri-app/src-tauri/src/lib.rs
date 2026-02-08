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
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
