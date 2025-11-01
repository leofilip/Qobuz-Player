// Minimal thumbar helpers. On Windows this file integrates with the
// taskbar thumbnail toolbar (ITaskbarList3). The renderer or SMTC
// plugin is authoritative for playback state; native code forwards
// clicks into the host app.

#[cfg(not(target_os = "windows"))]
mod stub {
    use tauri::App;

    pub fn init_thumbar(_app: &App, _window_label: &str) {
        // no-op on non-windows
    }
}

#[cfg(not(target_os = "windows"))]
pub use stub::*;

#[cfg(target_os = "windows")]
mod windows_impl {
    use tauri::App;
    use std::sync::OnceLock;
    static THUMBAR_ICONS: OnceLock<Vec<usize>> = OnceLock::new();

    static STORED_HWND: OnceLock<std::sync::Mutex<Option<raw_window_handle::Win32WindowHandle>>> = OnceLock::new();
    static PREV_WNDPROC: OnceLock<isize> = OnceLock::new();

    pub fn init_thumbar(_app: &App, _window_label: &str) {
        // Thumbar initialized
    }

    /// Store the given Win32WindowHandle for later native use.
    pub fn set_stored_hwnd(h: raw_window_handle::Win32WindowHandle) {
        if STORED_HWND.get().is_some() {
            if let Some(m) = STORED_HWND.get() {
                let mut guard = m.lock().unwrap();
                *guard = Some(h);
            }
        } else {
            let _ = STORED_HWND.set(std::sync::Mutex::new(Some(h)));
        }
    }



    /// Add the thumbnail toolbar buttons and install subclass to capture clicks.
    pub fn add_thumb_buttons() {
        load_icons();
        register_subclass();
        add_thumb_buttons_native();
    }

    /// Remove thumbnail toolbar buttons and cleanup.
    pub fn remove_thumb_buttons() {
        remove_subclass();
    }

    /// Cleanup any native resources.
    pub fn cleanup_thumbar() {
        remove_subclass();
        cleanup_icons();
    }

    /// Load icon files from candidate locations and store HICON pointers.
    pub fn load_icons() {
        use windows::core::PCWSTR;
        use windows::Win32::UI::WindowsAndMessaging::LoadImageW;
        use windows::Win32::UI::WindowsAndMessaging::{LR_LOADFROMFILE, IMAGE_ICON};
        use std::path::PathBuf;
        use std::os::windows::ffi::OsStrExt;

        if THUMBAR_ICONS.get().is_some() {
            return;
        }

        let mut candidates: Vec<PathBuf> = Vec::new();
        if let Ok(s) = std::env::var("TAURI_RESOURCE_DIR") {
            let p = PathBuf::from(s);
            candidates.push(p.clone());
            candidates.push(p.join("icons"));
        }
        candidates.push(std::path::Path::new("src-tauri").join("icons"));
        if let Ok(exe) = std::env::current_exe() {
            if let Some(dir) = exe.parent() {
                candidates.push(dir.to_path_buf());
                candidates.push(dir.join("icons"));
            }
        }

    // Use three icons: Prev, Play (or Play/Pause), Next. The separate
    // pause icon was removed during redesign so we only load these three.
    let files = ["win-thumbbar/app-back.ico", "win-thumbbar/app-play.ico", "win-thumbbar/app-next.ico"];
    let mut out: Vec<usize> = Vec::with_capacity(files.len());
        let repo_root = if let Ok(cwd) = std::env::current_dir() {
            // If cwd ends with `src-tauri`, prefer its parent as repo root so
            // relative candidate paths resolve correctly during dev runs.
            if let Some(name) = cwd.file_name() {
                if name == "src-tauri" {
                    cwd.parent().map(|p| p.to_path_buf()).unwrap_or(cwd)
                } else {
                    cwd
                }
            } else { cwd }
        } else {
            std::path::PathBuf::from("")
        };

        for f in files.iter() {
            let mut found: Option<PathBuf> = None;
            for base in candidates.iter() {
                // Check base.join(f) directly and also repo_root.join(base).join(f)
                let p = base.join(f);
                if p.exists() {
                    found = Some(p);
                    break;
                }
                if base.is_relative() {
                    let p2 = repo_root.join(base).join(f);
                    if p2.exists() {
                        found = Some(p2);
                        break;
                    }
                }
            }
            let p = match found {
                Some(p) => p,
                None => { out.push(0); continue; }
            };

            let wide: Vec<u16> = p.as_os_str().encode_wide().chain(std::iter::once(0)).collect();
            let pcw = PCWSTR(wide.as_ptr());
            let res = unsafe { LoadImageW(None, pcw, IMAGE_ICON, 0, 0, LR_LOADFROMFILE) };
            match res {
                Ok(handle) => {
                    if !handle.0.is_null() {
                        // Try an explicit 16x16 load as a fallback for taskbar thumbnails
                        let alt = unsafe { LoadImageW(None, pcw, IMAGE_ICON, 16, 16, LR_LOADFROMFILE) };
                        match alt {
                            Ok(alt_handle) => {
                                if !alt_handle.0.is_null() {
                                    // Destroy original larger handle to avoid leaking it
                                    unsafe {
                                        let h = windows::Win32::UI::WindowsAndMessaging::HICON(handle.0 as *mut std::ffi::c_void);
                                        let _ = windows::Win32::UI::WindowsAndMessaging::DestroyIcon(h);
                                    }
                                    out.push(alt_handle.0 as usize);
                                } else {
                                    out.push(handle.0 as usize);
                                }
                            }
                            Err(_) => {
                                out.push(handle.0 as usize);
                            }
                        }
                    } else {
                        out.push(0);
                    }
                }
                Err(_) => { out.push(0); }
            }
        }

        let _ = THUMBAR_ICONS.set(out);
    }

