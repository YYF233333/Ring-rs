use dioxus::prelude::*;

use crate::render_state::RenderState;
use crate::state::AppState;

/// 选项面板：显示选择支并处理用户选择。
#[component]
pub fn ChoicePanel(render_state: Signal<RenderState>) -> Element {
    let app_state = use_context::<AppState>();
    let rs = render_state.read();

    let choices_state = match &rs.choices {
        Some(c) => c,
        None => return rsx! {},
    };

    let choices = &choices_state.choices;

    rsx! {
        div { class: "vn-choices",
            div { class: "vn-choices__panel",
                for (i, choice) in choices.iter().enumerate() {
                    {
                        let text = choice.text.clone();
                        let app = app_state.clone();
                        rsx! {
                            button {
                                key: "{i}",
                                class: "vn-choices__btn",
                                onclick: move |evt: Event<MouseData>| {
                                    evt.stop_propagation();
                                    if let Ok(mut inner) = app.inner.lock() {
                                        inner.process_choose(i);
                                    }
                                },
                                "{text}"
                            }
                        }
                    }
                }
            }
        }
    }
}
