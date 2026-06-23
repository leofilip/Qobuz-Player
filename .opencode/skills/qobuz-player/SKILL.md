---
name: qobuz-player
description: >
  Project-scale reference for the Qobuz-Player desktop app. Use this skill for
  ANY work on this project — adding features, fixing bugs, refactoring, code review,
  building/packaging, understanding architecture, or modifying settings/themes/tray/titlebar.
  This skill MUST be used whenever the user mentions Qobuz, Tauri, the player, thumbar,
  tray icons, window management, settings overlay, custom titlebar, or the build script.
---

# Qobuz-Player Project Reference

## Overview

Qobuz-Player is a **Windows desktop application** that wraps the Qobuz web player (`https://play.qobuz.com/`) inside a lightweight native window using **Tauri 2** (Rust) and **WebView2** (Edge Chromium). Its key differentiator from a standard PWA is **minimize/close to system tray** plus **Windows taskbar thumbnail media controls**.

- **Author:** Leonardo Calé (leofilip)
- **Version:** 0.5.0
- **Identifier:** `com.leo.qobuz-player`
- **Current branch:** `18-feat-introduce-themes`
- **Default branch:** `main`

## Project Structure

```
Qobuz-Player/
  .opencode/skills/qobuz-player/SKILL.md   — this file
  README.md                                — project documentation
  build-menu.ps1                           — PowerShell build helper (interactive + CLI)
  src-tauri/                               — all source code
    Cargo.toml                             — Rust package manifest
    Cargo.lock                             — locked dependencies
    tauri.conf.json                        — Tauri app config (window, CSP, bundle)
    build.rs                               — Tauri build script
    settings.html                          — Settings overlay UI (self-contained HTML)
    capabilities/default.json              — Tauri capability permissions
    gen/schemas/                           — auto-generated JSON schemas
    icons/                                 — app icons + thumbnail toolbar icons
    src/
      main.rs                              — app entry, tray, titlebar injection, commands
      settings.rs                          — persistence, struct, Windows autostart
      thumbar.rs                           — taskbar thumbnail buttons (Win32 COM)
      window_manager.rs                    — minimize-to-tray hook (Win32 subclassing)
    target/                                — Rust build artifacts (gitignored)
```

## Technology Stack

| Technology | Purpose |
|---|---|
| **Rust** (edition 2024) | Core application logic |
| **Tauri 2.9.1** | Desktop framework |
| **WebView2** (Edge Chromium) | Embedded browser |
| **HTML/CSS/JS** (vanilla) | Settings overlay UI |
| **Windows API** (`windows` crate v0.62.2) | Win32 interop (tray, thumbar, subclassing) |
| **PowerShell** | Build helper script |

## Architecture — Module by Module

### `src/main.rs` (~558 lines) — Entry Point & Orchestration

The heart of the app. Does all of the following in its `main()` function:

1. **Loads settings** via `settings::Settings::load()` at startup
2. **Builds Tauri app** with:
   - `AppState` (Mutex-wrapped `Settings`) managed as Tauri state
   - **9 IPC commands** registered via `tauri::generate_handler![]`:
     - `native_add_thumb_buttons` / `native_remove_thumb_buttons` — toggle thumbnail toolbar
     - `get_settings` / `save_settings` — settings CRUD (save also handles autostart registry)
     - `minimize_window` — hide or minimize depending on `minimize_to_tray` setting
     - `open_settings_window` / `close_settings_window` — inject/remove settings overlay via JS eval
     - `apply_theme_from_string` — update titlebar colors for dark/light theme
3. **Setup hook** (runs after app builds):
   - Sets `AppUserModelID` for taskbar grouping (differs by debug/release)
   - Creates **system tray** with 3 items: Show, Settings, Quit
   - Tray left-click shows menu; double-click shows window + restores thumbar
   - **Injects custom titlebar** into Qobuz page via JS `window.eval()`:
     - Injects a 32px-high fixed titlebar with minimize/maximize/close/settings buttons
     - Patches Qobuz CSS (`.ui-app { margin-top: 32px }`, bottom panel fix)
     - Sets up `MutationObserver` on `<html class>` to detect Qobuz theme changes (dark/light)
   - Initializes thumbar + window manager + stores HWND
4. **Window event handler**: intercepts `CloseRequested` → hides to tray or quits based on `close_to_tray`

