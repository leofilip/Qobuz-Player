---
name: rust-crates
description: >
  Reference for the specific crate versions and APIs used in the Qobuz-Player
  project. Use this skill whenever generating or modifying Rust code that calls
  external crate APIs — including the `windows` crate (Win32 FFI), `serde`/
  `serde_json` (serialization), `raw-window-handle` (window handles),
  `tauri-plugin-*` APIs, `base64`, `dirs`, `winreg`, or `tauri-build`.
  Check Cargo.toml versions before writing code that uses these crates to ensure
  API compatibility. Do NOT use for the core `tauri` crate (covered by the tauri2 skill)
  or general Rust language patterns (covered by rust-lang skill).
---
# Rust Crate Dependency Skill

Crate versions are pinned in `src-tauri/Cargo.toml`. ALWAYS check the version there
before writing code to ensure API compatibility.

## Dependency Table

| Crate | Version | Notes |
|-------|---------|-------|
| `serde` | 1.0.228 | With `derive` feature |
| `serde_json` | 1.0.145 | JSON serialization |
| `base64` | 0.22 | Base64 encode/decode |
| `windows` | 0.62.2 | Win32 FFI — see features below |
| `raw-window-handle` | 0.6.2 | Cross-platform window handle access |
| `dirs` | 5.0 | Platform config/data dirs |
| `winreg` | 0.52 | Windows Registry (Windows-only, via `[target.'cfg(target_os = "windows")'.dependencies]`) |
| `tauri-build` | 2.5.1 | Build-time (in `[build-dependencies]`) |
| `tauri-plugin-opener` | 2.5.2 | Open URLs/files externally |
| `tauri-plugin-media` | 0.1.1 | Media session (playback controls) |
| `tauri-plugin-single-instance` | 2.3.6 | Prevent duplicate instances |

## `serde` 1.0.228

Docs: <https://docs.rs/serde/1.0.228>

- `#[derive(Serialize, Deserialize)]` for struct/enum serialization
- `#[serde(rename = "snake_case")]`, `#[serde(default)]`, `#[serde(skip_serializing_if = "...")]`
- `serde_json::to_string_pretty(&value)` for pretty-printed JSON
- `serde_json::from_str::<T>(&json_str)` for deserialization

## `windows` 0.62.2

Docs: <https://docs.rs/crate/windows/0.62.2>

This is the Microsoft `windows` crate (NOT `windows-sys`). It provides type-safe
bindings to Win32 and WinRT APIs. Version 0.62.2 uses typed wrappers for flags
parameters — e.g., `MENU_ITEM_FLAGS(0u32)` for `MF_STRING`, `TRACK_POPUP_MENU_FLAGS::default()`
for `TrackPopupMenu` flags.

### Features Enabled in this Project
- `Win32_Foundation` — `HWND`, `POINT`, `BOOL`, `RECT`, `LRESULT`, `WPARAM`, `LPARAM`, `HMENU`
- `Win32_System_Com` — COM interfaces
- `Win32_UI_Shell` — `Shell_NotifyIconW`, `SetCurrentProcessExplicitAppUserModelID`, `ITaskbarList4`
- `Win32_UI_WindowsAndMessaging` — window procs, messages (`WM_SYSCOMMAND`, `WM_COMMAND`), `CallWindowProcW`, `SetWindowLongPtrW`, `CreatePopupMenu`, `AppendMenuW`, `TrackPopupMenu`, `DestroyMenu`, `GetCursorPos`
- `Win32_UI_Controls` — common controls
- `Win32_UI_Input_KeyboardAndMouse` — `GetCursorPos`, input handling
- `Win32_Graphics_Gdi` — GDI graphics
- `Media_Control`, `Media_Playback` — SystemMediaTransportControls
- `Foundation`, `Foundation_Collections` — WinRT foundations

### Common Patterns

```rust
use windows::Win32::UI::WindowsAndMessaging::{
    CreatePopupMenu, AppendMenuW, MENU_ITEM_FLAGS,
};
use windows::Win32::Foundation::HWND;
use windows::core::PWSTR;

unsafe {
    let Ok(hmenu) = CreatePopupMenu() else { return; };
    let text: Vec<u16> = "Item\0".encode_utf16().collect();
    let _ = AppendMenuW(hmenu, MENU_ITEM_FLAGS(0u32), id, PWSTR(text.as_ptr() as *mut _));
}
```

