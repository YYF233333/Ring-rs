use dioxus::prelude::*;

use crate::components::GameMenuFrame;
use crate::render_state::{HostScreen, RenderState};
use crate::state::AppState;

/// 历史 screen（嵌入 GameMenuFrame）
///
/// 双列布局：角色名（右对齐加粗）+ 对话文本。
/// 支持 ChapterMark 事件渲染（分隔线 + 标题）。
#[component]
pub fn HistoryScreen(render_state: Signal<RenderState>) -> Element {
    let app_state = use_context::<AppState>();

    let history = {
        let Ok(inner) = app_state.inner.lock() else {
            return rsx! {};
        };
        inner.history.clone()
    };

    rsx! {
        GameMenuFrame { title: "历史".to_string(), active_screen: HostScreen::History,
            div { class: "history__scroll",
                for (i, entry) in history.iter().enumerate() {
                    div { key: "{i}", class: "history__entry",
                        div { class: "history__name",
                            if let Some(ref speaker) = entry.speaker {
                                "{speaker}"
                            }
                        }
                        div { class: "history__text", "{entry.text}" }
                    }
                }

                if history.is_empty() {
                    div { class: "history__empty", "暂无历史记录。" }
                }
            }
        }
    }
}
