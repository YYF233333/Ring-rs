use dioxus::prelude::*;

use crate::screen_defs::ActionDef;
use crate::state::AppState;

/// 待确认操作
#[derive(Debug, Clone)]
pub struct PendingConfirm {
    /// 弹窗消息
    pub message: String,
    /// 用户确认后执行的动作
    pub on_confirm: ActionDef,
}

/// 模态确认弹窗组件
///
/// 全屏遮罩 + 居中面板 + 消息 + 确定/取消按钮。
/// 通过 `use_context::<Signal<Option<PendingConfirm>>>()` 控制显隐。
#[component]
pub fn ConfirmDialog() -> Element {
    let mut pending = use_context::<Signal<Option<PendingConfirm>>>();
    let app_state = use_context::<AppState>();

    let confirm = pending.read();
    let Some(ref data) = *confirm else {
        return rsx! {};
    };

    let message = data.message.clone();
    let action = data.on_confirm.clone();

    rsx! {
        div {
            class: "confirm-overlay",
            // 点击遮罩 = 取消
            onclick: move |_| {
                pending.set(None);
            },

            div {
                class: "confirm-panel",
                // 阻止面板内点击冒泡到遮罩
                onclick: move |evt: Event<MouseData>| {
                    evt.stop_propagation();
                },

                // 消息文字
                div { class: "confirm-panel__message", "{message}" }

                // 按钮区
                div { class: "confirm-panel__buttons",
                    button {
                        class: "confirm-panel__btn",
                        onclick: {
                            let app = app_state.clone();
                            move |_| {
                                pending.set(None);
                                if matches!(action, ActionDef::Exit) {
                                    dioxus::desktop::window().close();
                                } else if let Ok(mut inner) = app.inner.lock() {
                                    inner.execute_action(&action);
                                }
                            }
                        },
                        "确定"
                    }
                    button {
                        class: "confirm-panel__btn",
                        onclick: move |_| {
                            pending.set(None);
                        },
                        "取消"
                    }
                }
            }
        }
    }
}