**Key pattern**: Titlebar and settings overlay are injected by evaluating JavaScript strings into the Qobuz webview. The JS uses `window.__TAURI__` APIs (`invoke`, `getCurrentWindow`) from the Tauri JS bridge (enabled by `withGlobalTauri: true`).

### `src/settings.rs` (~132 lines) — Configuration

- **`Settings` struct**: `close_to_tray`, `minimize_to_tray`, `launch_on_login`, `launch_mode`
- **`LaunchMode` enum**: `Restored`, `Minimized`, `MinimizedToTray`, `Maximized`
- Storage: JSON at `{config_dir}/qobuz-player/settings.json` (via `dirs::config_dir()`)
- Defaults: `close_to_tray: true`, others false, `launch_mode: Restored`
- **Autostart module** (Windows-only via `winreg`): writes to `HKCU\Software\Microsoft\Windows\CurrentVersion\Run` with optional `--minimized`, `--minimized-to-tray`, `--maximized` flags
- Non-Windows autostart returns an error

### `src/thumbar.rs` (~329 lines) — Thumbnail Toolbar

Implements Windows 7+ taskbar thumbnail buttons (Previous, Play/Pause, Next).

- **Icon loading**: Searches multiple paths for `.ico` files (resource dir, exe dir, relative paths)
- **COM interface**: Uses `ITaskbarList3` → `ThumbBarAddButtons` with 3 `THUMBBUTTON`s (IDs 100, 101, 102)
- **Window subclass**: Installs a Win32 `SetWindowLongPtrW` hook on `WM_COMMAND` with `THBN_CLICKED` notification
- **On click**: Executes JavaScript in the Qobuz webview to find and click DOM elements via multiple CSS selector fallbacks (e.g., `button[aria-label*="revious"]`, `.pct-player-previous`, etc.)
- Non-Windows builds use no-op stubs

### `src/window_manager.rs` (~128 lines) — Minimize-to-Tray Hook

- Installs a Win32 window subclass intercepting `WM_SYSCOMMAND` with `SC_MINIMIZE`
- If `minimize_to_tray` is enabled, hides the window instead of minimizing
- Uses `SetWindowLongPtrW` / `CallWindowProcW` to chain with the original WNDPROC
- Non-Windows builds use no-op stubs

### `settings.html` (~447 lines) — Settings Overlay UI

A self-contained HTML page (inline styles + scripts) injected as a DOM overlay into the Qobuz page via `main.rs`'s `open_settings_window` command. Not a separate window.

- **Controls**: Close to Tray (checkbox), Minimize to Tray (checkbox), Launch on Login (checkbox with launch mode radio sub-options)
- **Theme-aware**: Detects Qobuz theme from parent document's `<html>` class, applies CSS variables
- **Communicates** with Rust backend via `window.__TAURI__.core.invoke()`
- Injected by parsing `<body>` and `<style>` from `settings.html`, escaping backticks, and calling `window.eval()`

## Tauri Configuration (`tauri.conf.json`)

- **Window**: 1200x700 min, no native decorations (`decorations: false`), `titleBarStyle: "Overlay"`
- **Security**: CSP scoped to `play.qobuz.com` and `*.qobuz.com`; allows `data:`, `blob:`, `wss:`
- **Bundle**: MSI only, resources include `icons/win-thumbbar/*.ico` and `settings.html`
- **Capabilities**: window operations (minimize, maximize, show, hide, close, set-focus), opener plugin

## Dependencies (`Cargo.toml`)

| Crate | Version | Purpose |
|---|---|---|
| `tauri` | 2.9.1 | Core framework (features: `tray-icon`, `protocol-asset`) |
| `tauri-plugin-opener` | 2.5.2 | Open URLs/files |
| `tauri-plugin-media` | 0.1.1 | Media session integration |
| `tauri-plugin-single-instance` | 2.3.6 | Prevent multiple instances |
| `serde` / `serde_json` | 1.0 | Settings serialization |
| `base64` | 0.22 | Base64 encoding |
| `windows` | 0.62.2 | Win32 API (Foundation, COM, Shell, UI, Media) |
| `raw-window-handle` | 0.6.2 | Cross-platform window handle access |
| `dirs` | 5.0 | Platform config directories |
| `winreg` | 0.52 | Windows registry (autostart) |
| `tauri-build` | 2.5.1 | Build dependency |

**Release profile**: optimized for size (`opt-level = "z"`), LTO, single codegen unit, stripped, `panic = "abort"`.

## Build & Development

### Commands

