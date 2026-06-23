# Thumbnail Toolbar Buttons — End-to-End Flow

This is a Tauri 2 app that wraps the Qobuz web player in a WebView2 window on Windows. The thumbnail toolbar (taskbar preview buttons) gives Prev/PlayPause/Next controls. Here's the full flow:

---

## 1. App Startup (`main.rs:244-557`)

`main()` in `src-tauri/src/main.rs` creates the Tauri app. Inside `.setup()`:

1. **Sets AppUserModelID** (Windows branding, `main.rs:270-283`) — required for taskbar integration.
2. **Creates the tray icon** with Show/Settings/Quit menu (`main.rs:285-348`).
3. **Calls `thumbar::init_thumbar(app, "main")`** (`main.rs:350`) — this stores the `tauri::AppHandle` in the global `APP_HANDLE` OnceLock (`thumbar.rs:29`). This handle is needed later to get the webview window and run JS.
4. **Gets the window's raw HWND** (`main.rs:525-530`) by calling `window.window_handle()` and matching `RawWindowHandle::Win32(h)`. It stores this HWND in `thumbar::STORED_HWND` via `set_stored_hwnd(h)`, and in `window_manager::MAIN_HWND` for the minimize hook.

**Crucially, thumbnail buttons are NOT added at first launch**. They are only added when the window is shown from the tray.

---

## 2. Triggering Thumbnail Bar Creation

Thumbnail buttons are added in two paths, both in `main.rs`:

### Tray "Show" menu event (`main.rs:300-312`)

```rust
"show" => {
    // unminimize + show + focus the window
    // get HWND via window_handle() → set_stored_hwnd(h) → add_thumb_buttons()
}
```

### Tray icon double-click (`main.rs:319-346`)

Same pattern: show the window, re-acquire HWND, call `add_thumb_buttons()`.

Both paths call `thumbar::add_thumb_buttons()` which runs three steps:

---

## 3. `add_thumb_buttons()` (`thumbar.rs:43-47`)

```rust
pub fn add_thumb_buttons() {
    load_icons();          // Load .ico files for the 3 buttons
    register_subclass();   // Install custom window proc to intercept clicks
    add_thumb_buttons_native();  // Tell Windows taskbar about the buttons
}
```

### 3a. `load_icons()` (`thumbar.rs:58-164`)

Searches for three icon files across multiple paths: `TAURI_RESOURCE_DIR`, `src-tauri/icons`, the exe directory, and `repo_root/src-tauri`:

- `win-thumbbar/app-back.ico` (or `app-back.ico`) — for "Prev"
- `win-thumbbar/app-play.ico` (or `app-play.ico`) — for "Play/Pause"
- `win-thumbbar/app-next.ico` (or `app-next.ico`) — for "Next"

For each file, it calls `LoadImageW` to load it as an `IMAGE_ICON`. It first loads at default size, then tries to load at 16x16 (toolbar-appropriate). The 16x16 version is preferred if available. Handles are stored in `THUMBAR_ICONS: OnceLock<Vec<usize>>`.

### 3b. `register_subclass()` (`thumbar.rs:253-310`)

Retrieves the stored HWND and installs a custom window procedure via `SetWindowLongPtrW(hwnd, GWLP_WNDPROC, new_proc)`.

The custom proc (`wndproc`, `thumbar.rs:265-303`) intercepts `WM_COMMAND` messages. When a thumbnail button is clicked, Windows sends `WM_COMMAND` with:
- `wparam` low 16 bits = button ID (100, 101, or 102)
- `wparam` high 16 bits = notification code (`THBN_CLICKED` = 0x1800)

The proc checks if `id` is in range 100-102 AND `notif == THBN_CLICKED`. If so, it:

1. Gets the `AppHandle` from `APP_HANDLE` OnceLock
2. Gets the "main" webview window
3. Executes JS via `window.eval()` matching the button ID

If the message is not a thumbnail click, it chains to the original window proc via `CallWindowProcW`.

### 3c. `add_thumb_buttons_native()` (`thumbar.rs:177-251`)

Creates 3 `THUMBBUTTON` structs (a Win32 API type):

| Button | ID | Tooltip | Icon |
|--------|----|---------|------|
| Prev | 100 | "Prev" | app-back.ico |
| Play/Pause | 101 | "Play/Pause" | app-play.ico |
| Next | 102 | "Next" | app-next.ico |

Each button has `dwMask` = `THB_ICON | THB_TOOLTIP | THB_FLAGS` (0xE) and `dwFlags` = 0 (enabled).

