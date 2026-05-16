use dioxus::prelude::*;

use crate::{
    models::AppPreset,
    win::monitor::enumerate_monitors,
};

#[derive(Props, Clone, PartialEq)]
pub struct PresetListProps {
    pub presets: Vec<AppPreset>,
    pub on_toggle: EventHandler<String>,
    pub on_launch: EventHandler<String>,
    pub on_edit: EventHandler<String>,
    pub on_delete: EventHandler<String>,
}

#[component]
pub fn PresetList(props: PresetListProps) -> Element {
    let monitors = enumerate_monitors();

    rsx! {
        div { class: "preset-list",
            if props.presets.is_empty() {
                div { class: "empty-state",
                    p { "プリセットがありません。" }
                    p { "「+ 新規プリセット」または「既存ウィンドウから取得」で追加してください。" }
                }
            }
            for preset in &props.presets {
                PresetCard {
                    key: "{preset.id}",
                    preset: preset.clone(),
                    monitors: monitors.clone(),
                    on_toggle: props.on_toggle.clone(),
                    on_launch: props.on_launch.clone(),
                    on_edit: props.on_edit.clone(),
                    on_delete: props.on_delete.clone(),
                }
            }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
struct PresetCardProps {
    preset: AppPreset,
    monitors: Vec<crate::win::monitor::MonitorInfo>,
    on_toggle: EventHandler<String>,
    on_launch: EventHandler<String>,
    on_edit: EventHandler<String>,
    on_delete: EventHandler<String>,
}

#[component]
fn PresetCard(props: PresetCardProps) -> Element {
    let preset = &props.preset;
    let id = preset.id.clone();
    let id2 = id.clone();
    let id3 = id.clone();
    let id4 = id.clone();

    let monitor_label = match preset.monitor_index {
        Some(idx) => {
            if let Some(m) = props.monitors.get(idx) {
                format!("Monitor {}", m.index + 1)
            } else {
                format!("Monitor {}", idx + 1)
            }
        }
        None => "座標通り".to_string(),
    };

    let exe_name = std::path::Path::new(&preset.executable_path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(&preset.executable_path)
        .to_string();

    rsx! {
        div {
            class: if preset.enabled { "preset-card" } else { "preset-card disabled" },

            div { class: "preset-card-top",
                input {
                    r#type: "checkbox",
                    checked: preset.enabled,
                    onchange: move |_| props.on_toggle.call(id.clone()),
                }
                div { class: "preset-info",
                    span { class: "preset-name", "{preset.name}" }
                    span { class: "preset-geometry",
                        "{preset.width}x{preset.height}+{preset.x}+{preset.y} ({monitor_label})"
                    }
                    span { class: "preset-exe", "{exe_name}" }
                }
                div { class: "preset-actions",
                    button {
                        class: "btn btn-icon",
                        title: "個別起動",
                        onclick: move |_| props.on_launch.call(id2.clone()),
                        "▶"
                    }
                    button {
                        class: "btn btn-icon",
                        title: "編集",
                        onclick: move |_| props.on_edit.call(id3.clone()),
                        "編集"
                    }
                    button {
                        class: "btn btn-icon btn-danger",
                        title: "削除",
                        onclick: move |_| props.on_delete.call(id4.clone()),
                        "削除"
                    }
                }
            }
        }
    }
}
