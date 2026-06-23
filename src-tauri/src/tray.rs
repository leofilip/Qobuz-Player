//! System tray icon, left-click window toggle, right-click context menu,
//! and double-click restore behavior.
//!
//! On Windows the context menu is a raw Win32 popup (not Tauri's menu system)
//! to avoid the OS automatically showing the menu on both left and right clicks.

#[cfg(windows)]
use raw_window_handle::HasWindowHandle;
use std::sync::atomic::Ordering;
use tauri::{
    Manager,
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
};

use crate::AppState;

pub fn build_tray(app: &tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    TrayIconBuilder::new()
        .icon(app.default_window_icon().unwrap().clone())
        .on_tray_icon_event(|tray, event| match event {
            TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Down,
                ..
            } => {
                handle_tray_left_click(tray.app_handle());
            }
            TrayIconEvent::Click {
                button: MouseButton::Right,
                button_state: MouseButtonState::Down,
                ..
            } => {
                show_tray_context_menu(tray.app_handle());
            }
            TrayIconEvent::DoubleClick {
                button: MouseButton::Left,
                ..
            } => {
                handle_tray_double_click(tray.app_handle());
            }
            _ => {}
        })
        .build(app)?;
    Ok(())
}

fn restore_window(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
        app.state::<AppState>()
            .tray_hidden
            .store(false, Ordering::SeqCst);

        #[cfg(windows)]
        if let Ok(wh) = window.window_handle()
            && let raw_window_handle::RawWindowHandle::Win32(h) = wh.into()
        {
            crate::thumbar::set_stored_hwnd(h);
            crate::thumbar::add_thumb_buttons();
        }
    }
}

fn hide_window(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.hide();
        app.state::<AppState>()
            .tray_hidden
            .store(true, Ordering::SeqCst);
    }
}

fn handle_tray_left_click(app: &tauri::AppHandle) {
    if app.state::<AppState>().tray_hidden.load(Ordering::SeqCst) {
        restore_window(app);
    } else {
        hide_window(app);
    }
}

fn handle_tray_double_click(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let script = include_str!("../inject/remove_overlay.js");
        let _ = window.eval(script);
    }
    restore_window(app);
}

#[cfg(windows)]
fn show_tray_context_menu(app: &tauri::AppHandle) {
    unsafe {
        use windows::Win32::Foundation::{HWND, POINT};
        use windows::Win32::UI::WindowsAndMessaging::{
            AppendMenuW, CreatePopupMenu, DestroyMenu, GetCursorPos, TrackPopupMenu,
            MENU_ITEM_FLAGS, TRACK_POPUP_MENU_FLAGS,
        };
        use windows::core::PWSTR;

        let Ok(hmenu) = CreatePopupMenu() else {
            return;
        };

        macro_rules! add_item {
            ($id:expr, $text:expr) => {
                let text: Vec<u16> = concat!($text, "\0").encode_utf16().collect();
                let _ = AppendMenuW(
                    hmenu,
                    MENU_ITEM_FLAGS(0u32),
                    $id,
                    PWSTR(text.as_ptr() as *mut _),
                );
            };
        }

        add_item!(1001, "Show");
        add_item!(1002, "Settings");
        let _ = AppendMenuW(
            hmenu,
            MENU_ITEM_FLAGS(0x800u32),
            0,
            PWSTR(std::ptr::null_mut()),
        );
        add_item!(1003, "Quit");

        let mut pos = POINT { x: 0, y: 0 };
        let _ = GetCursorPos(&mut pos);

        let hwnd = app
            .get_webview_window("main")
            .map(|w| {
                if let Ok(wh) = w.window_handle()
                    && let raw_window_handle::RawWindowHandle::Win32(h) = wh.into()
                {
                    HWND(h.hwnd.get() as *mut std::ffi::c_void)
                } else {
                    HWND(std::ptr::null_mut())
                }
            })
            .unwrap_or(HWND(std::ptr::null_mut()));

        let _ = TrackPopupMenu(
            hmenu,
            TRACK_POPUP_MENU_FLAGS::default(),
            pos.x,
            pos.y,
            None,
            hwnd,
            None,
        );

        let _ = DestroyMenu(hmenu);
    }
}

#[cfg(not(windows))]
fn show_tray_context_menu(_app: &tauri::AppHandle) {
}
