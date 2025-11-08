use std::sync::OnceLock;

#[cfg(target_os = "windows")]
pub mod windows_impl {
    use super::*;
    use std::sync::Mutex;
    use tauri::Manager;
    use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
    use windows::Win32::UI::WindowsAndMessaging::{
        CallWindowProcW, DefWindowProcW, SetWindowLongPtrW, GWLP_WNDPROC,
        WM_SYSCOMMAND, SC_MINIMIZE,
    };

    static MAIN_HWND: OnceLock<Mutex<Option<isize>>> = OnceLock::new();
    static PREV_WNDPROC: OnceLock<isize> = OnceLock::new();
    static APP_HANDLE: OnceLock<tauri::AppHandle> = OnceLock::new();

    pub fn init_window_manager(app: &tauri::App) {
        let _ = APP_HANDLE.set(app.handle().clone());
        let _ = MAIN_HWND.set(Mutex::new(None));
    }

    pub fn set_main_window_hwnd(hwnd: isize) {
        if let Some(mutex) = MAIN_HWND.get() {
            if let Ok(mut guard) = mutex.lock() {
                *guard = Some(hwnd);
            }
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
                if cmd == SC_MINIMIZE as usize {
                    if let Some(app) = APP_HANDLE.get() {
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
                            return LRESULT(0);
                        }
                    }
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

        let new_proc = unsafe { std::mem::transmute::<_, isize>(wndproc as *const ()) };
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