- `CreatePopupMenu()` returns `Result<HMENU, Error>` (NOT raw HMENU)
- `AppendMenuW()` flags parameter is `MENU_ITEM_FLAGS` (NOT `u32`)
- `TrackPopupMenu()` flags parameter is `TRACK_POPUP_MENU_FLAGS` (NOT `u32`)
- `nReserved` parameter of `TrackPopupMenu` is `Option<i32>` (NOT `i32`)
- `TrackPopupMenu()` returns `Result<()>` — command ID is sent via `WM_COMMAND` to the owner window
- Strings must be null-terminated UTF-16 (`"text\0".encode_utf16().collect::<Vec<u16>>()`)
- `PWSTR` wraps `*mut u16` — cast from `Vec<u16>::as_mut_ptr()`
- `POINT { x, y }` has public `x` and `y` fields (both `i32`)

### Window Subclassing

```rust
use windows::Win32::UI::WindowsAndMessaging::{
    SetWindowLongPtrW, CallWindowProcW, GWLP_WNDPROC,
    WM_SYSCOMMAND, WM_COMMAND, SC_MINIMIZE,
};

unsafe extern "system" fn wndproc(
    hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM,
) -> LRESULT {
    if msg == WM_SYSCOMMAND { ... }
    else if msg == WM_COMMAND { ... }
    // fall through to previous proc
    CallWindowProcW(Some(prev), hwnd, msg, wparam, lparam)
}
```

## `raw-window-handle` 0.6.2

Docs: <https://docs.rs/raw-window-handle/0.6.2>

Used to obtain platform-specific window handles from Tauri windows.

```rust
use raw_window_handle::HasWindowHandle;

// Get the HWND from a Tauri window
let wh = window.window_handle()?;
if let raw_window_handle::RawWindowHandle::Win32(h) = wh.into() {
    let hwnd = HWND(h.hwnd.get() as *mut std::ffi::c_void);
}
```

- `Win32WindowHandle.hwnd` is `NonNull<c_void>` — use `.get()` to get the raw pointer
- `HasWindowHandle` trait provides `.window_handle()` returning `Result<WindowHandle<'_>>`
- `WindowHandle<'_>` borrows from the window — must extract data within the borrow scope

## `base64` 0.22

Docs: <https://docs.rs/crate/base64/0.22>

```rust
use base64::{Engine as _, engine::general_purpose};

let encoded = general_purpose::STANDARD.encode(&bytes);
let decoded = general_purpose::STANDARD.decode(&encoded)?;
```

## `dirs` 5.0

Docs: <https://docs.rs/crate/dirs/5.0>

```rust
let config_dir = dirs::config_dir().unwrap_or_default();
let data_dir = dirs::data_dir().unwrap_or_default();
```

## `winreg` 0.52 (Windows-only)

Docs: <https://docs.rs/crate/winreg/0.52>

```rust
use winreg::RegKey;
use winreg::enums::*;

let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
let key = hklm.open_subkey_with_flags("SOFTWARE\\Path", KEY_READ)?;
let value: String = key.get_value("Name")?;
```

## `tauri-build` 2.5.1

Docs: <https://docs.rs/crate/tauri-build/2.5.1>

Used in `build.rs`:
```rust
fn main() {
    tauri_build::build();
}
```

## `tauri-plugin-opener` 2.5.2

Docs: <https://docs.rs/crate/tauri-plugin-opener/2.5.2>

```rust
use tauri_plugin_opener::open_url;
// or in frontend via JS plugin API
```

## `tauri-plugin-single-instance` 2.3.6

Docs: <https://docs.rs/crate/tauri-plugin-single-instance/2.3.6>

```rust
.plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
    }
}))
```

## `tauri-plugin-media` 0.1.1

Docs: <https://docs.rs/crate/tauri-plugin-media/0.1.1>

Provides media session integration. Used for system media controls (play/pause/next/prev
on Windows taskbar and macOS Now Playing).