    /// Free any HICONs we created.
    pub fn cleanup_icons() {
        use windows::Win32::UI::WindowsAndMessaging::DestroyIcon;
        if let Some(vec) = THUMBAR_ICONS.get() {
            for &p in vec.iter() {
                if p != 0 {
                    unsafe { let h = windows::Win32::UI::WindowsAndMessaging::HICON(p as *mut std::ffi::c_void); let _ = DestroyIcon(h); }
                }
            }
        }
        // No imagelist to destroy when using HICONs directly.
    }

    /// Add THUMBBUTTONs to the taskbar for the stored HWND.
    pub fn add_thumb_buttons_native() {
        use windows::core::{GUID, Interface};
        use windows::Win32::UI::Shell::ITaskbarList3;
        use windows::Win32::System::Com::{CoCreateInstance, CoInitializeEx, CoUninitialize, COINIT_APARTMENTTHREADED, CLSCTX_ALL};
        use std::mem::MaybeUninit;

        let hwnd_raw = if let Some(m) = STORED_HWND.get() { 
            let guard = m.lock().unwrap(); 
            if let Some(h) = guard.as_ref() { 
                h.hwnd.get() 
            } else { 
                return; 
            } 
        } else { 
            return; 
        };
        if hwnd_raw == 0 { return; }
        let hwnd = windows::Win32::Foundation::HWND(hwnd_raw as *mut std::ffi::c_void);

        if THUMBAR_ICONS.get().is_none() { load_icons(); }
        let icons = THUMBAR_ICONS.get().map(|v| v.clone()).unwrap_or_default();

        let mut raw_buttons: [windows::Win32::UI::Shell::THUMBBUTTON; 3] = unsafe { MaybeUninit::zeroed().assume_init() };

        fn set_tip(dst: &mut [u16; 260], tip: &str) {
            let mut wide: Vec<u16> = tip.encode_utf16().collect();
            wide.truncate(259);
            wide.push(0);
            for i in 0..wide.len() { dst[i] = wide[i]; }
        }

        const THB_ICON: u32 = 0x2;
        const THB_TOOLTIP: u32 = 0x4;
        const THB_FLAGS: u32 = 0x8;
        const MASK: u32 = THB_ICON | THB_TOOLTIP | THB_FLAGS;

        use windows::Win32::UI::Shell::{THUMBBUTTONFLAGS, THUMBBUTTONMASK};

    raw_buttons[0].dwMask = THUMBBUTTONMASK(MASK as i32);
    raw_buttons[0].iId = 100;
    raw_buttons[0].iBitmap = 0;
    raw_buttons[0].hIcon = if *icons.get(0).unwrap_or(&0) != 0 { windows::Win32::UI::WindowsAndMessaging::HICON(*icons.get(0).unwrap() as *mut std::ffi::c_void) } else { windows::Win32::UI::WindowsAndMessaging::HICON(std::ptr::null_mut()) };
    set_tip(&mut raw_buttons[0].szTip, "Prev");
    raw_buttons[0].dwFlags = THUMBBUTTONFLAGS(0);

    raw_buttons[1].dwMask = THUMBBUTTONMASK(MASK as i32);
    raw_buttons[1].iId = 101;
    raw_buttons[1].iBitmap = 0;
    raw_buttons[1].hIcon = if *icons.get(1).unwrap_or(&0) != 0 { windows::Win32::UI::WindowsAndMessaging::HICON(*icons.get(1).unwrap() as *mut std::ffi::c_void) } else { windows::Win32::UI::WindowsAndMessaging::HICON(std::ptr::null_mut()) };
    set_tip(&mut raw_buttons[1].szTip, "Play/Pause");
    raw_buttons[1].dwFlags = THUMBBUTTONFLAGS(0);

        raw_buttons[2].dwMask = THUMBBUTTONMASK(MASK as i32);
        raw_buttons[2].iId = 102;
        raw_buttons[2].iBitmap = 0;
        raw_buttons[2].hIcon = if *icons.get(2).unwrap_or(&0) != 0 { windows::Win32::UI::WindowsAndMessaging::HICON(*icons.get(2).unwrap() as *mut std::ffi::c_void) } else { windows::Win32::UI::WindowsAndMessaging::HICON(std::ptr::null_mut()) };
        set_tip(&mut raw_buttons[2].szTip, "Next");
        raw_buttons[2].dwFlags = THUMBBUTTONFLAGS(0);

        // NOTE: We do NOT set WM_SETICON here. Tauri automatically manages
        // the app window icon from tauri.conf.json icons. Setting WM_SETICON
        // using thumbbar icons would overwrite the app icon with thumbnail
        // button icons (e.g., back/play/next), causing the taskbar and title
        // bar to show incorrect icons. The thumbbar only manages its own
        // THUMBBUTTON HICONs; app/window icons are handled by Tauri.

        let _ = unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED) };

        let clsid = GUID::from_u128(0x56FDF344_FD6D_11D0_958A_006097C9A090u128);
        if let Ok(obj) = unsafe { CoCreateInstance(&clsid, None, CLSCTX_ALL) } {
            let unk: windows::core::IUnknown = obj;
            if let Ok(tb) = unk.cast::<ITaskbarList3>() {
                let _ = unsafe { tb.HrInit() };
                let _ = unsafe { tb.ThumbBarAddButtons(hwnd, &raw_buttons) };
            }
        }

        let _ = unsafe { CoUninitialize() };
    }

    /// Install a window subclass that captures WM_COMMAND and converts
    /// thumbnail button clicks into system media key events. This avoids
    /// forwarding WM_APPCOMMAND and prevents feedback loops.
    pub fn register_subclass() {
        // Obtain stored HWND
        let hwnd_raw = if let Some(m) = STORED_HWND.get() {
            let guard = m.lock().unwrap();
            if let Some(h) = guard.as_ref() { h.hwnd.get() } else { return } 
        } else { return };
        if hwnd_raw == 0 { return; }
        let hwnd = windows::Win32::Foundation::HWND(hwnd_raw as *mut std::ffi::c_void);

        use windows::Win32::UI::WindowsAndMessaging::{SetWindowLongPtrW, CallWindowProcW, GWLP_WNDPROC, DefWindowProcW, WM_COMMAND};
        use windows::Win32::Foundation::{WPARAM, LPARAM, LRESULT};

        unsafe extern "system" fn wndproc(
            hwnd: windows::Win32::Foundation::HWND,
            msg: u32,
            wparam: WPARAM,
            lparam: LPARAM,
        ) -> LRESULT {
            // Only intercept WM_COMMAND for THBN_CLICKED
            if msg == WM_COMMAND {
                let raw = wparam.0 as usize;
                let id = (raw & 0xffff) as u32;
                let notif = ((raw >> 16) & 0xffff) as u32;
                const THBN_CLICKED: u32 = 0x1800;
                if id >= 100 && id <= 102 && notif == THBN_CLICKED {
                    // Map ids to VK_MEDIA_* and send the media key.
                    match id {
                        100 => { super::send_media_key(0xB1); }, // Prev
                        101 => { super::send_media_key(0xB3); }, // Play/Pause
                        102 => { super::send_media_key(0xB0); }, // Next
                        _ => {}
                    }
                    // Do not call any app callbacks here; media key will
                    // be handled by the system and by tauri-plugin-media.
                }
            }

            // Call original window proc if we have it
            let prev = PREV_WNDPROC.get().copied().unwrap_or(0);
            if prev != 0 {
                let prev_proc: unsafe extern "system" fn(
                    windows::Win32::Foundation::HWND,
                    u32,
                    WPARAM,
                    LPARAM,
                ) -> LRESULT = unsafe { std::mem::transmute(prev) };
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

    pub fn remove_subclass() {
        let hwnd_raw = if let Some(m) = STORED_HWND.get() {
            let guard = m.lock().unwrap();
            if let Some(h) = guard.as_ref() { h.hwnd.get() } else { return }
        } else { return };
        if hwnd_raw == 0 { return; }
        let hwnd = windows::Win32::Foundation::HWND(hwnd_raw as *mut std::ffi::c_void);
        use windows::Win32::UI::WindowsAndMessaging::SetWindowLongPtrW;
        if let Some(prev) = PREV_WNDPROC.get() {
            let _ = unsafe { SetWindowLongPtrW(hwnd, windows::Win32::UI::WindowsAndMessaging::GWLP_WNDPROC, *prev) };
        }
    }
}

#[cfg(target_os = "windows")]
pub use windows_impl::*;

// Send a simulated media key (VK_*) to the system so apps that listen for
// media keys (including the web container) react.
#[cfg(target_os = "windows")]
pub fn send_media_key(vk: u16) {
    use windows::Win32::UI::Input::KeyboardAndMouse::{SendInput, INPUT, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_KEYUP, VIRTUAL_KEY, KEYBD_EVENT_FLAGS};

    let mut inp_down: INPUT = unsafe { std::mem::MaybeUninit::<INPUT>::zeroed().assume_init() };
    inp_down.r#type = INPUT_KEYBOARD;
    inp_down.Anonymous.ki = KEYBDINPUT {
        wVk: VIRTUAL_KEY(vk),
        wScan: 0,
        dwFlags: KEYBD_EVENT_FLAGS(0),
        time: 0,
        dwExtraInfo: 0,
    }; 

    let mut inp_up: INPUT = unsafe { std::mem::MaybeUninit::<INPUT>::zeroed().assume_init() };
    inp_up.r#type = INPUT_KEYBOARD;
    inp_up.Anonymous.ki = KEYBDINPUT {
        wVk: VIRTUAL_KEY(vk),
        wScan: 0,
        dwFlags: KEYEVENTF_KEYUP,
        time: 0,
        dwExtraInfo: 0,
    };

    let inputs = [inp_down, inp_up];
    let _ = unsafe { SendInput(&inputs, std::mem::size_of::<INPUT>() as i32) };
}

#[cfg(not(target_os = "windows"))]
pub fn send_media_key(_vk: u16) { }
 
