use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct AppPreset {
    pub id: String,
    pub name: String,
    pub executable_path: String,
    pub arguments: Option<String>,

    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    pub monitor_index: Option<usize>,

    pub delay_ms: u32,
    pub enabled: bool,
}

impl AppPreset {
    pub fn new(name: String, executable_path: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            executable_path,
            arguments: None,
            x: 0,
            y: 0,
            width: 1280,
            height: 720,
            monitor_index: None,
            delay_ms: 1000,
            enabled: true,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct AppData {
    pub presets: Vec<AppPreset>,
}
