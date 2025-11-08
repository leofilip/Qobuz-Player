#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::{
    Manager, WindowEvent,
    menu::{Menu, MenuItem},
    tray::{MouseButton, TrayIconBuilder, TrayIconEvent},
};
use raw_window_handle::HasWindowHandle;
use std::sync::Mutex;

mod thumbar;
mod settings;
mod window_manager;

pub struct AppState {
    settings: Mutex<settings::Settings>,
}

#[tauri::command]
fn native_add_thumb_buttons() {
    thumbar::add_thumb_buttons();
}

#[tauri::command]
fn native_remove_thumb_buttons() {
    thumbar::remove_thumb_buttons();
}

#[tauri::command]
fn get_settings(state: tauri::State<AppState>) -> Result<settings::Settings, String> {
    let settings = state.settings.lock()
        .map_err(|e| format!("Failed to lock settings: {}", e))?;
    Ok(settings.clone())
}

#[tauri::command]
fn save_settings(settings: settings::Settings, state: tauri::State<AppState>) -> Result<(), String> {
    settings.save()?;
    
    let mut app_settings = state.settings.lock()
        .map_err(|e| format!("Failed to lock settings: {}", e))?;
    
    if settings.launch_on_login {
        settings::autostart::enable(&settings.launch_mode)?;
    } else {
        let _ = settings::autostart::disable();
    }
    
    *app_settings = settings;
    Ok(())
}

#[tauri::command]
fn close_settings_window(app: tauri::AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("settings") {
        window.close().map_err(|e| format!("Failed to close settings window: {}", e))?;
    }
    Ok(())
}

fn main() {
    let app_settings = settings::Settings::load();
    
    tauri::Builder::default()
        .manage(AppState {
            settings: Mutex::new(app_settings),
        })
        .invoke_handler(tauri::generate_handler![
            native_add_thumb_buttons, 
            native_remove_thumb_buttons,
            get_settings,
            save_settings,
            close_settings_window
        ])
        .register_uri_scheme_protocol("settings", |_app, _request| {
            let settings_html = include_str!("../settings.html");
            tauri::http::Response::builder()
                .header("Content-Type", "text/html; charset=utf-8")
                .header("Content-Security-Policy", "default-src 'self' 'unsafe-inline' settings:;")
                .body(settings_html.as_bytes().to_vec())
                .expect("Failed to build HTTP response")
        })
        .on_page_load(|_window, _payload| {
            thumbar::add_thumb_buttons();
        })
    .plugin(tauri_plugin_media::init())
    .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
        if let Some(window) = app.get_webview_window("main") {
            let _ = window.unminimize();
            let _ = window.show();
            let _ = window.set_focus();
        }
    }))
        .setup(|app| {
            #[cfg(target_os = "windows")]
            {
                use windows::core::PCWSTR;
                use windows::Win32::UI::Shell::SetCurrentProcessExplicitAppUserModelID;
                
                #[cfg(debug_assertions)]
                let app_id = "com.leo.qobuz-player.dev";
                #[cfg(not(debug_assertions))]
                let app_id = "com.leo.qobuz-player";
                
                let id = app_id.encode_utf16().chain(std::iter::once(0)).collect::<Vec<u16>>();
                let pcw = PCWSTR(id.as_ptr());
                let _ = unsafe { SetCurrentProcessExplicitAppUserModelID(pcw) };
            }
            
            let show = MenuItem::with_id(app, "show", "Show", true, None::<&str>)?;
            let settings = MenuItem::with_id(app, "settings", "Settings", true, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show, &settings, &quit])?;

            TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| match event.id().as_ref() {
                    "quit" => {
                        thumbar::cleanup_thumbar();
                        window_manager::remove_minimize_hook();
                        app.exit(0);
                    }
                    "show" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.unminimize();
                            let _ = window.show();
                            let _ = window.set_focus();
                            
                            #[cfg(target_os = "windows")]
                            if let Ok(wh) = window.window_handle()
                                && let raw_window_handle::RawWindowHandle::Win32(h) = wh.into() {
                                    thumbar::set_stored_hwnd(h);
                                    thumbar::add_thumb_buttons();
                                }
                        }
                    }
                    "settings" => {
                        if let Some(window) = app.get_webview_window("settings") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        } else {
                            use tauri::WebviewWindowBuilder;
                            
                            match WebviewWindowBuilder::new(app, "settings", tauri::WebviewUrl::CustomProtocol("settings://localhost/".parse().unwrap()))
                                .title("Settings")
                                .inner_size(600.0, 700.0)
                                .resizable(false)
                                .maximizable(false)
                                .minimizable(false)
                                .center()
                                .build() {
                                Ok(_) => {},
                                Err(e) => eprintln!("Failed to create settings window: {}", e),
                            }
                        }
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| if let TrayIconEvent::DoubleClick {
                        button: MouseButton::Left,
                        ..
                    } = event {
                    let app = tray.app_handle();
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.unminimize();
                        let _ = window.show();
                        let _ = window.set_focus();
                        
                        #[cfg(target_os = "windows")]
                        if let Ok(wh) = window.window_handle()
                            && let raw_window_handle::RawWindowHandle::Win32(h) = wh.into() {
                                thumbar::set_stored_hwnd(h);
                                thumbar::add_thumb_buttons();
                            }
                    }
                })
                .build(app)?;
            
            thumbar::init_thumbar(app, "main");
            window_manager::init_window_manager(app);
            
            if let Some(window) = app.get_webview_window("main")
                && let Ok(wh) = window.window_handle()
                    && let raw_window_handle::RawWindowHandle::Win32(h) = wh.into() {
                        thumbar::set_stored_hwnd(h);
                        window_manager::set_main_window_hwnd(h.hwnd.get());
                        window_manager::install_minimize_hook();
                    }
            
            Ok(())
        })
        .on_window_event(|window, event| {
            if window.label() == "main"
                && let WindowEvent::CloseRequested { api, .. } = event {
                    let app = window.app_handle();
                    let state = app.state::<AppState>();
                    
                    let close_to_tray = if let Ok(settings) = state.settings.lock() {
                        settings.close_to_tray
                    } else {
                        true
                    };
                    
                    if close_to_tray {
                        api.prevent_close();
                        let _ = window.hide();
                    } else {
                        thumbar::cleanup_thumbar();
                        window_manager::remove_minimize_hook();
                    }
                }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
