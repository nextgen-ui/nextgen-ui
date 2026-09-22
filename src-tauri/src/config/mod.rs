use serde::{Deserialize, Serialize};
use std::fs;
use tauri::Manager;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AppConfig {
    pub theme: String,
    pub shell: String,
    pub font_size: u32,
    pub stats_width: u32,
    #[serde(default = "default_true")]
    pub audio_enabled: bool,
    #[serde(default = "default_volume")]
    pub audio_volume: f32,
    /// Look up public IP and location from a third-party service. Off unless
    /// the user turns it on: it sends the user's IP to that service on every
    /// launch, and puts their city and IP on a screen people screenshot.
    #[serde(default)]
    pub geo_lookup: bool,
}

fn default_true() -> bool { true }
fn default_volume() -> f32 { 0.6 }

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            theme: "default".into(),
            shell: "".into(),
            font_size: 14,
            stats_width: 230,
            audio_enabled: true,
            audio_volume: 0.6,
            geo_lookup: false,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_is_valid() {
        let cfg = AppConfig::default();
        assert_eq!(cfg.theme, "default");
        assert_eq!(cfg.font_size, 14);
        assert!(cfg.audio_enabled);
        assert!((cfg.audio_volume - 0.6).abs() < 0.001);
        assert!(!cfg.geo_lookup, "geolocation must be opt-in");
    }

    #[test]
    fn config_from_before_geo_setting_keeps_geo_off() {
        let old = "theme = \"default\"\nshell = \"\"\nfontSize = 14\nstatsWidth = 230\n";
        let cfg: AppConfig = toml::from_str(old).expect("old config still parses");
        assert!(!cfg.geo_lookup);
    }

    #[test]
    fn toml_serialization_and_deserialization() {
        let cfg = AppConfig::default();
        let toml_str = toml::to_string(&cfg).expect("serialize toml");
        let parsed: AppConfig = toml::from_str(&toml_str).expect("deserialize toml");
        assert_eq!(cfg, parsed);
    }
}
