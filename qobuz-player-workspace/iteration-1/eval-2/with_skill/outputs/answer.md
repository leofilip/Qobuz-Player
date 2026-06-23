# Thumbnail Toolbar — End-to-End Flow

## 1. App Startup (`main.rs:244–557`)

**`main()`** calls `settings::Settings::load()`, then builds the Tauri app with:
- 9 IPC commands registered via `generate_handler![]` (including `native_add_thumb_buttons` and `native_remove_thumb_buttons` at lines 252–253)
- `AppState` wrapping settings in a `Mutex`

**`setup()` closure** runs after the app builds:

1. **Sets AppUserModelID** (`main.rs:271–283`) — calls `SetCurrentProcessExplicitAppUserModelID` with `"com.leo.qobuz-player"` (release) or `"com.leo.qobuz-player.dev"` (debug). This groups the taskbar icon properly.

2. **Creates system tray** (`main.rs:285–348`) — 3 menu items: Show, Settings, Quit. The tray's `on_menu_event` and `on_tray_icon_event` handlers will later be where thumbnail buttons get added.

3. **Initializes thumbar module** (`main.rs:350`) — `thumbar::init_thumbar(app, "main")` stores the Tauri `AppHandle` in the `APP_HANDLE` OnceLock (`thumbar.rs:28–30`).

4. **Stores HWND** (`main.rs:525–530`) — Gets the raw Win32 HWND via `window.window_handle()`, stores it in `STORED_HWND` via `thumbar::set_stored_hwnd(h)`.

**Important: thumbnail buttons are NOT added at startup.** They are added lazily later (see step 3).

## 2. IPC Command Registration (`main.rs:19–27`)

Two commands bridge the frontend to the Rust backend:

```rust
#[tauri::command]
fn native_add_thumb_buttons() {
    thumbar::add_thumb_buttons();   // main.rs:21
}

#[tauri::command]
fn native_remove_thumb_buttons() {
    thumbar::remove_thumb_buttons(); // main.rs:26
}
```

These can be called from the Qobuz webview via `window.__TAURI__.core.invoke('native_add_thumb_buttons')` or `window.__TAURI__.core.invoke('native_remove_thumb_buttons')`. They are registered in the `invoke_handler` array at lines 251–253.

## 3. When Thumbnail Buttons Are Added

Buttons are added in **three scenarios**:

### A. Tray "Show" clicked (`main.rs:300–312`)
```rust
"show" => {
    window.unminimize(); window.show(); window.set_focus();
    thumbar::set_stored_hwnd(h);
    thumbar::add_thumb_buttons();  // ← adds buttons
}
```
Restores the window and creates (or recreates) the thumbnail buttons.

### B. Tray icon double-clicked (`main.rs:319–345`)
Same pattern — restores window, re-stores HWND (safe — `set_stored_hwnd` updates the already-initialized `OnceLock`), calls `add_thumb_buttons()`.

### C. Frontend IPC call
Any JavaScript in the Qobuz page can call `invoke('native_add_thumb_buttons')` to toggle them on demand.

## 4. Button Creation (`thumbar.rs:43–47`)

`add_thumb_buttons()` is a three-step orchestration:

```rust
pub fn add_thumb_buttons() {
    load_icons();            // step A
    register_subclass();     // step B
    add_thumb_buttons_native(); // step C
}
```

### Step A — `load_icons()` (`thumbar.rs:58–164`)

Loads three `.ico` files from disk into HICON handles:

| File | Button |
|------|--------|
| `win-thumbbar/app-back.ico` | Previous (ID 100) |
| `win-thumbbar/app-play.ico` | Play/Pause (ID 101) |
| `win-thumbbar/app-next.ico` | Next (ID 102) |

**Search order**: `TAURI_RESOURCE_DIR` env var → `TAURI_RESOURCE_DIR/icons` → `src-tauri/icons` → exe directory → exe directory/icons → exe directory/resources. For each file, tries the dev path (with `win-thumbbar/` prefix) then the flat release path.

