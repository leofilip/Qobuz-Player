#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::{
    Manager, WindowEvent,
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
};
use raw_window_handle::HasWindowHandle;
// no direct Emitter usage here; native thumbar sends media keys directly

mod thumbar;

// Note: we removed the `native_set_playing` command — the native thumbar
// uses static icons now and the renderer/plugin is authoritative for
// playback state. Renderer should listen for `thumbar-*` events and act.

#[tauri::command]
fn native_add_thumb_buttons() {
    // Call into the native module to (re)create the thumbbar buttons.
    thumbar::add_thumb_buttons();
}

#[tauri::command]
fn native_remove_thumb_buttons() {
    // Remove the subclass and cleanup native icons.
    thumbar::remove_thumb_buttons();
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![native_add_thumb_buttons, native_remove_thumb_buttons])
    .on_page_load(|_window, _payload| {
            // When the page finishes loading, create the native thumbbar
            // buttons for the main window so they appear automatically.
            // The HWND is stored during `setup` so the native loader will
            // be able to call ThumbBarAddButtons.
            let _ = thumbar::add_thumb_buttons();
        })
    .plugin(tauri_plugin_media::init())
    .plugin(tauri_plugin_single_instance::init(|_app, _args, _cwd| {}))
        .setup(|app| {
            // On Windows, set an explicit AppUserModelID so Explorer
            // associates this process with the application identity and
            // (when possible) updates the taskbar/pinned icon. This helps
            // when icons change during development.
            #[cfg(target_os = "windows")]
            {
                use windows::core::PCWSTR;
                use windows::Win32::UI::Shell::SetCurrentProcessExplicitAppUserModelID;
                
                // Use different AppUserModelID for dev vs release builds so Windows
                // doesn't try to use the icon from the installed app location when
                // running dev builds. This prevents the "icon not found" issue where
                // Explorer looks for C:\Program Files\qobuz-player\qobuz-player.exe
                // which doesn't exist during development.
                #[cfg(debug_assertions)]
                let app_id = "com.leo.qobuz-player.dev";
                #[cfg(not(debug_assertions))]
                let app_id = "com.leo.qobuz-player";
                
                let id = app_id.encode_utf16().chain(std::iter::once(0)).collect::<Vec<u16>>();
                let pcw = PCWSTR(id.as_ptr());
                let _ = unsafe { SetCurrentProcessExplicitAppUserModelID(pcw) };
            }
            // Build menu items (only production items). The dev helper was
            // intentionally removed so development helpers don't clutter the
            // codebase — continue developing the thumbar feature instead.
            let show = MenuItem::with_id(app, "show", "Show", true, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show, &quit])?;

            // Create tray icon
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
                            // Toggle: if visible -> hide to tray; otherwise show/restore.
                            match window.is_visible() {
                                Ok(true) => {
                                    let _ = window.hide();
                                }
                                _ => {
                                    let _ = window.unminimize();
                                    let _ = window.show();
                                    let _ = window.set_focus();
                                    // Reinitialize thumbar for this window in case the
                                    // HWND changed during hide/minimize.
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
            // Initialize the thumbar scaffolding (Windows-only integration point)
            // Pass the `App` reference so the thumbar module can locate the
            // main window and emit events via the app handle.
            thumbar::init_thumbar(app, "main");
            // Attempt to store the main webview HWND now so native code can
            // call ThumbBarAddButtons. This mirrors the logic used when the
            // tray toggles the window and ensures a stored HWND early.
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
            // Thumbbar clicks are handled by the native module which now
            // converts them into system media key events. Media state and
            // SMTC integration are handled by `tauri-plugin-media`.
            // We'll add the thumbbar buttons once the page loads so buttons
            // appear automatically. Frontend can still call the commands
            // `native_set_playing`, `native_add_thumb_buttons` and
            // `native_remove_thumb_buttons` as needed to update state.
            // Use `on_page_load` to trigger native creation when the main
            // webview finishes loading.
            // Note: we register the on_page_load handler on the Builder below.
            // Thumbar scaffolding initialized; frontend can be wired to events
            // once native thumbar implementation is present.
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
