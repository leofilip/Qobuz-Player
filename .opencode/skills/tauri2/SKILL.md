---
name: tauri2
description: >
  Tauri 2 framework reference for the Qobuz-Player desktop app. Use this skill for
  ANY Tauri-specific code — tray icon setup and events, window management
  (WebviewWindow, WindowEvent, hide/show/minimize), IPC between Rust and frontend
  (#[tauri::command], invoke, events), menu system (Menu, MenuItem, Submenu), app
  lifecycle (Builder, setup, manage state, plugins), and platform-specific
  Windows integration. Triggers when working with tauri.conf.json, capabilities,
  window creation, tray/menu, state management, or Rust-frontend communication.
  This project uses Tauri 2.9.1 — consult the v2.tauri.app docs, NOT the Tauri 1 docs.
---
# Tauri 2 Framework Skill

This project uses **Tauri 2.9.1** with features `["tray-icon", "protocol-asset"]`.
All Tauri code must target the 2.x API surface — Tauri 1 patterns (e.g., `tauri::Window`,
`app.get_window()`) do NOT exist in Tauri 2.

Reference: <https://v2.tauri.app/start/>

## Key API Differences from Tauri 1

| Concept | Tauri 1 | Tauri 2 |
|---------|---------|---------|
| Window access | `app.get_window("main")` | `app.get_webview_window("main")` |
| Window type | `Window<R>` | `WebviewWindow<R>` |
| Manager trait | `windows()` method | `webview_windows()` method |
| Menu system | `Menu::new()` with items | `Menu::with_items()`, `MenuItem::with_id()` |
| Tray icon | `SystemTray` separate builder | `TrayIconBuilder` integrated |
| State | `app.manage()` | `app.manage()` (same) |

## App Setup Pattern

```rust
tauri::Builder::default()
    .manage(AppState { ... })
    .invoke_handler(tauri::generate_handler![cmd1, cmd2])
    .plugin(plugin_a::init())
    .setup(|app| {
        // initialization
        Ok(())
    })
    .run(tauri::generate_context!())
    .expect("error running app");
```

## Commands (IPC)

`#[tauri::command]` functions run on a thread pool by default. Return `Result<T, String>`
for fallible commands. Use `tauri::State<T>` to access managed state:

```rust
#[tauri::command]
fn my_command(state: tauri::State<AppState>) -> Result<Settings, String> {
    let settings = state.settings.lock()
        .map_err(|e| format!("Lock failed: {}", e))?;
    Ok(settings.clone())
}
```

## State Management

- `app.manage(AppState { ... })` in the builder
- Access with `state: tauri::State<AppState>` in commands
- Access with `app.state::<AppState>()` from AppHandle (e.g., in event handlers)
- State must be `Send + Sync` — use `Mutex`/`AtomicBool` for interior mutability

## Tray Icon System

```rust
TrayIconBuilder::new()
    .icon(app.default_window_icon().unwrap().clone())
    .menu(&menu)                         // register context menu (on non-Windows)
    .show_menu_on_left_click(true)       // macOS/Linux only
    .on_menu_event(|app, event| { ... }) // menu item clicked
    .on_tray_icon_event(|tray, event| match event {
        TrayIconEvent::Click { button: MouseButton::Left, button_state: MouseButtonState::Down, .. } => { ... }
        TrayIconEvent::Click { button: MouseButton::Right, button_state: MouseButtonState::Down, .. } => { ... }
        TrayIconEvent::DoubleClick { button: MouseButton::Left, .. } => { ... }
        _ => {}
    })
    .build(app)?;
```

IMPORTANT: On Windows, registering `.menu(&menu)` causes the OS to show the menu on
both left and right clicks. To have fine-grained control, omit `.menu()` on Windows
and use raw Win32 popup menus instead (see the `show_tray_context_menu` function).

`MouseButtonState` has `Down` and `Up` variants (NOT `Pressed`/`Released`).
Always filter to `Down` to avoid double-triggering.

## Window Management

- `WebviewWindow` is the primary window type (NOT `Window`)
- `app.get_webview_window("main") -> Option<WebviewWindow>` is the standard accessor
- Methods: `.show()`, `.hide()`, `.unminimize()`, `.set_focus()`, `.is_visible()`, `.eval(js)`
- `WebviewWindow` does NOT implement `Into<Window>` or `AsRef<Window>` in Tauri 2.9.x
- The raw HWND can be obtained via `HasWindowHandle`:
  ```rust
  use raw_window_handle::HasWindowHandle;
  if let Ok(wh) = window.window_handle()
      && let raw_window_handle::RawWindowHandle::Win32(h) = wh.into() {
          let hwnd = HWND(h.hwnd.get() as *mut c_void);
  }
  ```

## Menu System

```rust
let show = MenuItem::with_id(app, "show", "Show", true, None::<&str>)?;
let settings = MenuItem::with_id(app, "settings", "Settings", true, None::<&str>)?;
let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
let menu = Menu::with_items(app, &[&show, &settings, &quit])?;
```

- `MenuItem::with_id(app, id, text, enabled, accelerator)` - create items
- `Menu::with_items(app, &[items])` - aggregate into a menu
- `.on_menu_event(|app, event| match event.id().as_ref() { "id" => ... })` - handle
- `menu.popup(&window)` requires a `&Window` — NOT available on `WebviewWindow` in Tauri 2.9.x

## Window Events

```rust
.on_window_event(|window, event| match event {
    WindowEvent::CloseRequested { api, .. } => { ... }
    WindowEvent::Focused(focused) => { ... }
    _ => {}
})
```

## Plugins Used

- `tauri_plugin_opener` — external URL handling
- `tauri_plugin_single_instance` — prevents duplicate instances, restores window
- `tauri_plugin_media` — media session integration for playback controls

## Configuration (`tauri.conf.json`)

- `app.windows[].label` — window identifier (must match `app.get_webview_window("label")`)
- `app.withGlobalTauri` — exposes `window.__TAURI__` to frontend
- `app.security.csp` — Content Security Policy
