use dioxus::prelude::*;

use crate::{
    models::{AppData, AppPreset},
    storage,
    ui::{preset_editor::PresetEditor, preset_list::PresetList},
    win::{
        monitor::resolve_position,
        picker::capture_on_focus_change,
        process::launch_app,
        window::{find_window_by_exe_path, find_window_by_pid, set_window_position},
    },
};

#[derive(Clone, PartialEq)]
enum EditorState {
    Closed,
    Open(AppPreset),
}

#[derive(Clone, PartialEq)]
enum PickerState {
    Idle,
    Waiting,
}

/// Launch a preset: try PID first, fall back to exe-path search (for Chromium-family apps).
fn launch_preset(preset: &AppPreset) -> Result<(), String> {
    let args = preset.arguments.as_deref();
    let pid = launch_app(&preset.executable_path, args)?;

    std::thread::sleep(std::time::Duration::from_millis(preset.delay_ms as u64));

    let hwnd = find_window_by_pid(pid, 3000)
        .or_else(|| find_window_by_exe_path(&preset.executable_path, 3000))
        .ok_or_else(|| format!("{} のウィンドウを検出できませんでした", preset.name))?;

    let (abs_x, abs_y) = resolve_position(preset.x, preset.y, preset.monitor_index);
    set_window_position(hwnd, abs_x, abs_y, preset.width, preset.height)
}

