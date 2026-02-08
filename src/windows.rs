use tauri::{AppHandle, command, Emitter, Manager};
use tauri::Wry;
use serde_json::json;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use once_cell::sync::Lazy;
use windows::{
    Win32::Foundation::*,
    Win32::UI::Input::*,
    Win32::UI::WindowsAndMessaging::*,
};

use crate::{DeviceEvent, DeviceEventKind};

static IS_LISTENING: AtomicBool = AtomicBool::new(false);

#[derive(Clone, Copy, Debug)]
enum TrackingMode {
    Relative,
    Absolute,
}

struct PluginState {
    app: Option<AppHandle<Wry>>,
    hook: Option<HHOOK>,
    tracking_mode: TrackingMode,
}

static STATE: Lazy<Mutex<PluginState>> = Lazy::new(|| {
    Mutex::new(PluginState {
        app: None,
        hook: None,
        tracking_mode: TrackingMode::Relative,
    })
});

#[command]
pub fn start_raw_input(app: AppHandle<Wry>) -> std::result::Result<(), String> {
    if IS_LISTENING.load(Ordering::SeqCst) {
        return Ok(());
    }

    IS_LISTENING.store(true, Ordering::SeqCst);

    let mode = {
        let mut s = STATE.lock().map_err(|e| e.to_string())?;
        s.app = Some(app.clone());
        s.tracking_mode
    };

    // Only install the hook in relative mode; absolute mode uses polling
    if matches!(mode, TrackingMode::Relative) {
        unsafe { install_hook(&app).map_err(|e| format!("install_hook failed: {:?}", e))?; }
    }

    Ok(())
}

#[command]
pub fn get_mouse_position() -> std::result::Result<(i32, i32), String> {
    let mut pt = POINT { x: 0, y: 0 };
    unsafe {
        if !GetCursorPos(&mut pt).as_bool() {
            return Err("GetCursorPos failed".to_string());
        }
    }
    Ok((pt.x, pt.y))
}

#[command]
pub fn set_tracking_mode(mode: String) -> std::result::Result<(), String> {
    let tracking_mode = match mode.as_str() {
        "relative" => TrackingMode::Relative,
        "absolute" => TrackingMode::Absolute,
        _ => return Err("Invalid tracking mode. Use 'relative' or 'absolute'".to_string()),
    };

    let mut s = STATE.lock().map_err(|e| e.to_string())?;
    let old_mode = s.tracking_mode;
    s.tracking_mode = tracking_mode;
    eprintln!("Tracking mode set to: {:?}", tracking_mode);

    // Manage hook lifecycle when switching modes while listening
    if IS_LISTENING.load(Ordering::SeqCst) {
        match (old_mode, tracking_mode) {
            (TrackingMode::Relative, TrackingMode::Absolute) => {
                unsafe { uninstall_hook(&mut s); }
            },
            (TrackingMode::Absolute, TrackingMode::Relative) => {
                let app = s.app.clone();
                drop(s);
                if let Some(app_handle) = app {
                    unsafe { install_hook(&app_handle).map_err(|e| format!("Failed to install hook: {:?}", e))?; }
                }
            },
            _ => {},
        }
    }

    Ok(())
}

#[command]
pub fn get_tracking_mode() -> std::result::Result<String, String> {
    let s = STATE.lock().map_err(|e| e.to_string())?;
    let mode_str = match s.tracking_mode {
        TrackingMode::Relative => "relative",
        TrackingMode::Absolute => "absolute",
    };
    Ok(mode_str.to_string())
}

#[command]
pub fn stop_raw_input() -> std::result::Result<(), String> {
    if !IS_LISTENING.load(Ordering::SeqCst) {
        return Ok(());
    }

    IS_LISTENING.store(false, Ordering::SeqCst);

    let mut s = STATE.lock().map_err(|e| e.to_string())?;
    unsafe { uninstall_hook(&mut s); }
    s.app = None;

    Ok(())
}

unsafe extern "system" fn window_hook_proc(
    code: i32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    if code >= 0 {
        let msg = &*(lparam.0 as *const MSG);
        if msg.message == WM_INPUT && IS_LISTENING.load(Ordering::SeqCst) {
            let hrawinput = HRAWINPUT(msg.lParam.0 as isize);

            let (app_opt, mode) = {
                let s = STATE.lock().unwrap();
                (s.app.clone(), s.tracking_mode)
            };

            // Only emit from the hook in relative mode; absolute mode uses polling
            if matches!(mode, TrackingMode::Relative) {
                if let Some(app) = app_opt {
                    handle_raw_input(hrawinput, &app);
                }
            }
        }
    }

    CallNextHookEx(None, code, wparam, lparam)
}

unsafe fn install_hook(app: &AppHandle<Wry>) -> std::result::Result<(), windows::core::Error> {
    let window = match app.get_webview_window("main") {
        Some(w) => w,
        None => {
            eprintln!("Failed to get main window");
            return Ok(());
        },
    };

    let raw_handle = match window.hwnd() {
        Ok(h) => h,
        Err(_) => {
            eprintln!("Failed to get window handle");
            return Ok(());
        },
    };

    let hwnd = HWND(raw_handle.0 as isize);
    eprintln!("Window handle: {:?}", hwnd);

    // Register for mouse raw input
    let device = RAWINPUTDEVICE {
        usUsagePage: 0x01,
        usUsage: 0x02,
        dwFlags: RIDEV_INPUTSINK,
        hwndTarget: hwnd,
    };

    eprintln!("Registering raw input device");
    let _ = RegisterRawInputDevices(&[device], std::mem::size_of::<RAWINPUTDEVICE>() as u32).ok();

    // Install Windows hook
    let thread_id = GetWindowThreadProcessId(hwnd, None);
    eprintln!("Thread ID: {}", thread_id);
    let hhook = SetWindowsHookExW(WH_GETMESSAGE, Some(window_hook_proc), None, thread_id)?;
    eprintln!("Hook installed: {:?}", hhook);

    let mut s = STATE.lock().unwrap();
    s.hook = Some(hhook);

    Ok(())
}

unsafe fn uninstall_hook(s: &mut PluginState) {
    if let Some(hhook) = s.hook.take() {
        let _ = UnhookWindowsHookEx(hhook).ok();
        eprintln!("Hook uninstalled: {:?}", hhook);
    }
}

unsafe fn handle_raw_input(hrawinput: HRAWINPUT, app: &AppHandle<Wry>) {
    let mut size = 0u32;

    let _res = GetRawInputData(
        hrawinput,
        RID_INPUT,
        None,
        &mut size,
        std::mem::size_of::<RAWINPUTHEADER>() as u32,
    );

    if size == 0 {
        return;
    }

    let mut buffer = vec![0u8; size as usize];
    let res2 = GetRawInputData(
        hrawinput,
        RID_INPUT,
        Some(buffer.as_mut_ptr() as *mut _),
        &mut size,
        std::mem::size_of::<RAWINPUTHEADER>() as u32,
    );

    if res2 == 0 || size == 0 {
        return;
    }

    let raw = &*(buffer.as_ptr() as *const RAWINPUT);

    if raw.header.dwType == RIM_TYPEMOUSE.0 {
        let mouse = unsafe { &raw.data.mouse };
        let dx = mouse.lLastX;
        let dy = mouse.lLastY;

        let event = DeviceEvent {
            kind: DeviceEventKind::MouseMove,
            value: json!({ "x": dx, "y": dy }),
        };

        let _ = app.emit("device-changed", event);
    }
}
