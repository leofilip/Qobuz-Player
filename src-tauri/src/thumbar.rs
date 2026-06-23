//! Windows taskbar thumbnail toolbar buttons (Previous / Play-Pause / Next).
//!
//! Uses `ITaskbarList3::ThumbBarAddButtons` with icon files loaded via
//! `LoadImageW`. Icons are cached in a global [`OnceLock`] for the app lifetime.
//! The HWND is stored via [`set_stored_hwnd`] and must be set before calling
//! [`add_thumb_buttons`].

use tauri::App;
use std::sync::OnceLock;

use crate::interfaces::ThumbButtonConfig;

static THUMBAR_ICONS: OnceLock<Vec<usize>> = OnceLock::new();
static STORED_HWND: OnceLock<std::sync::Mutex<Option<raw_window_handle::Win32WindowHandle>>> = OnceLock::new();

/// Configuration for the three thumbnail buttons.
/// IDs must match the constants in `window_manager.rs`.
const THUMB_BUTTONS: &[ThumbButtonConfig] = &[
    ThumbButtonConfig {
        id: 100,
        tooltip: "Prev",
        icon_dev_path: "win-thumbbar/app-back.ico",
        icon_release_path: "app-back.ico",
    },
    ThumbButtonConfig {
        id: 101,
        tooltip: "Play/Pause",
        icon_dev_path: "win-thumbbar/app-play.ico",
        icon_release_path: "app-play.ico",
    },
    ThumbButtonConfig {
        id: 102,
        tooltip: "Next",
        icon_dev_path: "win-thumbbar/app-next.ico",
        icon_release_path: "app-next.ico",
    },
];

/// Initialization placeholder. Thumbar setup is lazy — icons are loaded
/// on the first call to [`add_thumb_buttons`].
pub fn init_thumbar(_app: &App, _window_label: &str) {
}

/// Stores the main window HWND so [`add_thumb_buttons`] can find it later.
pub fn set_stored_hwnd(h: raw_window_handle::Win32WindowHandle) {
    if STORED_HWND.get().is_some() {
        if let Some(m) = STORED_HWND.get()
            && let Ok(mut guard) = m.lock()
        {
            *guard = Some(h);
        }
    } else {
        let _ = STORED_HWND.set(std::sync::Mutex::new(Some(h)));
    }
}

/// Loads icon files and registers the three thumbnail buttons with the taskbar.
/// Safe to call multiple times — icons are loaded only once.
pub fn add_thumb_buttons() {
    load_icons();
    add_thumb_buttons_native();
}

/// Removes thumbnail buttons from the taskbar. Currently a no-op because
/// the buttons disappear automatically when the window is destroyed.
pub fn remove_thumb_buttons() {
}

/// Destroys loaded icon handles to avoid resource leaks on app exit.
pub fn cleanup_thumbar() {
    cleanup_icons();
}

