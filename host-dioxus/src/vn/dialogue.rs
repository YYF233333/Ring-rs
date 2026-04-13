use dioxus::prelude::*;

use vn_runtime::command::TextMode;

use crate::render_state::RenderState;

/// ADV 对话框组件：显示说话人 + 打字机文本 + 推进指示器。
///
/// 打字机效果由后端 `process_tick` 驱动 `visible_chars` 递增，
/// 前端只负责截取对应长度的文本渲染。
/// NVL 模式下不渲染（NVL 有独立的全屏面板）。
#[component]
pub fn DialogueBox(render_state: Signal<RenderState>) -> Element {
    let rs = render_state.read();

    // NVL 模式、不可见、或无对话时不渲染
    if rs.text_mode == TextMode::NVL || !rs.ui_visible {
        return rsx! {};
    }
    let dialogue = match &rs.dialogue {
        Some(d) => d,
        None => return rsx! {},
    };

    let speaker = dialogue.speaker.clone();
    let visible_text: String = dialogue
        .content
        .chars()
        .take(dialogue.visible_chars)
        .collect();
    let is_complete = dialogue.is_complete;

    rsx! {
        div { class: "vn-dialogue",
            // 说话人名牌
            if let Some(name) = speaker {
                div { class: "vn-dialogue__name", "{name}" }
            }

            // 文本区域
            div { class: "vn-dialogue__text",
                "{visible_text}"

                // 推进指示器（打字完成后闪烁）
                if is_complete {
                    span { class: "vn-dialogue__advance", "\u{25BC}" }
                }
            }
        }
    }
}
