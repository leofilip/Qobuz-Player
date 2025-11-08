use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub close_to_tray: bool,
    pub minimize_to_tray: bool,
    pub launch_on_login: bool,
    pub launch_mode: LaunchMode,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum LaunchMode {
    Restored,
    Minimized,
    MinimizedToTray,
    Maximized,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            close_to_tray: true,
            minimize_to_tray: false,
            launch_on_login: false,
            launch_mode: LaunchMode::Restored,
        }
    }
}

impl Settings {
    fn get_config_path() -> Result<PathBuf, String> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| "Could not determine config directory".to_string())?;
        let app_config = config_dir.join("qobuz-player");
        if !app_config.exists() {
            fs::create_dir_all(&app_config)
                .map_err(|e| format!("Failed to create config directory: {}", e))?;
        }
        Ok(app_config.join("settings.json"))
    }

    pub fn load() -> Self {
        match Self::get_config_path() {
            Ok(path) => {
                if path.exists() {
                    match fs::read_to_string(&path) {
                        Ok(contents) => {
                            serde_json::from_str::<Settings>(&contents).unwrap_or_default()
                        }
                        Err(_) => Settings::default(),
                    }
                } else {
                    Settings::default()
                }
            }
            Err(_) => Settings::default(),
        }
    }

    pub fn save(&self) -> Result<(), String> {
        let path = Self::get_config_path()?;
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize settings: {}", e))?;
        fs::write(&path, json)
            .map_err(|e| format!("Failed to write settings file: {}", e))?;
        Ok(())
    }
}

#[cfg(target_os = "windows")]
pub mod autostart {
    use super::LaunchMode;
    use winreg::enums::*;
    use winreg::RegKey;

    const RUN_KEY_PATH: &str = "Software\\Microsoft\\Windows\\CurrentVersion\\Run";
    const APP_NAME: &str = "QobuzPlayer";

    pub fn enable(launch_mode: &LaunchMode) -> Result<(), String> {
        let exe_path = std::env::current_exe()
            .map_err(|e| format!("Failed to get executable path: {}", e))?;
        
        let mut command = format!("\"{}\"", exe_path.display());
        
        match launch_mode {
            LaunchMode::Minimized => command.push_str(" --minimized"),
            LaunchMode::MinimizedToTray => command.push_str(" --minimized-to-tray"),
            LaunchMode::Maximized => command.push_str(" --maximized"),
            LaunchMode::Restored => {}
        }

        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let run_key = hkcu
            .open_subkey_with_flags(RUN_KEY_PATH, KEY_WRITE)
            .map_err(|e| format!("Failed to open registry key: {}", e))?;

        run_key
            .set_value(APP_NAME, &command)
            .map_err(|e| format!("Failed to set registry value: {}", e))?;

        Ok(())
    }

    pub fn disable() -> Result<(), String> {
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let run_key = hkcu
            .open_subkey_with_flags(RUN_KEY_PATH, KEY_WRITE)
            .map_err(|e| format!("Failed to open registry key: {}", e))?;

        run_key
            .delete_value(APP_NAME)
            .map_err(|e| format!("Failed to delete registry value: {}", e))?;

        Ok(())
    }
}

#[cfg(not(target_os = "windows"))]
pub mod autostart {
    use super::LaunchMode;

    pub fn enable(_launch_mode: &LaunchMode) -> Result<(), String> {
        Err("Autostart is only supported on Windows".to_string())
    }

    pub fn disable() -> Result<(), String> {
        Err("Autostart is only supported on Windows".to_string())
    }
}
