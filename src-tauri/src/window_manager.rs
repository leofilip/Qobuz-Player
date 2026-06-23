use std::sync::OnceLock;

#[cfg(target_os = "windows")]
pub mod windows_impl {
    use super::*;
    use std::sync::Mutex;
    use tauri::Manager;
    use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
    use windows::Win32::UI::WindowsAndMessaging::{
        CallWindowProcW, DefWindowProcW, SetWindowLongPtrW, GWLP_WNDPROC,
        WM_SYSCOMMAND, WM_COMMAND, SC_MINIMIZE,
    };

    static MAIN_HWND: OnceLock<Mutex<Option<isize>>> = OnceLock::new();
    static PREV_WNDPROC: OnceLock<isize> = OnceLock::new();
    static APP_HANDLE: OnceLock<tauri::AppHandle> = OnceLock::new();

    pub fn init_window_manager(app: &tauri::App) {
        let _ = APP_HANDLE.set(app.handle().clone());
        let _ = MAIN_HWND.set(Mutex::new(None));
    }

    pub fn set_main_window_hwnd(hwnd: isize) {
        if let Some(mutex) = MAIN_HWND.get()
            && let Ok(mut guard) = mutex.lock() {
                *guard = Some(hwnd);
            }
    }

    pub fn install_minimize_hook() {
        let hwnd_raw = if let Some(m) = MAIN_HWND.get() {
            if let Ok(guard) = m.lock() {
                if let Some(h) = *guard {
                    h
                } else {
                    return;
                }
            } else {
                return;
            }
        } else {
            return;
        };

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
                if cmd == SC_MINIMIZE as usize
                    && let Some(app) = APP_HANDLE.get() {
                        let state = app.state::<crate::AppState>();
                        let minimize_to_tray = if let Ok(settings) = state.settings.lock() {
                            settings.minimize_to_tray
                        } else {
                            false
                        };

                        if minimize_to_tray {
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.hide();
                            }
                            state.tray_hidden.store(true, std::sync::atomic::Ordering::SeqCst);
                            return LRESULT(0);
                        }
                    }
            } else if msg == WM_COMMAND {
                let cmd = (wparam.0 & 0xFFFF) as u16;
                if lparam.0 == 0
                    && let Some(app) = APP_HANDLE.get() {
                        match cmd {
                            1001 => {
                                if let Some(window) = app.get_webview_window("main") {
                                    let _ = window.unminimize();
                                    let _ = window.show();
                                    let _ = window.set_focus();
                                    app.state::<crate::AppState>().tray_hidden.store(false, std::sync::atomic::Ordering::SeqCst);
                                    use raw_window_handle::HasWindowHandle;
                                    if let Ok(wh) = window.window_handle()
                                        && let raw_window_handle::RawWindowHandle::Win32(h) = wh.into() {
                                            crate::thumbar::set_stored_hwnd(h);
                                            crate::thumbar::add_thumb_buttons();
                                        }
                                }
                            }
                            1002 => {
                                crate::open_settings_window_pub(app.clone());
                            }
                            1003 => {
                                crate::thumbar::cleanup_thumbar();
                                remove_minimize_hook();
                                std::process::exit(0);
                            }
                            _ => {}
                        }
                        return LRESULT(0);
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

    pub fn remove_minimize_hook() {
        let hwnd_raw = if let Some(m) = MAIN_HWND.get() {
            if let Ok(guard) = m.lock() {
                if let Some(h) = *guard {
                    h
                } else {
                    return;
                }
            } else {
                return;
            }
        } else {
            return;
        };

        if hwnd_raw == 0 {
            return;
        }

        let hwnd = HWND(hwnd_raw as *mut std::ffi::c_void);
        if let Some(prev) = PREV_WNDPROC.get() {
            let _ = unsafe { SetWindowLongPtrW(hwnd, GWLP_WNDPROC, *prev) };
        }
    }
}

#[cfg(not(target_os = "windows"))]
pub mod windows_impl {
    pub fn init_window_manager(_app: &tauri::App) {}
    pub fn set_main_window_hwnd(_hwnd: isize) {}
    pub fn install_minimize_hook() {}
    pub fn remove_minimize_hook() {}
}

pub use windows_impl::*;
