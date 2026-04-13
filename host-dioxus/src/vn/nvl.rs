use dioxus::prelude::*;
use vn_runtime::command::TextMode;

use crate::render_state::RenderState;

/// NVL 全屏文本面板：累积显示多条对话。
///
/// 仅当 `text_mode == NVL` 时渲染。每条 entry 独立打字机。
#[component]
pub fn NvlPanel(render_state: Signal<RenderState>) -> Element {
    let rs = render_state.read();

    if rs.text_mode != TextMode::NVL || !rs.ui_visible {
        return rsx! {};
    }

    let entries = &rs.nvl_entries;
    if entries.is_empty() {
        return rsx! {};
    }

    rsx! {
        div { class: "vn-nvl",
            div { class: "vn-nvl__scroll",
                for (i, entry) in entries.iter().enumerate() {
                    {
                        let visible_text: String = entry.content.chars().take(entry.visible_chars).collect();
                        // "旁白" 视为旁白，不显示名称
                        let speaker = entry.speaker.as_deref()
                            .filter(|s| !s.is_empty() && *s != "旁白")
                            .map(|s| s.to_string());
                        rsx! {
                            div { key: "{i}", class: "vn-nvl__entry",
                                if let Some(name) = speaker {
                                    span { class: "vn-nvl__speaker", "{name}" }
                                }
                                span { class: "vn-nvl__text", "{visible_text}" }
                            }
                        }
                    }
                }
            }
        }
    }
}