Each icon is loaded via `LoadImageW` with `LR_LOADFROMFILE`. A second `LoadImageW` call with explicit 16×16 size is attempted; if it succeeds, the default-sized icon is destroyed and the 16×16 handle is kept (smaller icons look better in the thumbnail bar).

Icons are stored once in `THUMBAR_ICONS: OnceLock<Vec<usize>>` — subsequent calls return immediately (line 65–67).

### Step B — `register_subclass()` (`thumbar.rs:253–310`)

Installs a Win32 window subclass by replacing the window procedure:

1. Retrieves HWND from `STORED_HWND`
2. Defines `wndproc()` — an `unsafe extern "system"` function that intercepts window messages
3. Calls `SetWindowLongPtrW(hwnd, GWLP_WNDPROC, new_proc)` to replace the window procedure
4. Stores the **previous** WNDPROC pointer in `PREV_WNDPROC: OnceLock<isize>` for chaining

The `wndproc` chains to the original via `CallWindowProcW` (line 299), falling back to `DefWindowProcW` if no previous proc exists (line 301).

### Step C — `add_thumb_buttons_native()` (`thumbar.rs:177–251`)

Creates the COM `ITaskbarList3` interface and adds three `THUMBBUTTON` structs:

1. **Initializes COM** via `CoInitializeEx(COINIT_APARTMENTTHREADED)` (line 239)
2. **Creates COM object** via `CoCreateInstance` with CLSID `{56FDF344-FD6D-11D0-958A-006097C9A090}` (the TaskbarList CLSID) (line 242)
3. **Casts** `IUnknown` → `ITaskbarList3` (line 244)
4. **Calls `HrInit()`** to initialize the object (line 245)
5. **Builds 3 THUMBBUTTONs** (lines 218–237):
   - Button 100: icon from `icons[0]`, tooltip "Prev"
   - Button 101: icon from `icons[1]`, tooltip "Play/Pause"
   - Button 102: icon from `icons[2]`, tooltip "Next"
   - Each has `dwMask = THB_ICON | THB_TOOLTIP | THB_FLAGS` and `dwFlags = 0` (enabled)
6. **Calls `ThumbBarAddButtons(hwnd, &raw_buttons)`** (line 246) — the actual Win32 API that registers the buttons on the taskbar thumbnail preview
7. **Uninitializes COM** via `CoUninitialize()` (line 250)

## 5. Button Click — the WM_COMMAND path (`thumbar.rs:265–303`)

When a user clicks a thumbnail button, Windows sends a `WM_COMMAND` message to the window procedure:

```
WM_COMMAND
  wParam: LOWORD = button ID (100, 101, 102)
          HIWORD = THBN_CLICKED (0x1800)
```

The custom `wndproc` intercepts this:

```rust
if msg == WM_COMMAND {
    let raw = wparam.0;
    let id = (raw & 0xffff) as u32;           // button ID
    let notif = ((raw >> 16) & 0xffff) as u32;  // notification code
    const THBN_CLICKED: u32 = 0x1800;
    if (100..=102).contains(&id) && notif == THBN_CLICKED
        && let Some(app) = APP_HANDLE.get()
        && let Some(window) = app.get_webview_window("main") {
            let js = match id { ... };
            window.eval(&js);
        }
}
```

Three match arms produce three different JavaScript strings:

### Button 100 — Previous Track
```javascript
(function() {
    let selectors = [
        'button[aria-label*="revious"]',
        'button[aria-label*="Previous"]',
        'button[aria-label*="PREVIOUS"]',
        'button[title*="revious"]',
        'button[title*="Previous"]',
        '.pct-player-previous',
        '.player__action-previous',
        'button[class*="previous"]',
        'button[class*="prev"]',
        'button[class*="back"]',
        '[data-testid*="previous"]',
        '[data-testid*="prev"]',
        'button.pct-player-previous',
        'span.pct-player-previous'
    ];
    for(let s of selectors) {
        let el = document.querySelector(s);
        if(el) { el.click(); return; }
    }
})()
```
Iterates through 14 CSS selector fallbacks, clicks the first match found.