fn load_icons() {
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
    if let Ok(exe) = std::env::current_exe()
        && let Some(dir) = exe.parent()
    {
        candidates.push(dir.to_path_buf());
        candidates.push(dir.join("icons"));
        candidates.push(dir.join("resources"));
    }

    let repo_root = if let Ok(cwd) = std::env::current_dir() {
        if let Some(name) = cwd.file_name() {
            if name == "src-tauri" {
                cwd.parent().map(|p| p.to_path_buf()).unwrap_or(cwd)
            } else {
                cwd
            }
        } else {
            cwd
        }
    } else {
        std::path::PathBuf::from("")
    };

    let mut out: Vec<usize> = Vec::with_capacity(THUMB_BUTTONS.len());

    for config in THUMB_BUTTONS {
        let mut found: Option<PathBuf> = None;
        for base in candidates.iter() {
            let p = base.join(config.icon_dev_path);
            if p.exists() {
                found = Some(p);
                break;
            }
            let p_flat = base.join(config.icon_release_path);
            if p_flat.exists() {
                found = Some(p_flat);
                break;
            }
            if base.is_relative() {
                let p2 = repo_root.join(base).join(config.icon_dev_path);
                if p2.exists() {
                    found = Some(p2);
                    break;
                }
                let p2_flat = repo_root.join(base).join(config.icon_release_path);
                if p2_flat.exists() {
                    found = Some(p2_flat);
                    break;
                }
            }
        }
        let p = match found {
            Some(p) => p,
            None => {
                out.push(0);
                continue;
            }
        };

        let wide: Vec<u16> = p.as_os_str().encode_wide().chain(std::iter::once(0)).collect();
        let pcw = PCWSTR(wide.as_ptr());
        let res = unsafe { LoadImageW(None, pcw, IMAGE_ICON, 0, 0, LR_LOADFROMFILE) };
        match res {
            Ok(handle) => {
                if !handle.0.is_null() {
                    let alt = unsafe { LoadImageW(None, pcw, IMAGE_ICON, 16, 16, LR_LOADFROMFILE) };
                    match alt {
                        Ok(alt_handle) => {
                            if !alt_handle.0.is_null() {
                                unsafe {
                                    let h = windows::Win32::UI::WindowsAndMessaging::HICON(handle.0);
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
            Err(_) => {
                out.push(0);
            }
        }
    }

    let _ = THUMBAR_ICONS.set(out);
}

fn cleanup_icons() {
    use windows::Win32::UI::WindowsAndMessaging::DestroyIcon;
    if let Some(vec) = THUMBAR_ICONS.get() {
        for &p in vec.iter() {
            if p != 0 {
                unsafe {
                    let h = windows::Win32::UI::WindowsAndMessaging::HICON(p as *mut std::ffi::c_void);
                    let _ = DestroyIcon(h);
                }
            }
        }
    }
}

fn add_thumb_buttons_native() {
    use windows::core::{GUID, Interface};
    use windows::Win32::UI::Shell::ITaskbarList3;
    use windows::Win32::System::Com::{CoCreateInstance, CoInitializeEx, CoUninitialize, COINIT_APARTMENTTHREADED, CLSCTX_ALL};
    use std::mem::MaybeUninit;

    let hwnd_raw = if let Some(m) = STORED_HWND.get() {
        if let Ok(guard) = m.lock() {
            if let Some(h) = guard.as_ref() {
                h.hwnd.get()
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
    let hwnd = windows::Win32::Foundation::HWND(hwnd_raw as *mut std::ffi::c_void);

    if THUMBAR_ICONS.get().is_none() {
        load_icons();
    }
    let icons = THUMBAR_ICONS.get().cloned().unwrap_or_default();

    let mut raw_buttons: Vec<windows::Win32::UI::Shell::THUMBBUTTON> =
        Vec::with_capacity(THUMB_BUTTONS.len());

    const THB_ICON: u32 = 0x2;
    const THB_TOOLTIP: u32 = 0x4;
    const THB_FLAGS: u32 = 0x8;
    const MASK: u32 = THB_ICON | THB_TOOLTIP | THB_FLAGS;

    use windows::Win32::UI::Shell::{THUMBBUTTONFLAGS, THUMBBUTTONMASK};

    for (i, config) in THUMB_BUTTONS.iter().enumerate() {
        let mut btn: windows::Win32::UI::Shell::THUMBBUTTON = unsafe { MaybeUninit::zeroed().assume_init() };
        btn.dwMask = THUMBBUTTONMASK(MASK as i32);
        btn.iId = config.id as u32;
        btn.iBitmap = 0;

        let icon_handle = icons.get(i).copied().unwrap_or(0);
        btn.hIcon = if icon_handle != 0 {
            windows::Win32::UI::WindowsAndMessaging::HICON(icon_handle as *mut std::ffi::c_void)
        } else {
            windows::Win32::UI::WindowsAndMessaging::HICON(std::ptr::null_mut())
        };

        let mut wide: Vec<u16> = config.tooltip.encode_utf16().collect();
        wide.truncate(259);
        wide.push(0);
        let tip_len = wide.len().min(260);
        btn.szTip[..tip_len].copy_from_slice(&wide[..tip_len]);

        btn.dwFlags = THUMBBUTTONFLAGS(0);
        raw_buttons.push(btn);
    }

    let com_initialized = unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED) }.is_ok();

    let clsid = GUID::from_u128(0x56FDF344_FD6D_11D0_958A_006097C9A090u128);
    if let Ok(obj) = unsafe { CoCreateInstance(&clsid, None, CLSCTX_ALL) } {
        let unk: windows::core::IUnknown = obj;
        if let Ok(tb) = unk.cast::<ITaskbarList3>() {
            let _ = unsafe { tb.HrInit() };
            let _ = unsafe { tb.ThumbBarAddButtons(hwnd, &raw_buttons) };
        }
    }

    if com_initialized {
        unsafe { CoUninitialize() };
    }
}
