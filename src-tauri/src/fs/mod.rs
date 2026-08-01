use serde::Serialize;
use std::fs;
use tauri::Manager;

#[derive(Serialize)]
pub struct FsEntry {
    name: String,
    path: String,
    is_dir: bool,
    size: u64,
    modified: String,
}

#[tauri::command]
pub fn fs_list_dir(path: String) -> Result<Vec<FsEntry>, String> {
    let entries = fs::read_dir(&path).map_err(|e| e.to_string())?;
    let mut result = Vec::new();
    for entry in entries.flatten() {
        let metadata = entry.metadata().map_err(|e| e.to_string())?;
        let name = entry.file_name().to_string_lossy().to_string();
        let full_path = entry.path().to_string_lossy().to_string();
        let size = if metadata.is_file() { metadata.len() } else { 0 };
        let modified = metadata.modified()
            .ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| format_duration(d.as_secs()))
            .unwrap_or_default();
        result.push(FsEntry { name, path: full_path, is_dir: metadata.is_dir(), size, modified });
    }
    Ok(result)
}

#[tauri::command]
pub fn fs_home_dir(app: tauri::AppHandle) -> Result<String, String> {
    app.path().home_dir()
        .map(|p| p.to_string_lossy().to_string())
        .map_err(|e| e.to_string())
}

fn format_duration(secs: u64) -> String {
    let days = secs / 86400;
    if days > 0 { return format!("{}d ago", days); }
    let hours = secs / 3600;
    if hours > 0 { return format!("{}h ago", hours); }
    format!("{}m ago", secs / 60)
}
