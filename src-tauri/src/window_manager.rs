use std::sync::OnceLock;
use tauri::Manager;
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{
    CallWindowProcW, DefWindowProcW, SetWindowLongPtrW, GWLP_WNDPROC,
    WM_SYSCOMMAND, WM_COMMAND, SC_MINIMIZE,
};

use crate::interfaces::{AppState, WindowCommandDispatcher};

// Thumbar button IDs — must match ThumbButtonConfig
const THB_BACK: u16 = 100;
const THB_PLAY: u16 = 101;
const THB_NEXT: u16 = 102;

static PREV_WNDPROC: OnceLock<isize> = OnceLock::new();
static APP_HANDLE: OnceLock<tauri::AppHandle> = OnceLock::new();

pub fn init_window_manager(app: &tauri::App) {
    let _ = APP_HANDLE.set(app.handle().clone());
}

fn with_dispatcher<F, R>(f: F) -> Option<R>
where
    F: FnOnce(&dyn WindowCommandDispatcher, &tauri::AppHandle) -> R,
{
    let app = APP_HANDLE.get()?;
    let state = app.state::<AppState>();
    if let Ok(guard) = state.dispatcher.lock() {
        if let Some(ref dispatcher) = *guard {
            return Some(f(dispatcher.as_ref(), app));
        }
    }
    None
}

pub fn install_minimize_hook(hwnd_raw: isize) {
    if hwnd_raw == 0 {
        return;
    }

    let hwnd = HWND(hwnd_raw as *mut std::ffi::c_void);

    unsafe extern "system" fn wndproc(
        hwnd: HWND,
        msg: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        if msg == WM_SYSCOMMAND {
            let cmd = wparam.0 & 0xFFF0;
            if cmd == SC_MINIMIZE as usize {
                let minimize_to_tray = APP_HANDLE.get().and_then(|app| {
                    let state = app.state::<AppState>();
                    state.settings.lock().ok().map(|s| s.minimize_to_tray)
                }).unwrap_or(false);

                if with_dispatcher(|d, app| d.handle_minimize(app, minimize_to_tray)).unwrap_or(false) {
                    return LRESULT(0);
                }
            }
        } else if msg == WM_COMMAND {
            let cmd = (wparam.0 & 0xFFFF) as u16;
            let notif = ((wparam.0 >> 16) & 0xFFFF) as u32;
            const THBN_CLICKED: u32 = 0x1800;

            match cmd {
                1001 if lparam.0 == 0 => {
                    with_dispatcher(|d, app| d.handle_show(app));
                    return LRESULT(0);
                }
                1002 if lparam.0 == 0 => {
                    with_dispatcher(|d, app| d.handle_open_settings(app));
                    return LRESULT(0);
                }
                1003 if lparam.0 == 0 => {
                    with_dispatcher(|d, app| d.handle_quit(app));
                    return LRESULT(0);
                }
                THB_BACK if notif == THBN_CLICKED => {
                    with_dispatcher(|d, app| d.handle_prev_track(app));
                    return LRESULT(0);
                }
                THB_PLAY if notif == THBN_CLICKED => {
                    with_dispatcher(|d, app| d.handle_play_pause(app));
                    return LRESULT(0);
                }
                THB_NEXT if notif == THBN_CLICKED => {
                    with_dispatcher(|d, app| d.handle_next_track(app));
                    return LRESULT(0);
                }
                _ => {}
            }
        }

        let prev = PREV_WNDPROC.get().copied().unwrap_or(0);
        if prev != 0 {
            let prev_proc: unsafe extern "system" fn(HWND, u32, WPARAM, LPARAM) -> LRESULT =
                unsafe { std::mem::transmute(prev) };
            unsafe { CallWindowProcW(Some(prev_proc), hwnd, msg, wparam, lparam) }
        } else {
            unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
        }
    }

    let new_proc = wndproc as *const () as isize;
    let prev = unsafe { SetWindowLongPtrW(hwnd, GWLP_WNDPROC, new_proc) };
    if prev != 0 && PREV_WNDPROC.get().is_none() {
        let _ = PREV_WNDPROC.set(prev);
    }
}

pub fn remove_minimize_hook(hwnd_raw: isize) {
    if hwnd_raw == 0 {
        return;
    }

    let hwnd = HWND(hwnd_raw as *mut std::ffi::c_void);
    if let Some(prev) = PREV_WNDPROC.get() {
        let _ = unsafe { SetWindowLongPtrW(hwnd, GWLP_WNDPROC, *prev) };
    }
}
