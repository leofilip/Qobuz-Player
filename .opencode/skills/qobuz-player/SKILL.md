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
- **Default branch:** `main`

## Project Structure

```
Qobuz-Player/
  .opencode/skills/qobuz-player/SKILL.md   — this file
  README.md                                — project documentation
  build-menu.ps1                           — PowerShell build helper (interactive + CLI)
  src-tauri/                               — all source code
    Cargo.toml                             — Rust package manifest
    tauri.conf.json                        — Tauri app config (window, CSP, bundle)
    build.rs                               — Tauri build script
    settings.html                          — Settings overlay UI (self-contained HTML)
    capabilities/default.json              — Tauri capability permissions
    icons/                                 — app icons + thumbnail toolbar icons
    inject/                                — JS/CSS files loaded via include_str!()
      titlebar.js                          — custom titlebar injection
      apply_theme.js                       — theme color application
      settings_overlay.js                  — settings overlay injection
      remove_overlay.js                    — settings overlay removal
      thumb_prev.js                        — previous-track DOM click selectors
      thumb_play.js                        — play/pause DOM click selectors
      thumb_next.js                        — next-track DOM click selectors
    src/
      main.rs                              — app bootstrap (~137 lines)
      interfaces.rs                        — traits + shared types (AppState)
      commands.rs                          — IPC commands + AppCommandDispatcher
      tray.rs                              — system tray icon + context menu
      theme.rs                             — ThemeRegistry impl + apply_theme()
      settings.rs                          — persistence, Win32 autostart
      thumbar.rs                           — taskbar thumbnail buttons (Win32 COM)
      window_manager.rs                    — minimize-to-tray hook (Win32 subclassing)
    target/                                — Rust build artifacts (gitignored)
```

## Architecture — Design Principles

The codebase follows **SOLID** principles with trait-based dependency inversion:

| Principle | Application |
|---|---|
| **SRP** | Each Rust source file has exactly one responsibility (commands, tray, theme, settings, thumbar, window_manager) |
| **OCP** | New commands → add to `commands.rs`; new themes → extend `DEFAULT_THEMES` in `theme.rs`; new tray items → add to `build_tray()`; no core files change |
| **LSP** | `WindowCommandDispatcher` / `ThemeRegistry` traits can have any number of implementations interchangeable at runtime |
| **ISP** | `WindowCommandDispatcher` has exactly 4 methods needed by Win32 callbacks; `ThemeRegistry` has `find()` only |
| **DIP** | `window_manager.rs` depends on `WindowCommandDispatcher` trait, never on concrete modules; `theme.rs` depends on `ThemeRegistry` trait |

## Architecture — Module by Module

### `src/main.rs` (~137 lines) — Entry Point

Thin bootstrap: loads settings, builds Tauri app with `AppState` managed state, registers 8 IPC commands in `invoke_handler`, and runs the setup hook (tray, thumbar init, HWND storage, window event handlers). All heavy lifting is delegated to focused modules.

**IPC commands** registered: `native_add_thumb_buttons`, `native_remove_thumb_buttons`, `get_settings`, `save_settings`, `minimize_window`, `open_settings_window`, `close_settings_window`, `apply_theme_from_string`.

### `src/interfaces.rs` — Shared Types & Traits

- **`AppState`**: `Mutex<Settings>` + `Mutex<Option<Box<dyn WindowCommandDispatcher>>>` — single source of managed state shape
- **`WindowCommandDispatcher`** trait: `toggle_minimize()`, `handle_prev_track()`, `handle_play_pause()`, `handle_next_track()` — the 4 actions the Win32 wndproc can trigger
- **`ThemeRegistry`** trait: `find(name) -> Option<ThemeColors>` — lookup by theme name
- **`ThemeColors`**: `bg`, `text`, `hover_bg`, `hover_text`, `border`
- **`ThumbButtonConfig`**: `id`, `icon_index`, `tooltip`, `flags`

### `src/commands.rs` — IPC + Dispatcher

- **8 `#[tauri::command]` functions** called from JS via `window.__TAURI__.core.invoke()`
- **`AppCommandDispatcher`** implements `WindowCommandDispatcher` — routes Win32 events to JS `click_selector()` calls (using `include_str!("../inject/thumb_*.js")` for DOM selector patterns)
- JS helpers (`click_selector`, `listener_exists`), settings commands, window commands

### `src/tray.rs` — System Tray

- **`build_tray()`**: Creates tray with 3 context-menu items (Show, Settings, Quit)
- Left-click toggles show/hide; double-click restores + re-adds thumbar
- Win32: `show_tray_context_menu()` spawns a raw popup menu to control click behavior
- Non-Windows: fallback tray context menu via TrayIcon::on_menu_event

### `src/theme.rs` — Theme Colors

- `DEFAULT_THEMES` static array with `Light`, `Dark`, `Midnight Blue`, `Emerald Green`, `Amber Glow`, `Rose` schemes
- `DefaultThemeRegistry` implements `ThemeRegistry`
- `apply_theme(window, name, &dyn ThemeRegistry)` — evaluates `include_str!("../inject/apply_theme.js")` with color interpolation
- Driven by `MutationObserver` on `<html>` class changes in the Qobuz page

### `src/settings.rs` — Configuration

- `Settings` struct + `LaunchMode` enum, persisted as JSON
- **Autostart module** (Windows-only via `winreg`): `HKCU\...\Run` with launch flags
- Non-Windows autostart returns `Err`

### `src/thumbar.rs` — Thumbnail Toolbar (Windows-only)

