//! Qobuz-Player: a Windows desktop wrapper for the Qobuz web player.
//!
//! Embeds `https://play.qobuz.com/` in a Tauri 2 WebView2 window with
//! custom titlebar, system tray integration, and Windows taskbar thumbnail
//! media controls. Windows-only Win32 interop is gated behind `#[cfg(windows)]`.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![cfg_attr(not(windows), allow(dead_code))]

use std::sync::atomic::AtomicBool;
use std::sync::Mutex;
#[cfg(windows)]
use raw_window_handle::HasWindowHandle;
use tauri::Manager;

mod commands;
mod interfaces;
mod settings;
mod theme;
#[cfg(windows)]
mod thumbar;
mod tray;
#[cfg(windows)]
mod window_manager;

pub(crate) use interfaces::AppState;
pub(crate) use commands::open_settings_window_pub;

#[cfg(windows)]
fn set_app_user_model_id() {
    use windows::core::PCWSTR;
    use windows::Win32::UI::Shell::SetCurrentProcessExplicitAppUserModelID;

    #[cfg(debug_assertions)]
    let app_id = "com.leo.qobuz-player.dev";
    #[cfg(not(debug_assertions))]
    let app_id = "com.leo.qobuz-player";

    let id = app_id
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect::<Vec<u16>>();
    let pcw = PCWSTR(id.as_ptr());
    let _ = unsafe { SetCurrentProcessExplicitAppUserModelID(pcw) };
}

fn theme_registry() -> Box<dyn interfaces::ThemeRegistry> {
    Box::new(theme::DefaultThemeRegistry)
}

fn main() {
    let app_settings = settings::Settings::load();

    let builder = tauri::Builder::default()
        .manage(AppState {
            settings: Mutex::new(app_settings),
            tray_hidden: AtomicBool::new(false),
            dispatcher: Mutex::new(Some(
                Box::new(commands::AppCommandDispatcher)
                    as Box<dyn interfaces::WindowCommandDispatcher>,
            )),
            theme_registry: theme_registry(),
        })
        .invoke_handler(tauri::generate_handler![
            commands::native_add_thumb_buttons,
            commands::native_remove_thumb_buttons,
            commands::get_settings,
            commands::save_settings,
            commands::close_settings_window,
            commands::open_settings_window,
            commands::minimize_window,
            commands::apply_theme_from_string,
        ]);

    #[cfg(windows)]
    let builder = builder.plugin(tauri_plugin_media::init());

    builder
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.unminimize();
                let _ = window.show();
                let _ = window.set_focus();
            }
        }))
        .setup(|app| {
            #[cfg(windows)]
            set_app_user_model_id();

            tray::build_tray(app)?;

            #[cfg(windows)]
            {
                thumbar::init_thumbar(app, "main");
                window_manager::init_window_manager(app);
            }

            if let Some(window) = app.get_webview_window("main") {
                let init_script = include_str!("../inject/titlebar.js");
                let _ = window.eval(init_script);

                #[cfg(windows)]
                if let Ok(wh) = window.window_handle()
                    && let raw_window_handle::RawWindowHandle::Win32(h) = wh.into()
                {
                    thumbar::set_stored_hwnd(h);
                    window_manager::install_minimize_hook(h.hwnd.get());
                }
            }

            Ok(())
        })
        .on_window_event(|window, event| {
            if window.label() == "main"
                && let tauri::WindowEvent::CloseRequested { api, .. } = event
            {
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
                    #[cfg(windows)]
                    {
                        thumbar::cleanup_thumbar();
                        if let Ok(wh) = window.window_handle()
                            && let raw_window_handle::RawWindowHandle::Win32(h) = wh.into()
                        {
                            window_manager::remove_minimize_hook(h.hwnd.get());
                        }
                    }
                }
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