#[component]
pub fn App() -> Element {
    let mut app_data = use_signal(|| storage::load());
    let mut editor = use_signal(|| EditorState::Closed);
    let mut picker_state = use_signal(|| PickerState::Idle);
    let mut status_msg = use_signal(|| String::new());
    let mut delete_confirm = use_signal(|| Option::<String>::None);

    use_effect(move || {
        spawn(async move {
            let data = app_data();
            let targets: Vec<AppPreset> = data.presets
                .iter()
                .filter(|p| p.enabled)
                .cloned()
                .collect();
            if targets.is_empty() {
                return;
            }
            status_msg.set("起動中...".to_string());
            let result = tokio::task::spawn_blocking(move || {
                let mut errors = Vec::new();
                for preset in &targets {
                    if let Err(e) = launch_preset(preset) {
                        errors.push(e);
                    }
                }
                errors
            })
            .await;
            match result {
                Ok(errors) if errors.is_empty() => {
                    status_msg.set("全アプリを起動しました".to_string());
                }
                Ok(errors) => {
                    status_msg.set(errors.join(" / "));
                }
                Err(_) => {
                    status_msg.set("起動処理でエラーが発生しました".to_string());
                }
            }
        });
    });

    let mut save_data = move |data: AppData| {
        if let Err(e) = storage::save(&data) {
            status_msg.set(format!("保存失敗: {e}"));
        }
        app_data.set(data);
    };

    rsx! {
        style { {include_str!("../style.css")} }

        div { class: "container",
            div { class: "header",
                h1 { "Startup Layout" }
                div { class: "header-actions",

                    // ── 全部起動 ──────────────────────────────────────────
                    button {
                        class: "btn btn-primary",
                        onclick: move |_| {
                            let data = app_data();
                            let targets: Vec<AppPreset> = data.presets
                                .iter()
                                .filter(|p| p.enabled)
                                .cloned()
                                .collect();
                            if targets.is_empty() {
                                status_msg.set("有効なプリセットがありません".to_string());
                                return;
                            }
                            status_msg.set("起動中...".to_string());
                            let mut errors = Vec::new();
                            for preset in &targets {
                                if let Err(e) = launch_preset(preset) {
                                    errors.push(e);
                                }
                            }
                            if errors.is_empty() {
                                status_msg.set("全アプリを起動しました".to_string());
                            } else {
                                status_msg.set(errors.join(" / "));
                            }
                        },
                        "▶ 全部起動"
                    }

                    // ── 新規プリセット ────────────────────────────────────
                    button {
                        class: "btn",
                        onclick: move |_| {
                            let preset = AppPreset::new("新規プリセット".to_string(), String::new());
                            editor.set(EditorState::Open(preset));
                        },
                        "+ 新規プリセット"
                    }

                    // ── 既存ウィンドウから取得 ────────────────────────────
                    button {
                        class: "btn",
                        disabled: picker_state() != PickerState::Idle,
                        onclick: move |_| {
                            picker_state.set(PickerState::Waiting);
                            status_msg.set(
                                "取得対象のウィンドウをクリックしてください".to_string()
                            );
                            spawn(async move {
                                let captured = tokio::task::spawn_blocking(|| {
                                    capture_on_focus_change(30_000)
                                })
                                .await
                                .ok()
                                .flatten();
                                match captured {
                                    Some(c) => {
                                        let name = std::path::Path::new(&c.executable_path)
                                            .file_stem()
                                            .and_then(|s| s.to_str())
                                            .unwrap_or("New Preset")
                                            .to_string();
                                        let preset = AppPreset {
                                            id: uuid::Uuid::new_v4().to_string(),
                                            name,
                                            executable_path: c.executable_path,
                                            arguments: None,
                                            x: c.x,
                                            y: c.y,
                                            width: c.width,
                                            height: c.height,
                                            monitor_index: None,
                                            delay_ms: 1000,
                                            enabled: true,
                                        };
                                        editor.set(EditorState::Open(preset));
                                        status_msg.set(String::new());
                                    }
                                    None => {
                                        status_msg.set("タイムアウトしました".to_string());
                                    }
                                }
                                picker_state.set(PickerState::Idle);
                            });
                        },
                        if picker_state() == PickerState::Waiting {
                            "待機中..."
                        } else {
                            "既存ウィンドウから取得"
                        }
                    }
                }
            }

            if !status_msg().is_empty() {
                div { class: "status-bar",
                    span { "{status_msg}" }
                    button {
                        class: "btn-close",
                        onclick: move |_| status_msg.set(String::new()),
                        "✕"
                    }
                }
            }

            PresetList {
                presets: app_data().presets.clone(),
                on_toggle: move |id: String| {
                    let mut data = app_data();
                    if let Some(p) = data.presets.iter_mut().find(|p| p.id == id) {
                        p.enabled = !p.enabled;
                    }
                    save_data(data);
                },
                on_launch: move |id: String| {
                    let data = app_data();
                    if let Some(preset) = data.presets.iter().find(|p| p.id == id) {
                        match launch_preset(preset) {
                            Ok(()) => status_msg.set(format!("{} を起動しました", preset.name)),
                            Err(e) => status_msg.set(e),
                        }
                    }
                },
                on_edit: move |id: String| {
                    let data = app_data();
                    if let Some(preset) = data.presets.iter().find(|p| p.id == id) {
                        editor.set(EditorState::Open(preset.clone()));
                    }
                },
                on_delete: move |id: String| {
                    delete_confirm.set(Some(id));
                },
            }

            // 削除確認ダイアログ
            if let Some(ref del_id) = delete_confirm() {
                {
                    let del_id = del_id.clone();
                    let del_id2 = del_id.clone();
                    rsx! {
                        div {
                            class: "editor-overlay",
                            div { class: "confirm-dialog",
                                p { "このプリセットを削除しますか？" }
                                div { class: "editor-actions",
                                    button {
                                        class: "btn btn-danger",
                                        onclick: move |_| {
                                            let mut data = app_data();
                                            data.presets.retain(|p| p.id != del_id);
                                            save_data(data);
                                            delete_confirm.set(None);
                                        },
                                        "削除"
                                    }
                                    button {
                                        class: "btn",
                                        onclick: move |_| {
                                            let _ = del_id2.clone();
                                            delete_confirm.set(None);
                                        },
                                        "キャンセル"
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // プリセット編集ダイアログ
            if let EditorState::Open(ref preset) = editor() {
                PresetEditor {
                    preset: preset.clone(),
                    on_save: move |saved: AppPreset| {
                        let mut data = app_data();
                        if let Some(existing) = data.presets.iter_mut().find(|p| p.id == saved.id) {
                            *existing = saved;
                        } else {
                            data.presets.push(saved);
                        }
                        save_data(data);
                        editor.set(EditorState::Closed);
                    },
                    on_cancel: move |_| {
                        editor.set(EditorState::Closed);
                    },
                }
            }
        }
    }
}
