#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::{
    Manager, WindowEvent,
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
};
use raw_window_handle::HasWindowHandle;

mod thumbar;

#[tauri::command]
fn native_add_thumb_buttons() {
    thumbar::add_thumb_buttons();
}

#[tauri::command]
fn native_remove_thumb_buttons() {
    thumbar::remove_thumb_buttons();
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![native_add_thumb_buttons, native_remove_thumb_buttons])
        .on_page_load(|_window, _payload| {
            let _ = thumbar::add_thumb_buttons();
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
            let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show, &quit])?;

            TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| match event.id().as_ref() {
                    "quit" => {
                        thumbar::cleanup_thumbar();
                        app.exit(0);
                    }
                    "show" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| match event {
                    TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } => {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            match window.is_visible() {
                                Ok(true) => {
                                    let _ = window.hide();
                                }
                                _ => {
                                    let _ = window.unminimize();
                                    let _ = window.show();
                                    let _ = window.set_focus();
                                    
                                    #[cfg(target_os = "windows")]
                                    if let Ok(wh) = window.window_handle() {
                                        match wh.into() {
                                            raw_window_handle::RawWindowHandle::Win32(h) => {
                                                thumbar::set_stored_hwnd(h);
                                                thumbar::add_thumb_buttons();
                                            }
                                            _ => {}
                                        }
                                    }
                                }
                            }
                        }
                    }
                    _ => {}
                })
                .build(app)?;
            
            thumbar::init_thumbar(app, "main");
            
            if let Some(window) = app.get_webview_window("main") {
                if let Ok(wh) = window.window_handle() {
                    match wh.into() {
                        raw_window_handle::RawWindowHandle::Win32(h) => {
                            thumbar::set_stored_hwnd(h);
                        }
                        _ => {}
                    }
                }
            }
            
            Ok(())
        })
        .on_window_event(|app, event| {
            match event {
                // Hide to tray instead of closing
                WindowEvent::CloseRequested { api, .. } => {
                    api.prevent_close();
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.hide();
                    }
                }
                _ => {}
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
