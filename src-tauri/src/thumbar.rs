// Native Windows implementation using the Win32 ITaskbarList3 THUMBBUTTON API.
// Exposes:
//  - `init_thumbar(app_handle, window_label)` to initialize thumbar buttons
//    for the given Tauri window label (typically "main").
//  - `set_playing(is_playing)` to toggle the middle button icon (play vs pause).
//
// On non-Windows platforms these functions are no-ops.

#[cfg(not(target_os = "windows"))]
mod stub {
    use tauri::App;

    pub fn init_thumbar(_app: &App, _window_label: &str) {
        // no-op on non-windows
    }

    pub fn set_playing(_is_playing: bool) {}
}

#[cfg(not(target_os = "windows"))]
pub use stub::*;

#[cfg(target_os = "windows")]
mod windows_impl {
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::{Arc, OnceLock};
    use tauri::App;
    // raw-window-handle not required for the simplified init; we will add
    // low-level HWND handling in the next iteration.

    static THUMBAR_STATE: OnceLock<Arc<AtomicBool>> = OnceLock::new();

    pub fn init_thumbar(_app: &App, window_label: &str) {
        let state = Arc::new(AtomicBool::new(false));
        let _ = THUMBAR_STATE.set(state.clone());

        // Minimal init: record state and log. Detailed native thumbar
        // integration (HWND, ITaskbarList3) will be implemented next.
        println!("thumbar: initialized for window label: {}", window_label);
    }

    pub fn set_playing(is_playing: bool) {
        if let Some(s) = THUMBAR_STATE.get() {
            s.store(is_playing, Ordering::SeqCst);
        }
    }
}

#[cfg(target_os = "windows")]
pub use windows_impl::*;
 