- `THUMB_BUTTONS: &[ThumbButtonConfig]` drives button creation — no magic indices
- `add_thumb_buttons()` iterates config, calls `add_thumb_buttons_native()` which uses `ITaskbarList3::ThumbBarAddButtons`
- Icon files searched in `TAURI_RESOURCE_DIR`, exe dir, relative paths
- `init_thumbar()` is a no-op placeholder; `remove_thumb_buttons()` is a no-op (buttons auto-remove on window destroy)
- Non-Windows: no-op stubs

### `src/window_manager.rs` — Minimize-to-Tray (Windows-only)

- Installs `SetWindowLongPtrW` (GWLP_WNDPROC) hook intercepting `WM_SYSCOMMAND` / `SC_MINIMIZE`
- Uses `with_dispatcher()` to look up `WindowCommandDispatcher` from `APP_HANDLE` static
- Routes `WM_COMMAND` with `THBN_CLICKED` to dispatcher's `handle_prev_track` / `handle_play_pause` / `handle_next_track`
- `WM_DESTROY` triggers `remove_window_manager()` cleanup

## Tauri Configuration (`tauri.conf.json`)

- **Window**: 1200x700 min, `decorations: false`, `titleBarStyle: "Overlay"`
- **Security**: CSP scoped to `play.qobuz.com` and `*.qobuz.com`; allows `data:`, `blob:`, `wss:`
- **Bundle**: MSI only, resources include `icons/win-thumbbar/*.ico` and `settings.html`
- **Capabilities**: window operations, opener plugin

## Crate Dependency Pattern

```toml
[dependencies]
tauri = { version = "2.9.1", features = ["tray-icon", "protocol-asset"] }
serde / serde_json = "1.0"
raw-window-handle = "0.6.2"
dirs = "5.0"

[target.'cfg(windows)'.dependencies]
windows = "0.62.2"           # Win32 APIs
winreg = "0.52"              # autostart registry
tauri-plugin-media = "0.1.1" # Windows media integration
```

Only `windows`, `winreg`, `tauri-plugin-media` are Windows-gated. All other crates compile on any target. This enables `cargo check` on Linux for CI / code review.

## Key Implementation Patterns

### 1. Trait-based Dependency Injection

The Win32 window procedure callback cannot accept closures or `dyn` trait objects directly (it's a `extern "system" fn`). The solution is a global `OnceLock<Mutex<Option<Box<dyn WindowCommandDispatcher>>>>` stored in `APP_HANDLE`, populated once during app setup. The wndproc calls `with_dispatcher()` to look it up.

### 2. JS Injection

JS strings live in `src-tauri/inject/*.js` files loaded via `include_str!()` at compile time. No raw string literals ≥ 5 lines in Rust source.

### 3. Platform Gating

`#[cfg(windows)]` on individual modules (`mod thumbar`, `mod window_manager`) and `use` statements. `#[cfg(not(windows))]` fallback for `show_tray_context_menu()`. Dependencies are conditionally compiled in `Cargo.toml`.

### 4. Settings Overlay (not a separate window)

Settings are a DOM overlay injected into the Qobuz page via JS. The Rust command parses `settings.html`, extracts `<body>` and `<style>`, and builds a JS eval string.

### 5. No-op Placeholders

Functions that are intentionally empty (`init_thumbar`, `remove_thumb_buttons`) carry a doc comment explaining why (lazy init, auto-cleanup by OS).

## Build & Development

### Commands

```sh
cargo tauri dev          # dev mode (debug build, hot-reload)
cargo tauri build        # release build + MSI installer
cargo check              # cross-platform compiles + Linux check target
.\build-menu.ps1         # interactive menu (Windows only)
```

### Build Helper Script (`build-menu.ps1`)

Options: dependency check, dev mode, quick dev (no version check), release build, set version. See `.\build-menu.ps1 -?` for CLI flags.

### Known Issues

- **WSL filesystem**: Build on Windows filesystem only.
- **WiX Toolset**: Required for MSI packaging.
- **tauri-plugin-media 0.1.1**: Linux compile is broken (MPRIS `str: RefArg`) — irrelevant as Windows-only target.

## Common Development Tasks

### Adding a new Tauri command
1. Add the function in `commands.rs` with `#[tauri::command]`
2. Register in `invoke_handler` array in `main.rs`
3. Call from JS via `window.__TAURI__.core.invoke('name', { args })`

### Adding a new action to the Win32 dispatcher
1. Add method to `WindowCommandDispatcher` trait in `interfaces.rs`
2. Implement in `AppCommandDispatcher` in `commands.rs`
3. Call from `window_manager.rs` `wndproc` via `with_dispatcher()`
4. Route tray menu items in `tray.rs`

### Adding a new theme
1. Add `ThemeColors` entry to `DEFAULT_THEMES` in `theme.rs`
2. Theme is auto-discovered via `DefaultThemeRegistry::find()`

### Modifying the titlebar
1. Edit `src-tauri/inject/titlebar.js`
2. Update theme colors in `apply_theme.js` if needed

### Adding thumbnail buttons
1. Add icon files to `icons/win-thumbbar/`
2. Add entry to `THUMB_BUTTONS` in `thumbar.rs`
3. Update resources in `tauri.conf.json`

## Testing

No automated tests. All testing is manual.

## Build Verification Checklist

Before committing:
- [ ] `cargo check` passes on Linux (or WSL)
- [ ] `cargo build` passes on Windows (native, not WSL)
- [ ] No warnings in either build
- [ ] JS files in `inject/` are syntactically valid
