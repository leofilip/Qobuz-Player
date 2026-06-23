use crate::interfaces::{ThemeColors, ThemeRegistry};

static DEFAULT_THEMES: &[ThemeColors] = &[
    ThemeColors {
        name: "dark",
        bg_color: "#181818",
        text_color: "#ffffff",
    },
    ThemeColors {
        name: "light",
        bg_color: "#FFFFFF",
        text_color: "#242424",
    },
];

pub struct DefaultThemeRegistry;

impl ThemeRegistry for DefaultThemeRegistry {
    fn get(&self, name: &str) -> Option<&ThemeColors> {
        DEFAULT_THEMES.iter().find(|t| t.name == name)
    }
}

pub fn apply_theme(window: &tauri::WebviewWindow, theme_name: &str, registry: &dyn ThemeRegistry) -> Result<(), String> {
    let colors = registry.get(theme_name).unwrap_or_else(|| registry.get("dark").unwrap());

    let template = include_str!("../inject/apply_theme.js");
    let script = template
        .replacen("{}", colors.bg_color, 1)
        .replacen("{}", colors.text_color, 1);

    window
        .eval(&script)
        .map_err(|e| format!("Failed to apply theme: {}", e))?;
    Ok(())
}