### Button 101 — Play/Pause
```javascript
(function() {
    let m = document.querySelector('audio, video');
    if(m) {
        if(m.paused) m.play(); else m.pause();
    } else {
        document.querySelector(
            'button[aria-label*="lay"], button[aria-label*="ause"], .play-button, .pause-button, .pct-player-play, .pct-player-pause'
        )?.click();
    }
})()
```
Tries native HTMLMediaElement play/pause first (most reliable), falls back to CSS selector-based clicks on the Qobuz DOM.

### Button 102 — Next Track
```javascript
(function() {
    let selectors = [
        'button[aria-label*="ext"]',
        'button[aria-label*="Next"]',
        '.pct-player-next',
        'button[class*="next"]',
        '[data-testid*="next"]'
    ];
    for(let s of selectors) {
        let el = document.querySelector(s);
        if(el) { el.click(); return; }
    }
})()
```
Five CSS selector fallbacks for the next button.

The message then passes through to the **original window procedure** via `CallWindowProcW`/`DefWindowProcW` (lines 291–302), so the app's normal message handling continues uninterrupted.

## 6. Cleanup

### `remove_thumb_buttons()` (`thumbar.rs:49–51`)
Calls `remove_subclass()` — restores the original WNDPROC via `SetWindowLongPtrW(hwnd, GWLP_WNDPROC, prev)` (lines 312–324).

### `cleanup_thumbar()` (`thumbar.rs:53–56`)
Calls `remove_subclass()` then `cleanup_icons()` — iterates `THUMBAR_ICONS`, calls `DestroyIcon` on each handle.

Called from:
- **Quit** (`main.rs:296`) — tray menu quit handler
- **Close without tray** (`main.rs:551`) — `CloseRequested` event when `close_to_tray` is disabled

## Summary: Key Files and Functions

| File | Function/Line | Role |
|------|--------------|------|
| `main.rs:20–22` | `native_add_thumb_buttons()` | IPC command — Rust-side entry point |
| `main.rs:25–27` | `native_remove_thumb_buttons()` | IPC command — remove thumb buttons |
| `main.rs:300–312` | Tray "show" handler | Calls `add_thumb_buttons()` when restoring |
| `main.rs:341–344` | Tray double-click handler | Calls `add_thumb_buttons()` on double-click |
| `main.rs:350` | `setup()` | `init_thumbar()` stores AppHandle |
| `main.rs:527` | `setup()` | `set_stored_hwnd()` stores the HWND |
| `thumbar.rs:28–30` | `init_thumbar()` | Saves AppHandle in `APP_HANDLE` OnceLock |
| `thumbar.rs:32–41` | `set_stored_hwnd()` | Saves HWND in `STORED_HWND` OnceLock |
| `thumbar.rs:43–47` | `add_thumb_buttons()` | Orchestrates icons + subclass + native |
| `thumbar.rs:58–164` | `load_icons()` | Loads 3 .ico files → HICON handles |
| `thumbar.rs:177–251` | `add_thumb_buttons_native()` | COM ITaskbarList3 → ThumbBarAddButtons |
| `thumbar.rs:253–310` | `register_subclass()` | SetWindowLongPtrW → custom wndproc |
| `thumbar.rs:265–303` | `wndproc()` | Intercepts WM_COMMAND/THBN_CLICKED → `window.eval()` |
| `thumbar.rs:312–324` | `remove_subclass()` | Restores original WNDPROC |
| `thumbar.rs:53–56` | `cleanup_thumbar()` | Full teardown: subclass + DestroyIcon |
| `settings.html:265–445` | Settings UI JS | Can invoke `native_add_thumb_buttons`/`native_remove_thumb_buttons` via IPC |
