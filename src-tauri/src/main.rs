#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::{
    Manager, WindowEvent,
    menu::{Menu, MenuItem},
    tray::{MouseButton, TrayIconBuilder, TrayIconEvent},
};
use raw_window_handle::HasWindowHandle;
use std::sync::Mutex;

mod thumbar;
mod settings;
mod window_manager;

pub struct AppState {
    settings: Mutex<settings::Settings>,
}

#[tauri::command]
fn native_add_thumb_buttons() {
    thumbar::add_thumb_buttons();
}

#[tauri::command]
fn native_remove_thumb_buttons() {
    thumbar::remove_thumb_buttons();
}

#[tauri::command]
fn get_settings(state: tauri::State<AppState>) -> Result<settings::Settings, String> {
    let settings = state.settings.lock()
        .map_err(|e| format!("Failed to lock settings: {}", e))?;
    Ok(settings.clone())
}

#[tauri::command]
fn minimize_window(app: tauri::AppHandle, state: tauri::State<AppState>) -> Result<(), String> {
    let minimize_to_tray = {
        let settings = state.settings.lock()
            .map_err(|e| format!("Failed to lock settings: {}", e))?;
        settings.minimize_to_tray
    };
    
    if let Some(window) = app.get_webview_window("main") {
        if minimize_to_tray {
            window.hide().map_err(|e| format!("Failed to hide window: {}", e))?;
        } else {
            window.minimize().map_err(|e| format!("Failed to minimize window: {}", e))?;
        }
    }
    
    Ok(())
}

#[tauri::command]
fn save_settings(settings: settings::Settings, state: tauri::State<AppState>) -> Result<(), String> {
    settings.save()?;
    
    let mut app_settings = state.settings.lock()
        .map_err(|e| format!("Failed to lock settings: {}", e))?;
    
    if settings.launch_on_login {
        settings::autostart::enable(&settings.launch_mode)?;
    } else {
        let _ = settings::autostart::disable();
    }
    
    *app_settings = settings.clone();
    
    Ok(())
}

#[tauri::command]
fn apply_theme_from_string(app: tauri::AppHandle, theme: String) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        apply_theme(&window, &theme)?;
    }
    Ok(())
}