```sh
cargo tauri dev          # dev mode (debug build, hot-reload)
cargo tauri build        # release build + MSI installer
```

### Build Helper Script (`build-menu.ps1`)

Interactive menu + CLI mode. Supports: dependency check, dev mode (with/without version check), release build, open installer folder, set version (updates both `Cargo.toml` and `tauri.conf.json`).

Usage:
```powershell
.\build-menu.ps1            # interactive menu
.\build-menu.ps1 d          # dev mode with version check
.\build-menu.ps1 q          # quick dev (no version check)
.\build-menu.ps1 b 0.5.1    # build release with version update
.\build-menu.ps1 v 0.5.1    # set version
```

### Versioning

Version must match across `src-tauri/Cargo.toml` and `src-tauri/tauri.conf.json`. The `.last-build-version` file tracks the last built version to warn if unchanged.

### Environment Variables

- `TAURI_RESOURCE_DIR` — used by thumbar to locate icon files at runtime
- `CARGO_INCREMENTAL=0` — fix incremental compilation lock errors (WSL)
- `CARGO_TARGET_DIR` — redirect build output (avoid WSL filesystem issues)

### Known Issues

- **WSL filesystem**: Running inside WSL causes incremental compilation errors. Build on Windows filesystem only.
- **WiX Toolset**: Required for MSI packaging. Install from https://github.com/wixtoolset/wix/releases/

## Key Implementation Patterns

### 1. JavaScript Injection

The app communicates with the Qobuz web page by evaluating JavaScript strings via `window.eval()`. This is used for:

- Injecting the custom titlebar (CSS + DOM + event handlers)
- Injecting/removing the settings overlay
- Applying theme colors to titlebar elements
- Clicking Qobuz player controls (for thumbnail toolbar)

The Tauri JS bridge is available via `window.__TAURI__` (enabled by `withGlobalTauri: true`).

### 2. Win32 Window Subclassing

Both `thumbar.rs` and `window_manager.rs` use `SetWindowLongPtrW` to subclass the main window's WNDPROC. This is a low-level Win32 technique where you replace the window procedure pointer with your own function, chain calls to the original via `CallWindowProcW`, and clean up by restoring the original pointer.

**Important**: When the app quits, cleanup must restore the original WNDPROC and destroy loaded icons to avoid leaks.

### 3. Settings Overlay (not a separate window)

Settings are not a separate Tauri window — they're a DOM overlay injected into the Qobuz page. The Rust command parses `settings.html`, extracts `<body>` and `<style>`, escapes backticks/template literals, and builds a JS string that creates a fixed-position overlay div.

### 4. Platform-Gated Modules

Windows-only code is separated into `windows_impl` modules with `#[cfg(target_os = "windows")]` guards. Non-Windows builds get no-op stubs via `pub use` re-exports. This is used in both `thumbar.rs` and `window_manager.rs`.

## Getting Started for Development

1. Install Rust via `rustup`
2. Install Tauri CLI: `cargo install tauri-cli`
3. Install WiX Toolset (for MSI builds)
4. Clone the repo to a **Windows filesystem** (not WSL)
5. Run `.\build-menu.ps1` or `cargo tauri dev`

## Common Development Tasks

### Adding a new Tauri command
1. Add the function in `main.rs` with `#[tauri::command]` attribute
2. Register it in the `invoke_handler` array
3. Call it from the frontend JS via `window.__TAURI__.core.invoke('command_name', { args })`

### Changing settings
1. Update the `Settings` struct in `settings.rs`
2. Add UI control in `settings.html`
3. Handle save/load in the settings overlay JS

### Modifying the titlebar
1. Edit the `init_script` string in `main.rs` (the `injectTitlebar` function)
2. Update CSS classes or HTML structure there
3. Update `apply_theme` for any new themed elements

### Adding thumbnail buttons
1. Add icon files to `icons/win-thumbbar/`
2. Update `load_icons()` in `thumbar.rs` to load the new file
3. Add a `THUMBBUTTON` entry in `add_thumb_buttons_native()`
4. Add click handler in the `wndproc` function
5. Update resources in `tauri.conf.json`

### Theme support
- Uses `MutationObserver` on `<html>` element's `class` attribute
- Calls `apply_theme_from_string` IPC command when dark/light class changes
- The `apply_theme` function updates titlebar background and button colors

## Testing

There is **no automated testing** in this project. No `#[cfg(test)]` blocks, no test files, no CI pipeline. Testing is manual.
