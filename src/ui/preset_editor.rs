use dioxus::prelude::*;

use crate::{models::AppPreset, win::monitor::enumerate_monitors};

#[derive(Props, Clone, PartialEq)]
pub struct PresetEditorProps {
    pub preset: AppPreset,
    pub on_save: EventHandler<AppPreset>,
    pub on_cancel: EventHandler<()>,
}

#[component]
pub fn PresetEditor(props: PresetEditorProps) -> Element {
    let mut name = use_signal(|| props.preset.name.clone());
    let mut executable_path = use_signal(|| props.preset.executable_path.clone());
    let mut arguments = use_signal(|| props.preset.arguments.clone().unwrap_or_default());
    let mut x = use_signal(|| props.preset.x.to_string());
    let mut y = use_signal(|| props.preset.y.to_string());
    let mut width = use_signal(|| props.preset.width.to_string());
    let mut height = use_signal(|| props.preset.height.to_string());
    let mut delay_ms = use_signal(|| props.preset.delay_ms.to_string());
    let mut enabled = use_signal(|| props.preset.enabled);
    let mut monitor_index = use_signal(|| {
        props
            .preset
            .monitor_index
            .map(|i| i.to_string())
            .unwrap_or_else(|| "none".to_string())
    });
    let mut error_msg = use_signal(|| String::new());

    let monitors = enumerate_monitors();
    let monitor_options: Vec<(String, String)> = monitors
        .iter()
        .map(|m| {
            let suffix = if m.is_primary { " [Primary]" } else { "" };
            let label = format!(
                "Monitor {} ({}x{}){suffix}",
                m.index + 1,
                m.width,
                m.height
            );
            (m.index.to_string(), label)
        })
        .collect();

    let on_save = props.on_save.clone();
    let on_cancel = props.on_cancel.clone();
    let preset_id = props.preset.id.clone();

    rsx! {
        div {
            class: "editor-overlay",
            onclick: move |_| on_cancel.call(()),

            div {
                class: "editor-dialog",
                onclick: move |e| e.stop_propagation(),

                h2 { "プリセット編集" }

                if !error_msg().is_empty() {
                    div { class: "error-banner", "{error_msg}" }
                }

                div { class: "form-row",
                    label { "表示名" }
                    input {
                        value: "{name}",
                        oninput: move |e| name.set(e.value()),
                        placeholder: "例: Slack"
                    }
                }

                div { class: "form-row",
                    label { "実行ファイルパス" }
                    input {
                        value: "{executable_path}",
                        oninput: move |e| executable_path.set(e.value()),
                        placeholder: r"例: C:\Program Files\App\app.exe",
                        style: "flex: 1;"
                    }
                }

                div { class: "form-row",
                    label { "起動引数 (任意)" }
                    input {
                        value: "{arguments}",
                        oninput: move |e| arguments.set(e.value()),
                        placeholder: "--flag value"
                    }
                }

                div { class: "form-grid",
                    div { class: "form-row",
                        label { "X" }
                        input {
                            r#type: "number",
                            value: "{x}",
                            oninput: move |e| x.set(e.value()),
                        }
                    }
                    div { class: "form-row",
                        label { "Y" }
                        input {
                            r#type: "number",
                            value: "{y}",
                            oninput: move |e| y.set(e.value()),
                        }
                    }
                    div { class: "form-row",
                        label { "Width" }
                        input {
                            r#type: "number",
                            value: "{width}",
                            oninput: move |e| width.set(e.value()),
                        }
                    }
                    div { class: "form-row",
                        label { "Height" }
                        input {
                            r#type: "number",
                            value: "{height}",
                            oninput: move |e| height.set(e.value()),
                        }
                    }
                }

                div { class: "form-row",
                    label { "モニタ" }
                    select {
                        value: "{monitor_index}",
                        onchange: move |e| monitor_index.set(e.value()),
                        option { value: "none", "座標通り" }
                        for (val, label) in &monitor_options {
                            option {
                                value: "{val}",
                                "{label}"
                            }
                        }
                    }
                }

                div { class: "form-row",
                    label { "遅延 (ms)" }
                    input {
                        r#type: "number",
                        value: "{delay_ms}",
                        oninput: move |e| delay_ms.set(e.value()),
                    }
                }

                div { class: "form-row form-row-check",
                    label {
                        input {
                            r#type: "checkbox",
                            checked: enabled(),
                            onchange: move |e| enabled.set(e.checked()),
                        }
                        " 一括起動に含める"
                    }
                }

                div { class: "editor-actions",
                    button {
                        class: "btn btn-primary",
                        onclick: move |_| {
                            let name_val = name();
                            if name_val.trim().is_empty() {
                                error_msg.set("表示名を入力してください".to_string());
                                return;
                            }
                            let path_val = executable_path();
                            if path_val.trim().is_empty() {
                                error_msg.set("実行ファイルパスを入力してください".to_string());
                                return;
                            }

                            let mon_idx = {
                                let v = monitor_index();
                                if v == "none" { None } else { v.parse::<usize>().ok() }
                            };

                            let args_val = arguments();
                            let saved = AppPreset {
                                id: preset_id.clone(),
                                name: name_val,
                                executable_path: path_val,
                                arguments: if args_val.trim().is_empty() { None } else { Some(args_val) },
                                x: x().parse::<i32>().unwrap_or(0),
                                y: y().parse::<i32>().unwrap_or(0),
                                width: width().parse::<i32>().unwrap_or(1280),
                                height: height().parse::<i32>().unwrap_or(720),
                                monitor_index: mon_idx,
                                delay_ms: delay_ms().parse::<u32>().unwrap_or(1000),
                                enabled: enabled(),
                            };
                            on_save.call(saved);
                        },
                        "保存"
                    }
                    button {
                        class: "btn",
                        onclick: move |_| on_cancel.call(()),
                        "キャンセル"
                    }
                }
            }
        }
    }
}
