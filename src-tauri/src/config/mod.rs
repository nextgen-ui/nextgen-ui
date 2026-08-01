use serde::{Deserialize, Serialize};
use std::fs;
use tauri::Manager;

#[derive(Serialize, Deserialize, Clone)]
pub struct AppConfig {
    pub theme: String,
    pub shell: String,
    pub font_size: u32,
    pub stats_width: u32,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            theme: "default".into(),
            shell: "".into(),
            font_size: 14,
            stats_width: 230,
        }
    }
}

fn config_path(app: &tauri::AppHandle) -> std::path::PathBuf {
    app.path().app_config_dir()
        .unwrap_or_else(|_| std::path::PathBuf::from("."))
        .join("config.toml")
}

#[tauri::command]
pub fn get_config(app: tauri::AppHandle) -> AppConfig {
    let path = config_path(&app);
    fs::read_to_string(&path)
        .ok()
        .and_then(|s| toml::from_str(&s).ok())
        .unwrap_or_default()
}

#[tauri::command]
pub fn save_config(app: tauri::AppHandle, config: AppConfig) -> Result<(), String> {
    let path = config_path(&app);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let toml = toml::to_string(&config).map_err(|e| e.to_string())?;
    fs::write(&path, toml).map_err(|e| e.to_string())?;
    Ok(())
}
