//! DexPet: Dex the folder as a desktop companion.
//!
//! Phase 0: the pet shell. The pet window is click-through everywhere except
//! Dex's own box: a ~30 Hz cursor poll toggles interactivity and feeds the eye
//! tracking, and stops while the pet is hidden. The tray owns the lifetime -
//! closing a window hides it, only tray Quit exits. The pet's position persists
//! in the app config dir. No filesystem, network or shell permission is granted
//! to the pages.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Mutex;
use std::time::Duration;
use tauri::{Emitter, Manager};

#[cfg(windows)]
mod cursor {
    #[repr(C)]
    pub struct POINT {
        pub x: i32,
        pub y: i32,
    }
    #[repr(C)]
    pub struct RECT {
        pub left: i32,
        pub top: i32,
        pub right: i32,
        pub bottom: i32,
    }
    #[link(name = "user32")]
    extern "system" {
        pub fn GetCursorPos(point: *mut POINT) -> i32;
        pub fn GetWindowRect(hwnd: isize, rect: *mut RECT) -> i32;
        pub fn IsWindowVisible(hwnd: isize) -> i32;
        pub fn GetDpiForWindow(hwnd: isize) -> u32;
    }
}

#[derive(Default)]
struct HitBox {
    x: f64,
    y: f64,
    w: f64,
    h: f64,
    set: bool,
}

#[derive(Default)]
struct PetState {
    hitbox: Mutex<HitBox>,
}

#[tauri::command]
fn set_pet_hitbox(state: tauri::State<PetState>, x: f64, y: f64, w: f64, h: f64) {
    *state.hitbox.lock().unwrap() = HitBox { x, y, w, h, set: true };
}

#[tauri::command]
fn save_pet_position(app: tauri::AppHandle, x: i32, y: i32) -> Result<(), String> {
    let dir = app.path().app_config_dir().map_err(|e| e.to_string())?;
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    std::fs::write(dir.join("pet-position.json"), format!("{{\"x\":{x},\"y\":{y}}}"))
        .map_err(|e| e.to_string())
}

fn restore_pet_position(app: &tauri::AppHandle) {
    let Ok(dir) = app.path().app_config_dir() else { return };
    let Ok(text) = std::fs::read_to_string(dir.join("pet-position.json")) else { return };
    let Ok(v) = serde_json::from_str::<serde_json::Value>(&text) else { return };
    let (Some(x), Some(y)) = (v["x"].as_i64(), v["y"].as_i64()) else { return };
    if let Some(win) = app.get_webview_window("pet") {
        let _ = win.set_position(tauri::PhysicalPosition::new(x as i32, y as i32));
    }
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(win) = app.get_webview_window("pet") {
                let _ = win.show();
                let _ = win.set_focus();
            }
        }))
        .manage(PetState::default())
        .invoke_handler(tauri::generate_handler![set_pet_hitbox, save_pet_position])
        .setup(|app| {
            let handle = app.handle().clone();
            restore_pet_position(&handle);

            let pet_win = app.get_webview_window("pet");
            let pet_hwnd = pet_win
                .as_ref()
                .and_then(|w| w.hwnd().ok())
                .map(|h| h.0 as isize)
                .unwrap_or(0);
            if let Some(win) = &pet_win {
                let _ = win.set_ignore_cursor_events(true);
            }

            use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
            use tauri::tray::TrayIconBuilder;
            let show = MenuItem::with_id(app, "show", "Show / hide Dex", true, None::<&str>)?;
            let pause = MenuItem::with_id(app, "pause", "Pause tricks", true, None::<&str>)?;
            let settings = MenuItem::with_id(app, "settings", "Settings", true, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "Quit DexPet", true, None::<&str>)?;
            let menu = Menu::with_items(
                app,
                &[&show, &pause, &settings, &PredefinedMenuItem::separator(app)?, &quit],
            )?;
            TrayIconBuilder::with_id("dex-tray")
                .icon(app.default_window_icon().expect("bundle icon").clone())
                .menu(&menu)
                .tooltip("DexPet")
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "quit" => app.exit(0),
                    "show" => {
                        if let Some(win) = app.get_webview_window("pet") {
                            if win.is_visible().unwrap_or(false) {
                                let _ = win.hide();
                            } else {
                                let _ = win.show();
                                let _ = win.set_focus();
                            }
                        }
                    }
                    "settings" => {
                        if let Some(win) = app.get_webview_window("panel") {
                            let _ = win.show();
                            let _ = win.set_focus();
                        }
                    }
                    "pause" => {
                        // Trick toggles arrive with the tricks (Phase 2+).
                    }
                    _ => {}
                })
                .build(app)?;

            std::thread::spawn(move || {
                let Some(win) = pet_win else { return };
                let hwnd = pet_hwnd;
                let mut interactive = false;
                let mut last_visible = true;
                let mut last_emit = (f64::NAN, f64::NAN);
                loop {
                    std::thread::sleep(Duration::from_millis(33));
                    // Direct Win32 reads: no per-tick Tauri event-loop round-trips,
                    // which alone cost ~2% of a core at 30 Hz.
                    let visible = unsafe { cursor::IsWindowVisible(hwnd) } != 0;
                    if visible != last_visible {
                        // Hiding the OS window does not tell WebView2; without this the
                        // renderer stays awake (and visibilityState stays "visible").
                        #[cfg(windows)]
                        let _ = win.with_webview(move |w| unsafe {
                            let _ = w.controller().SetIsVisible(visible);
                        });
                        let _ = win.emit("pet-visible", visible);
                        last_visible = visible;
                    }
                    if !visible {
                        if interactive {
                            let _ = win.set_ignore_cursor_events(true);
                            interactive = false;
                        }
                        continue;
                    }
                    let mut pt = cursor::POINT { x: 0, y: 0 };
                    if unsafe { cursor::GetCursorPos(&mut pt) } == 0 { continue; }
                    let mut rc = cursor::RECT { left: 0, top: 0, right: 0, bottom: 0 };
                    if unsafe { cursor::GetWindowRect(hwnd, &mut rc) } == 0 { continue; }
                    let dpi = unsafe { cursor::GetDpiForWindow(hwnd) } as f64;
                    let scale = if dpi > 0.0 { dpi / 96.0 } else { 1.0 };
                    let (ww, wh) = ((rc.right - rc.left) as f64, (rc.bottom - rc.top) as f64);
                    let state = handle.state::<PetState>();
                    let hb = state.hitbox.lock().unwrap();
                    let (hx, hy, hw, hh) = if hb.set {
                        (hb.x * scale, hb.y * scale, hb.w * scale, hb.h * scale)
                    } else {
                        (0.0, 0.0, ww, wh)
                    };
                    drop(hb);
                    let margin = 6.0 * scale;
                    let (cx, cy) = (pt.x as f64, pt.y as f64);
                    let (wx, wy) = (rc.left as f64, rc.top as f64);
                    let inside = cx >= wx + hx - margin
                        && cx <= wx + hx + hw + margin
                        && cy >= wy + hy - margin
                        && cy <= wy + hy + hh + margin;
                    if inside != interactive {
                        let _ = win.set_ignore_cursor_events(!inside);
                        interactive = inside;
                    }
                    let reach = 400.0 * scale;
                    let near = cx >= wx - reach
                        && cx <= wx + ww + reach
                        && cy >= wy - reach
                        && cy <= wy + wh + reach;
                    if near && (cx != last_emit.0 || cy != last_emit.1) {
                        let _ = win.emit("pet-cursor", [cx - wx, cy - wy, ww, wh]);
                        last_emit = (cx, cy);
                    }
                }
            });
            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running DexPet");
}