use tauri::Manager;
use crate::interfaces::WindowCommandDispatcher;
use crate::{AppState, theme};
#[cfg(windows)]
use crate::{thumbar, window_manager};
#[cfg(windows)]
use raw_window_handle::HasWindowHandle;
use std::sync::atomic::Ordering;

/// Concrete dispatcher for Win32 window-procedure commands.
pub struct AppCommandDispatcher;

impl WindowCommandDispatcher for AppCommandDispatcher {
    fn handle_minimize(&self, app: &tauri::AppHandle, minimize_to_tray: bool) -> bool {
        if let Some(window) = app.get_webview_window("main") {
            if minimize_to_tray {
                let _ = window.hide();
            } else {
                let _ = window.minimize();
            }
        }
        if minimize_to_tray {
            app.state::<AppState>().tray_hidden.store(true, Ordering::SeqCst);
        }
        minimize_to_tray
    }

    fn handle_show(&self, app: &tauri::AppHandle) {
        if let Some(window) = app.get_webview_window("main") {
            let _ = window.unminimize();
            let _ = window.show();
            let _ = window.set_focus();
            app.state::<AppState>().tray_hidden.store(false, Ordering::SeqCst);
            #[cfg(windows)]
            if let Ok(wh) = window.window_handle()
                && let raw_window_handle::RawWindowHandle::Win32(h) = wh.into()
            {
                thumbar::set_stored_hwnd(h);
                thumbar::add_thumb_buttons();
            }
        }
    }

    fn handle_open_settings(&self, app: &tauri::AppHandle) {
        crate::open_settings_window_pub(app.clone());
    }

    fn handle_quit(&self, app: &tauri::AppHandle) {
        #[cfg(windows)]
        {
            thumbar::cleanup_thumbar();
            if let Some(window) = app.get_webview_window("main") {
                if let Ok(wh) = window.window_handle()
                    && let raw_window_handle::RawWindowHandle::Win32(h) = wh.into()
                {
                    window_manager::remove_minimize_hook(h.hwnd.get());
                }
            }
        }
        std::process::exit(0);
    }

    fn handle_prev_track(&self, app: &tauri::AppHandle) {
        click_selector(app, "(function(){ let selectors = ['button[aria-label*=\"revious\"]', 'button[aria-label*=\"Previous\"]', 'button[aria-label*=\"PREVIOUS\"]', 'button[title*=\"revious\"]', 'button[title*=\"Previous\"]', '.pct-player-previous', '.player__action-previous', 'button[class*=\"previous\"]', 'button[class*=\"prev\"]', 'button[class*=\"back\"]', '[data-testid*=\"previous\"]', '[data-testid*=\"prev\"]', 'button.pct-player-previous', 'span.pct-player-previous']; for(let s of selectors) { let el = document.querySelector(s); if(el) { el.click(); return; } } })()");
    }

    fn handle_play_pause(&self, app: &tauri::AppHandle) {
        click_selector(app, "(function(){ let m = document.querySelector('audio, video'); if(m) { if(m.paused) m.play(); else m.pause(); } else { document.querySelector('button[aria-label*=\"lay\"], button[aria-label*=\"ause\"], .play-button, .pause-button, .pct-player-play, .pct-player-pause')?.click(); } })()");
    }

    fn handle_next_track(&self, app: &tauri::AppHandle) {
        click_selector(app, "(function(){ let selectors = ['button[aria-label*=\"ext\"]', 'button[aria-label*=\"Next\"]', '.pct-player-next', 'button[class*=\"next\"]', '[data-testid*=\"next\"]']; for(let s of selectors) { let el = document.querySelector(s); if(el) { el.click(); return; } } })()");
    }
}

fn click_selector(app: &tauri::AppHandle, js: &str) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.eval(js);
    }
}

#[cfg(windows)]
#[tauri::command]
pub fn native_add_thumb_buttons() {
    thumbar::add_thumb_buttons();
}

#[cfg(not(windows))]
#[tauri::command]
pub fn native_add_thumb_buttons() {
}

#[cfg(windows)]
#[tauri::command]
pub fn native_remove_thumb_buttons() {
    thumbar::remove_thumb_buttons();
}

#[cfg(not(windows))]
#[tauri::command]
pub fn native_remove_thumb_buttons() {
}

#[tauri::command]
pub fn get_settings(state: tauri::State<AppState>) -> Result<crate::settings::Settings, String> {
    let settings = state
        .settings
        .lock()
        .map_err(|e| format!("Failed to lock settings: {}", e))?;
    Ok(settings.clone())
}

#[tauri::command]
pub fn minimize_window(
    app: tauri::AppHandle,
    state: tauri::State<AppState>,
) -> Result<(), String> {
    let minimize_to_tray = {
        let settings = state
            .settings
            .lock()
            .map_err(|e| format!("Failed to lock settings: {}", e))?;
        settings.minimize_to_tray
    };

    if let Some(window) = app.get_webview_window("main") {
        if minimize_to_tray {
            window
                .hide()
                .map_err(|e| format!("Failed to hide window: {}", e))?;
        } else {
            window
                .minimize()
                .map_err(|e| format!("Failed to minimize window: {}", e))?;
        }
    }

    Ok(())
}

#[tauri::command]
pub fn save_settings(
    settings: crate::settings::Settings,
    state: tauri::State<AppState>,
) -> Result<(), String> {
    settings.save()?;

    let mut app_settings = state
        .settings
        .lock()
        .map_err(|e| format!("Failed to lock settings: {}", e))?;

    #[cfg(windows)]
    if settings.launch_on_login {
        crate::settings::autostart::enable(&settings.launch_mode)?;
    } else {
        let _ = crate::settings::autostart::disable();
    }

    *app_settings = settings.clone();

    Ok(())
}

#[tauri::command]
pub fn apply_theme_from_string(app: tauri::AppHandle, theme: String) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        let state = app.state::<AppState>();
        theme::apply_theme(&window, &theme, state.theme_registry.as_ref())?;
    }
    Ok(())
}

#[tauri::command]
pub fn close_settings_window(app: tauri::AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        let script = include_str!("../inject/remove_overlay.js");
        window
            .eval(script)
            .map_err(|e| format!("Failed to remove settings overlay: {}", e))?;
    }
    Ok(())
}

pub(crate) fn open_settings_window_pub(app: tauri::AppHandle) {
    let _ = open_settings_window(app);
}

#[tauri::command]
pub fn open_settings_window(app: tauri::AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.set_focus();

        let settings_html = include_str!("../settings.html");

        let body_start = settings_html.find("<body>").unwrap_or(0) + 6;
        let body_end = settings_html
            .find("</body>")
            .unwrap_or(settings_html.len());
        let body_content = &settings_html[body_start..body_end];

        let style_start = settings_html.find("<style>").unwrap_or(0);
        let style_end = settings_html.find("</style>").unwrap_or(0) + 8;
        let styles = if style_start > 0 && style_end > 8 {
            &settings_html[style_start..style_end]
        } else {
            ""
        };

        let body_escaped = body_content.replace("`", "\\`").replace("${", "\\${");
        let styles_escaped = styles.replace("`", "\\`").replace("${", "\\${");
        let template = include_str!("../inject/settings_overlay.js");
        let js_code = template
            .replacen("{}", &styles_escaped, 1)
            .replacen("{}", &body_escaped, 1);

        window
            .eval(&js_code)
            .map_err(|e| format!("Failed to inject settings overlay: {}", e))?;

        Ok(())
    } else {
        Err("Main window not found".to_string())
    }
}
