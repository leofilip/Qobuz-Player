#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::{
    Manager, WindowEvent,
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
};

mod thumbar;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|_app, _args, _cwd| {}))
        .setup(|app| {
            // Build menu items
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
                        println!("Quitting application...");
                        app.exit(0);
                    }
                    "show" => {
                        if let Some(window) = app.get_webview_window("main") {
                            window.show().unwrap();
                            window.set_focus().unwrap();
                        }
                    }
                    _ => {
                        println!("Unknown menu item {:?} clicked", event.id());
                    }
                })
                .on_tray_icon_event(|tray, event| match event {
                    TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } => {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.unminimize();
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    _ => {}
                })
                .build(app)?;
            // Initialize the thumbar scaffolding (Windows-only integration point)
            // Pass the `App` reference so the thumbar module can locate the
            // main window and emit events via the app handle.
            thumbar::init_thumbar(app, "main");
            // Thumbar scaffolding initialized; frontend can be wired to events
            // once native thumbar implementation is present.
            Ok(())
        })
        .on_window_event(|app, event| {
            // match the close request on the window event and hide instead of closing
            if let WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                if let Some(window) = app.get_webview_window("main") {
                    window.hide().unwrap();
                }
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
