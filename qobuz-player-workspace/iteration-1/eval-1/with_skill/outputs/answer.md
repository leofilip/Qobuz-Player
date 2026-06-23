# Always-on-Top Setting — Implementation Plan

Three files need to change: `src/settings.rs`, `src/main.rs`, `settings.html`.

---

## 1. `src-tauri/src/settings.rs` — Add field + default

**Struct** (line 6-11): add `pub always_on_top: bool,`

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub close_to_tray: bool,
    pub minimize_to_tray: bool,
    pub launch_on_login: bool,
    pub launch_mode: LaunchMode,
    pub always_on_top: bool,
}
```

**Default impl** (line 24-29): add `always_on_top: false,`

```rust
impl Default for Settings {
    fn default() -> Self {
        Settings {
            close_to_tray: true,
            minimize_to_tray: false,
            launch_on_login: false,
            launch_mode: LaunchMode::Restored,
            always_on_top: false,
        }
    }
}
```

---

## 2. `src-tauri/src/main.rs` — Apply on save + apply at startup

### 2a. Capture setting before moving into state (line ~245):

```rust
fn main() {
    let app_settings = settings::Settings::load();
    let always_on_top = app_settings.always_on_top;   // NEW

    tauri::Builder::default()
        .manage(AppState {
            settings: Mutex::new(app_settings),
        })
```

### 2b. Apply at startup in `setup` closure, after `if let Some(window)` block (around line 530, before `Ok(())`):

```rust
if always_on_top {
    let _ = window.set_always_on_top(true);
}
```

Insert before the closing `Ok(())` at line 533 (after the HWND block ends, around line 531):

```rust
            // … existing HWND setup …

            if always_on_top {
                let _ = window.set_always_on_top(true);
            }

            Ok(())
```

### 2c. Update `save_settings` command (line 55) — accept `AppHandle`, apply flag:

Change signature from:
```rust
fn save_settings(settings: settings::Settings, state: tauri::State<AppState>) -> Result<(), String> {
```
to:
```rust
fn save_settings(app: tauri::AppHandle, settings: settings::Settings, state: tauri::State<AppState>) -> Result<(), String> {
```

Add after the autostart block (before `*app_settings = settings.clone();`):

```rust
    // Apply always-on-top
    if let Some(window) = app.get_webview_window("main") {
        window.set_always_on_top(settings.always_on_top)
            .map_err(|e| format!("Failed to set always on top: {}", e))?;
    }
```

Full updated function:

```rust
#[tauri::command]
fn save_settings(app: tauri::AppHandle, settings: settings::Settings, state: tauri::State<AppState>) -> Result<(), String> {
    settings.save()?;
    
    let mut app_settings = state.settings.lock()
        .map_err(|e| format!("Failed to lock settings: {}", e))?;
    
    if settings.launch_on_login {
        settings::autostart::enable(&settings.launch_mode)?;
    } else {
        let _ = settings::autostart::disable();
    }

    if let Some(window) = app.get_webview_window("main") {
        window.set_always_on_top(settings.always_on_top)
            .map_err(|e| format!("Failed to set always on top: {}", e))?;
    }
    
    *app_settings = settings.clone();
    
    Ok(())
}
```

---

## 3. `src-tauri/settings.html` — Add UI control + wire JS

### 3a. Add a new setting group after the minimize-to-tray group (after line 306).

Insert after the `</div>` closing the "Minimize to Tray" setting-group (line 306) and before the "Launch on Login" group (line 308):

```html
        <div class="setting-group">
            <div class="setting-item">
                <div class="setting-header">
                    <label for="always-on-top">Always on Top</label>
                    <div class="checkbox-wrapper">
                        <input type="checkbox" id="always-on-top">
                        <span class="slider"></span>
                    </div>
                </div>
                <div class="setting-description">
                    Keep the Qobuz Player window always on top of other windows.
                </div>
            </div>
        </div>
```

### 3b. Wire the checkbox in JS:

Add the reference after `launchOnLoginCheckbox` (around line 369):

```javascript
const alwaysOnTopCheckbox = document.getElementById('always-on-top');
```

In `loadSettings()`, add after `launchOnLoginCheckbox.checked = ...` (around line 386):

```javascript
alwaysOnTopCheckbox.checked = settings.always_on_top;
```

In the `saveBtn` click handler's `settings` object (around line 413-418), add:

```javascript
const settings = {
    close_to_tray: closeToTrayCheckbox.checked,
    minimize_to_tray: minimizeToTrayCheckbox.checked,
    launch_on_login: launchOnLoginCheckbox.checked,
    launch_mode: selectedLaunchMode,
    always_on_top: alwaysOnTopCheckbox.checked   // NEW
};
```

---

## Summary of all changes

| File | Change type | Detail |
|------|-------------|--------|
| `src-tauri/src/settings.rs:8` | Add field | `pub always_on_top: bool` |
| `src-tauri/src/settings.rs:28` | Add default | `always_on_top: false` |
| `src-tauri/src/main.rs:246` | Capture value | `let always_on_top = app_settings.always_on_top;` |
| `src-tauri/src/main.rs:531` (approx) | Apply at startup | `if always_on_top { window.set_always_on_top(true) }` |
| `src-tauri/src/main.rs:56` | Add param | `app: tauri::AppHandle` to `save_settings` |
| `src-tauri/src/main.rs:62-67` (approx) | Apply on save | `window.set_always_on_top(settings.always_on_top)` |
| `src-tauri/settings.html:308-320` (approx) | New HTML block | Setting group with checkbox + description |
| `src-tauri/settings.html:371` (approx) | New JS var | `const alwaysOnTopCheckbox = ...` |
| `src-tauri/settings.html:387` (approx) | Load value | `alwaysOnTopCheckbox.checked = settings.always_on_top;` |
| `src-tauri/settings.html:418` (approx) | Save value | `always_on_top: alwaysOnTopCheckbox.checked` |