It then:
1. Calls `CoInitializeEx` (COM initialization)
2. Creates `ITaskbarList3` COM object via `CoCreateInstance` with CLSID `{56FDF344-FD6D-11D0-958A-006097C9A090}`
3. Calls `HrInit()` on the taskbar list
4. Calls `ThumbBarAddButtons(hwnd, &raw_buttons)` to register the 3 buttons with the Windows taskbar
5. Calls `CoUninitialize`

---

## 4. User Clicks a Thumbnail Button

When the user clicks a thumbnail button in the taskbar preview:

1. **Windows sends `WM_COMMAND`** to the window with the button ID (100-102) and `THBN_CLICKED` (0x1800).

2. **The subclassed `wndproc` catches it** (`thumbar.rs:271-288`) and matches the ID:

   - **ID 100 (Prev):** Injects JS that tries a cascade of DOM selectors for "previous" buttons:
     ```js
     ['button[aria-label*="revious"]', 'button[aria-label*="Previous"]',
      'button[aria-label*="PREVIOUS"]', 'button[title*="revious"]',
      'button[title*="Previous"]', '.pct-player-previous',
      '.player__action-previous', 'button[class*="previous"]',
      'button[class*="prev"]', 'button[class*="back"]',
      '[data-testid*="previous"]', '[data-testid*="prev"]',
      'button.pct-player-previous', 'span.pct-player-previous']
     ```
     Iterates through them and clicks the first match.

   - **ID 101 (Play/Pause):** Injects JS that first checks for an `<audio>` or `<video>` element and toggles `play()`/`pause()`. Falls back to trying selectors like `button[aria-label*="lay"]` (play), `button[aria-label*="ause"]` (pause), `.play-button`, `.pause-button`, `.pct-player-play`, `.pct-player-pause`.

   - **ID 102 (Next):** Injects JS trying selectors like `button[aria-label*="ext"]`, `.pct-player-next`, `[data-testid*="next"]`, etc.

3. The webview executes the JS, finds the Qobuz player DOM element, and clicks it — Qobuz's own JS handles the actual playback control.

---

## 5. Cleanup

### On Quit (`main.rs:296-299`)

```rust
"quit" => {
    thumbar::cleanup_thumbar();    // remove subclass + destroy icons
    window_manager::remove_minimize_hook();
    std::process::exit(0);
}
```

`cleanup_thumbar()` (`thumbar.rs:53-56`):
- Calls `remove_subclass()` — restores the original window proc via `SetWindowLongPtrW(hwnd, GWLP_WNDPROC, prev_proc)` using the saved `PREV_WNDPROC` value.
- Calls `cleanup_icons()` — iterates `THUMBAR_ICONS` and calls `DestroyIcon` on each handle.

### On Window Close (`main.rs:535-555`)

If `close_to_tray` is enabled (default), the window is simply hidden (not destroyed), so the thumbnail bar persists until the next show/hide cycle. If `close_to_tray` is disabled, `cleanup_thumbar()` and `remove_minimize_hook()` are called before the window closes.

---

## 6. Registered but Unused Tauri Commands

`main.rs:20-27` registers two Tauri commands:

- `native_add_thumb_buttons()` — calls `thumbar::add_thumb_buttons()`
- `native_remove_thumb_buttons()` — calls `thumbar::remove_thumb_buttons()`

These are listed in `generate_handler![]` (`main.rs:252-253`) but are **never invoked from the frontend JS** (there are no frontend source files calling them). They exist as future invoke targets. Currently, all thumbnail management happens from the Rust-side tray event handlers.

---

## Key Connection Diagram

```
main() → .setup() → init_thumbar() [stores AppHandle]
                   → set_stored_hwnd() [stores HWND]
                   → install_minimize_hook() [separate window_manager hook]
                   → eval(init_script) [injects custom titlebar in webview]

Tray "Show" / Double-click
    → show window + get HWND
    → add_thumb_buttons()
        → load_icons()           [loads 3 .ico files]
        → register_subclass()    [SetWindowLongPtrW + custom wndproc]
        → add_thumb_buttons_native()  [CoCreateInstance(ITaskbarList3) + ThumbBarAddButtons]

User clicks button in taskbar preview
    → Windows sends WM_COMMAND (id=100/101/102, THBN_CLICKED)
    → wndproc intercepts
    → app.get_webview_window("main").eval(js)
    → JS clicks Qobuz DOM element → Qobuz player responds
```
