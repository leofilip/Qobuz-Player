use std::sync::atomic::AtomicBool;
use std::sync::Mutex;

use crate::settings::Settings;

/// Dispatches Win32 window-procedure commands without depending on concrete modules.
pub trait WindowCommandDispatcher: Send + Sync {
    fn handle_minimize(&self, app: &tauri::AppHandle, minimize_to_tray: bool) -> bool;
    fn handle_show(&self, app: &tauri::AppHandle);
    fn handle_open_settings(&self, app: &tauri::AppHandle);
    fn handle_quit(&self, app: &tauri::AppHandle);
    fn handle_prev_track(&self, app: &tauri::AppHandle);
    fn handle_play_pause(&self, app: &tauri::AppHandle);
    fn handle_next_track(&self, app: &tauri::AppHandle);
}

/// Theme color scheme for the custom titlebar.
#[derive(Debug, Clone)]
pub struct ThemeColors {
    pub name: &'static str,
    pub bg_color: &'static str,
    pub text_color: &'static str,
}

/// Registry of available themes.
pub trait ThemeRegistry: Send + Sync {
    fn get(&self, name: &str) -> Option<&ThemeColors>;
}

/// Configuration for a single thumbnail toolbar button.
#[derive(Debug, Clone)]
pub struct ThumbButtonConfig {
    pub id: u16,
    pub tooltip: &'static str,
    pub icon_dev_path: &'static str,
    pub icon_release_path: &'static str,
}

/// Application state managed by Tauri.
pub struct AppState {
    pub settings: Mutex<Settings>,
    pub tray_hidden: AtomicBool,
    pub dispatcher: Mutex<Option<Box<dyn WindowCommandDispatcher>>>,
    pub theme_registry: Box<dyn ThemeRegistry>,
}
