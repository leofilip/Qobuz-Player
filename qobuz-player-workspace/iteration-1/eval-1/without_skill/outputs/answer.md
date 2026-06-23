# Adding "Always on Top" Setting

## Files to change (3 files)

### 1. `src-tauri/src/settings.rs`

**Add field** to the `Settings` struct (after `launch_mode`):
```rust
    pub always_on_top: bool,
```

**Add default** in `impl Default`:
```rust
            always_on_top: false,
```

### 2. `src-tauri/settings.html`

**Add UI element** inside one of the `<div class="setting-group">` blocks (before the buttons section at line 349). Insert after the launch-mode sub-settings `</div>` on line 346:
```html
            <div class="setting-item">
                <div class="setting-header">
                    <label for="always-on-top">Always on Top</label>
                    <div class="checkbox-wrapper">
                        <input type="checkbox" id="always-on-top">
                        <span class="slider"></span>
                    </div>
                </div>
                <div class="setting-description">
                    Keep the Qobuz Player window on top of all other windows.
                </div>
            </div>
```

**Add JavaScript** in the `<script>` block:

In `loadSettings()` (around line 385), add:
```js
                alwaysOnTopCheckbox.checked = settings.always_on_top;
```

In `saveBtn.click` handler's settings object (around line 414), add:
```js
                always_on_top: alwaysOnTopCheckbox.checked,
```

Declare the variable at line ~370:
```js
        const alwaysOnTopCheckbox = document.getElementById('always-on-top');
```

### 3. `src-tauri/src/main.rs`

**In `save_settings` command** (around line 56), after saving the settings to disk and updating the app state, apply the always-on-top state to the window. Add after line 68 (`*app_settings = settings.clone();`):
```rust
    if let Some(window) = app.get_webview_window("main") {
        window.set_always_on_top(settings.always_on_top)
            .map_err(|e| format!("Failed to set always on top: {}", e))?;
    }
```

**On app startup** (in `setup` closure), after the settings are loaded and the window is available (around line 353 where `window` is obtained), check the setting and apply it. Add after the `thumbar::set_stored_hwnd` call (around line 527):
```rust
                        if app_settings.always_on_top {
                            let _ = window.set_always_on_top(true);
                        }
```

Note: `app_settings` is originally loaded at line 245, so in the `setup` closure you'd access it via `app.state::<AppState>()` or capture it. The simplest approach is to capture `app_settings` before `tauri::Builder::default()` or to read it from the state inside setup after the window is created. Since `app_settings` is moved into `.manage()`, you'll need to check the flag before that move or re-read from state. Recommended: store `always_on_top` in a local before `.manage()`:

```rust
    let initial_always_on_top = app_settings.always_on_top;
```

Then in `setup`, after the window handle code:
```rust
                        if initial_always_on_top {
                            let _ = window.set_always_on_top(true);
                        }
```

## Summary

| File | Change |
|------|--------|
| `settings.rs` | Add `always_on_top: bool` field + default `false` |
| `settings.html` | Add toggle checkbox + description in UI, load/save it in JS |
| `main.rs` | Apply `window.set_always_on_top()` on save and on initial launch |