fn apply_theme(window: &tauri::WebviewWindow, theme: &str) -> Result<(), String> {
    let (bg_color, text_color) = match theme {
        "dark" => ("#181818", "#ffffff"),
        "light" => ("#FFFFFF", "#242424"),
        _ => ("#181818", "#ffffff")
    };
    
    let script = format!(r#"
        (function() {{
            const titlebar = document.getElementById('custom-titlebar');
            if (titlebar) {{
                titlebar.style.setProperty('background', '{}', 'important');
            }}
            
            const buttons = document.querySelectorAll('.titlebar-button');
            if (buttons.length > 0) {{
                buttons.forEach(btn => {{
                    btn.style.setProperty('color', '{}', 'important');
                }});
            }}
        }})();
    "#, bg_color, text_color);
    
    window.eval(&script).map_err(|e| format!("Failed to apply theme: {}", e))?;
    Ok(())
}

#[tauri::command]
fn close_settings_window(app: tauri::AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        window.eval(r#"
            (function() {
                const overlay = document.getElementById('qobuz-settings-overlay');
                if (overlay) document.body.removeChild(overlay);
                const backBtn = document.getElementById('qobuz-settings-back-btn');
                if (backBtn) document.body.removeChild(backBtn);
            })();
        "#).map_err(|e| format!("Failed to remove settings overlay: {}", e))?;
    }
    Ok(())
}

#[tauri::command]
fn open_settings_window(app: tauri::AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.set_focus();
        
        let settings_html = include_str!("../settings.html");
        
        let body_start = settings_html.find("<body>").unwrap_or(0) + 6;
        let body_end = settings_html.find("</body>").unwrap_or(settings_html.len());
        let body_content = &settings_html[body_start..body_end];
        
        let style_start = settings_html.find("<style>").unwrap_or(0);
        let style_end = settings_html.find("</style>").unwrap_or(0) + 8;
        let styles = if style_start > 0 && style_end > 8 {
            &settings_html[style_start..style_end]
        } else {
            ""
        };
        
        let body_escaped = body_content.replace("`", "\\`").replace("${", "\\${");
        let styles_escaped = styles.replace("`", "\\`").replace("${", "\\${");
        let js_code = format!(r#"
            (function() {{
                // Check if settings overlay already exists
                if (document.getElementById('qobuz-settings-overlay')) {{
                    return;
                }}
                
                // Detect current theme
                const isDark = document.documentElement.classList.contains('theme-dark');
                const isLight = document.documentElement.classList.contains('theme-light');
                const currentTheme = isLight ? 'light' : 'dark';
                
                const themeColors = {{
                    dark: {{ bg: '#1a1a1a', text: '#e0e0e0', btnBg: '#333', btnHover: '#444' }},
                    light: {{ bg: '#f5f5f5', text: '#242424', btnBg: '#e0e0e0', btnHover: '#d0d0d0' }}
                }};
                const colors = themeColors[currentTheme];
                
                // Create overlay container
                const overlay = document.createElement('div');
                overlay.id = 'qobuz-settings-overlay';
                overlay.style.cssText = `
                    position: fixed;
                    top: 0;
                    left: 0;
                    right: 0;
                    bottom: 0;
                    background: ${{colors.bg}};
                    z-index: 9999999;
                    display: flex;
                    flex-direction: column;
                    margin-top: 32px;
                    overflow-y: auto;
                `;
                
                // Create close button
                const closeBtn = document.createElement('button');
                closeBtn.id = 'qobuz-settings-back-btn';
                closeBtn.innerHTML = '← Back to Player';
                closeBtn.style.cssText = `
                    position: fixed;
                    top: 44px;
                    left: 12px;
                    padding: 8px 16px;
                    background: ${{colors.btnBg}};
                    color: ${{colors.text}};
                    border: none;
                    border-radius: 6px;
                    cursor: pointer;
                    font-size: 14px;
                    font-weight: 500;
                    z-index: 10000000;
                    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, Cantarell, sans-serif;
                `;
                closeBtn.onmouseover = function() {{ this.style.background = colors.btnHover; }};
                closeBtn.onmouseout = function() {{ this.style.background = colors.btnBg; }};
                closeBtn.onclick = function() {{
                    const overlay = document.getElementById('qobuz-settings-overlay');
                    if (overlay) {{
                        document.body.removeChild(overlay);
                    }}
                    const btn = document.getElementById('qobuz-settings-back-btn');
                    if (btn) {{
                        document.body.removeChild(btn);
                    }}
                }};
                
                // Inject styles
                const styleElement = document.createElement('div');
                styleElement.innerHTML = `{}`;
                overlay.appendChild(styleElement);
                
                // Inject body content
                const contentDiv = document.createElement('div');
                contentDiv.innerHTML = `{}`;
                overlay.appendChild(contentDiv);
                
                document.body.appendChild(overlay);
                document.body.appendChild(closeBtn);
                
                // Find and execute any script tags after DOM is ready, wrapped in IIFE
                const scripts = contentDiv.querySelectorAll('script');
                scripts.forEach(script => {{
                    const newScript = document.createElement('script');
                    // Wrap the script in an IIFE to avoid variable conflicts
                    newScript.textContent = '(function(){{' + script.textContent + '}})();';
                    document.body.appendChild(newScript);
                }});
            }})();
        "#, styles_escaped, body_escaped);
        
        window.eval(&js_code).map_err(|e| format!("Failed to inject settings overlay: {}", e))?;
        
        Ok(())
    } else {
        Err("Main window not found".to_string())
    }
}

fn main() {
    let app_settings = settings::Settings::load();
    
    tauri::Builder::default()
        .manage(AppState {
            settings: Mutex::new(app_settings),
        })
        .invoke_handler(tauri::generate_handler![
            native_add_thumb_buttons, 
            native_remove_thumb_buttons,
            get_settings,
            save_settings,
            close_settings_window,
            open_settings_window,
            minimize_window,
            apply_theme_from_string
        ])
    .plugin(tauri_plugin_media::init())
    .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
        if let Some(window) = app.get_webview_window("main") {
            let _ = window.unminimize();
            let _ = window.show();
            let _ = window.set_focus();
        }
    }))
        .setup(|app| {
            #[cfg(target_os = "windows")]
            {
                use windows::core::PCWSTR;
                use windows::Win32::UI::Shell::SetCurrentProcessExplicitAppUserModelID;
                
                #[cfg(debug_assertions)]
                let app_id = "com.leo.qobuz-player.dev";
                #[cfg(not(debug_assertions))]
                let app_id = "com.leo.qobuz-player";
                
                let id = app_id.encode_utf16().chain(std::iter::once(0)).collect::<Vec<u16>>();
                let pcw = PCWSTR(id.as_ptr());
                let _ = unsafe { SetCurrentProcessExplicitAppUserModelID(pcw) };
            }
            
            let show = MenuItem::with_id(app, "show", "Show", true, None::<&str>)?;
            let settings = MenuItem::with_id(app, "settings", "Settings", true, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show, &settings, &quit])?;

            TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .show_menu_on_left_click(true)
                .on_menu_event(|app, event| match event.id().as_ref() {
                    "quit" => {
                        thumbar::cleanup_thumbar();
                        window_manager::remove_minimize_hook();
                        std::process::exit(0);
                    }
                    "show" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.unminimize();
                            let _ = window.show();
                            let _ = window.set_focus();
                            
                            #[cfg(target_os = "windows")]
                            if let Ok(wh) = window.window_handle()
                                && let raw_window_handle::RawWindowHandle::Win32(h) = wh.into() {
                                    thumbar::set_stored_hwnd(h);
                                    thumbar::add_thumb_buttons();
                                }
                        }
                    }
                    "settings" => {
                        let _ = open_settings_window(app.clone());
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::DoubleClick {
                        button: MouseButton::Left,
                        ..
                    } = event {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.eval(r#"
                                (function() {
                                    const overlay = document.getElementById('qobuz-settings-overlay');
                                    if (overlay) document.body.removeChild(overlay);
                                    const backBtn = document.getElementById('qobuz-settings-back-btn');
                                    if (backBtn) document.body.removeChild(backBtn);
                                })();
                            "#);
                            
                            let _ = window.unminimize();
                            let _ = window.show();
                            let _ = window.set_focus();
                            
                            #[cfg(target_os = "windows")]
                            if let Ok(wh) = window.window_handle()
                                && let raw_window_handle::RawWindowHandle::Win32(h) = wh.into() {
                                    thumbar::set_stored_hwnd(h);
                                    thumbar::add_thumb_buttons();
                                }
                        }
                    }
                })
                .build(app)?;
            
            thumbar::init_thumbar(app, "main");
            window_manager::init_window_manager(app);
            
            if let Some(window) = app.get_webview_window("main") {
                let init_script = r#"
                    function injectTitlebar() {
                        const existing = document.getElementById('custom-titlebar');
                        if (existing) existing.remove();
                        
                        const style = document.createElement('style');
                        style.id = 'custom-titlebar-style';
                        style.textContent = `
                            #custom-titlebar {
                                position: fixed !important;
                                top: 0 !important;
                                left: 0 !important;
                                right: 0 !important;
                                width: 100% !important;
                                height: 32px !important;
                                background: #181818 !important;
                                display: flex !important;
                                align-items: center !important;
                                justify-content: flex-end !important;
                                z-index: 2147483647 !important;
                                -webkit-app-region: drag;
                                user-select: none;
                                -webkit-user-select: none;
                                font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, Cantarell, sans-serif !important;
                            }
                            #custom-titlebar-controls {
                                display: flex !important;
                                align-items: center !important;
                                -webkit-app-region: no-drag;
                                height: 100% !important;
                            }
                            .titlebar-button {
                                width: 46px !important;
                                height: 32px !important;
                                display: flex !important;
                                align-items: center !important;
                                justify-content: center !important;
                                background: transparent !important;
                                border: none !important;
                                color: #e0e0e0 !important;
                                cursor: pointer !important;
                                transition: background-color 0.15s !important;
                                padding: 0 !important;
                                pointer-events: auto !important;
                                -webkit-app-region: no-drag;
                            }
                            .titlebar-button:hover {
                                background-color: rgba(255, 255, 255, 0.1) !important;
                            }
                            .titlebar-button.close:hover {
                                background-color: #e81123 !important;
                            }
                            .titlebar-button svg {
                                fill: currentColor !important;
                                opacity: 0.9 !important;
                            }
                            .titlebar-button:hover svg {
                                opacity: 1 !important;
                            }
                            .titlebar-button.settings-btn {
                                margin-right: 8px !important;
                            }
                            .titlebar-button.settings-btn:hover {
                                background-color: rgba(0, 102, 204, 0.3) !important;
                            }
                            
                            /* Offset the Qobuz app element */
                            .ui-app {
                                margin-top: 32px !important;
                            }
                            
                            /* Fix bottom panel cropping */
                            .ui-layout-001--panel-outer-bottom {
                                margin-bottom: 32px !important;
                            }
                        `;
                        document.head.appendChild(style);
                        
                        const titlebar = document.createElement('div');
                        titlebar.id = 'custom-titlebar';
                        titlebar.innerHTML = `
                            <div id="custom-titlebar-controls">
                                <button class="titlebar-button settings-btn" id="titlebar-settings" title="Settings">
                                    <svg width="16" height="16" viewBox="0 0 512 512">
                                        <path d="M496,293.984c9.031-0.703,16-8.25,16-17.297v-41.375c0-9.063-6.969-16.594-16-17.313l-54.828-4.281 c-3.484-0.266-6.484-2.453-7.828-5.688l-18.031-43.516c-1.344-3.219-0.781-6.906,1.5-9.547l35.75-41.813 c5.875-6.891,5.5-17.141-0.922-23.547l-29.25-29.25c-6.406-6.406-16.672-6.813-23.547-0.922l-41.813,35.75 c-2.641,2.266-6.344,2.844-9.547,1.516l-43.531-18.047c-3.219-1.328-5.422-4.375-5.703-7.828l-4.266-54.813 C293.281,6.969,285.75,0,276.688,0h-41.375c-9.063,0-16.594,6.969-17.297,16.016l-4.281,54.813c-0.266,3.469-2.469,6.5-5.688,7.828 l-43.531,18.047c-3.219,1.328-6.906,0.75-9.563-1.516l-41.797-35.75c-6.875-5.891-17.125-5.484-23.547,0.922l-29.25,29.25 c-6.406,6.406-6.797,16.656-0.922,23.547l35.75,41.813c2.25,2.641,2.844,6.328,1.5,9.547l-18.031,43.516 c-1.313,3.234-4.359,5.422-7.813,5.688L16,218c-9.031,0.719-16,8.25-16,17.313v41.359c0,9.063,6.969,16.609,16,17.313l54.844,4.266 c3.453,0.281,6.5,2.484,7.813,5.703l18.031,43.516c1.344,3.219,0.75,6.922-1.5,9.563l-35.75,41.813 c-5.875,6.875-5.484,17.125,0.922,23.547l29.25,29.25c6.422,6.406,16.672,6.797,23.547,0.906l41.797-35.75 c2.656-2.25,6.344-2.844,9.563-1.5l43.531,18.031c3.219,1.344,5.422,4.359,5.688,7.844l4.281,54.813 c0.703,9.031,8.234,16.016,17.297,16.016h41.375c9.063,0,16.594-6.984,17.297-16.016l4.266-54.813 c0.281-3.484,2.484-6.5,5.703-7.844l43.531-18.031c3.203-1.344,6.922-0.75,9.547,1.5l41.813,35.75 c6.875,5.891,17.141,5.5,23.547-0.906l29.25-29.25c6.422-6.422,6.797-16.672,0.922-23.547l-35.75-41.813 c-2.25-2.641-2.844-6.344-1.5-9.563l18.031-43.516c1.344-3.219,4.344-5.422,7.828-5.703L496,293.984z M256,342.516 c-23.109,0-44.844-9-61.188-25.328c-16.344-16.359-25.344-38.078-25.344-61.203c0-23.109,9-44.844,25.344-61.172 c16.344-16.359,38.078-25.344,61.188-25.344c23.125,0,44.844,8.984,61.188,25.344c16.344,16.328,25.344,38.063,25.344,61.172 c0,23.125-9,44.844-25.344,61.203C300.844,333.516,279.125,342.516,256,342.516z"/>
                                    </svg>
                                </button>
                                <button class="titlebar-button minimize" id="titlebar-minimize" title="Minimize">
                                    <svg width="10" height="1" viewBox="0 0 10 1">
                                        <path d="M0 0h10v1H0z"/>
                                    </svg>
                                </button>
                                <button class="titlebar-button maximize" id="titlebar-maximize" title="Maximize">
                                    <svg width="10" height="10" viewBox="0 0 10 10">
                                        <path d="M0 0v10h10V0H0zm1 1h8v8H1V1z"/>
                                    </svg>
                                </button>
                                <button class="titlebar-button close" id="titlebar-close" title="Close">
                                    <svg width="10" height="10" viewBox="0 0 10 10">
                                        <path d="M0 0l10 10M10 0L0 10" stroke="currentColor" stroke-width="1"/>
                                    </svg>
                                </button>
                            </div>
                        `;
                        
                        document.body.appendChild(titlebar);
                        
                        setTimeout(() => {
                            if (typeof window.__TAURI__ === 'undefined') return;
                            
                            const { getCurrentWindow } = window.__TAURI__.window;
                            const { invoke } = window.__TAURI__.core;
                            const currentWindow = getCurrentWindow();
                            
                            document.getElementById('titlebar-minimize').onclick = () => invoke('minimize_window');
                            document.getElementById('titlebar-maximize').onclick = async () => {
                                if (await currentWindow.isMaximized()) {
                                    currentWindow.unmaximize();
                                } else {
                                    currentWindow.maximize();
                                }
                            };
                            document.getElementById('titlebar-close').onclick = () => currentWindow.close();
                            document.getElementById('titlebar-settings').onclick = () => invoke('open_settings_window');
                            
                            function detectAndApplyTheme() {
                                const html = document.documentElement;
                                const isDark = html.classList.contains('theme-dark');
                                const isLight = html.classList.contains('theme-light');
                                
                                if (isDark) {
                                    invoke('apply_theme_from_string', { theme: 'dark' });
                                } else if (isLight) {
                                    invoke('apply_theme_from_string', { theme: 'light' });
                                } else {
                                    invoke('apply_theme_from_string', { theme: 'dark' });
                                }
                            }
                            
                            setTimeout(detectAndApplyTheme, 1000);
                            
                            const observer = new MutationObserver((mutations) => {
                                mutations.forEach((mutation) => {
                                    if (mutation.type === 'attributes' && mutation.attributeName === 'class') {
                                        detectAndApplyTheme();
                                    }
                                });
                            });
                            
                            observer.observe(document.documentElement, {
                                attributes: true,
                                attributeFilter: ['class']
                            });
                        }, 300);
                    }
                    
                    function waitAndInject() {
                        if (document.readyState === 'loading') {
                            document.addEventListener('DOMContentLoaded', () => setTimeout(injectTitlebar, 500));
                        } else {
                            setTimeout(injectTitlebar, 500);
                        }
                        
                        window.addEventListener('load', () => setTimeout(injectTitlebar, 500));
                    }
                    
                    waitAndInject();
                "#;
                
                let _ = window.eval(init_script);
                
                if let Ok(wh) = window.window_handle()
                    && let raw_window_handle::RawWindowHandle::Win32(h) = wh.into() {
                        thumbar::set_stored_hwnd(h);
                        window_manager::set_main_window_hwnd(h.hwnd.get());
                        window_manager::install_minimize_hook();
                    }
            }
            
            Ok(())
        })
        .on_window_event(|window, event| {
            if window.label() == "main"
                && let WindowEvent::CloseRequested { api, .. } = event {
                    let app = window.app_handle();
                    let state = app.state::<AppState>();
                    
                    let close_to_tray = if let Ok(settings) = state.settings.lock() {
                        settings.close_to_tray
                    } else {
                        true
                    };
                    
                    if close_to_tray {
                        api.prevent_close();
                        let _ = window.hide();
                    } else {
                        thumbar::cleanup_thumbar();
                        window_manager::remove_minimize_hook();
                    }
                }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
