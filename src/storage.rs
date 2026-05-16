use std::path::PathBuf;

use crate::models::AppData;

pub fn data_path() -> PathBuf {
    let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push("startup-layout");
    path.push("presets.json");
    path
}

pub fn load() -> AppData {
    let path = data_path();
    if !path.exists() {
        return AppData::default();
    }
    let text = std::fs::read_to_string(&path).unwrap_or_default();
    serde_json::from_str(&text).unwrap_or_default()
}

pub fn save(data: &AppData) -> Result<(), String> {
    let path = data_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let text = serde_json::to_string_pretty(data).map_err(|e| e.to_string())?;
    std::fs::write(&path, text).map_err(|e| e.to_string())
}
