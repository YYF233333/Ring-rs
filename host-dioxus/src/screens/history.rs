use dioxus::prelude::*;

use crate::render_state::{HostScreen, RenderState};
use crate::state::AppState;

/// 历史 screen：可滚动的对话历史列表
#[component]
pub fn HistoryScreen(render_state: Signal<RenderState>) -> Element {
    let app_state = use_context::<AppState>();

    let history = {
        let inner = app_state.inner.lock().unwrap();
        inner.history.clone()
    };

    let app_back = app_state.clone();

    rsx! {
        div { class: "screen-history",
            div { class: "screen-history__header",
                h2 { "History" }
                button {
                    class: "screen-history__back-btn",
                    onclick: move |_| {
                        if let Ok(mut inner) = app_back.inner.lock() {
                            inner.set_host_screen(HostScreen::InGame);
                        }
                    },
                    "Back"
                }
            }

            div { class: "screen-history__scroll",
                for (i, entry) in history.iter().enumerate() {
                    div { key: "{i}", class: "screen-history__entry",
                        if let Some(ref speaker) = entry.speaker {
                            span { class: "screen-history__speaker", "{speaker}" }
                        }
                        span { class: "screen-history__text", "{entry.text}" }
                    }
                }

                if history.is_empty() {
                    div { class: "screen-history__empty", "No history yet." }
                }
            }
        }
    }
}
