use dioxus::prelude::*;

use crate::components::PendingConfirm;
use crate::render_state::RenderState;
use crate::screen_defs::ActionDef;
use crate::state::AppState;

/// 游内暂停菜单（数据驱动，从 screens.json ingame_menu.buttons 渲染）
///
/// 半透明遮罩 + 居中按钮列表。点击遮罩关闭。
#[component]
pub fn InGameMenu(render_state: Signal<RenderState>) -> Element {
    let app_state = use_context::<AppState>();
    let mut pending_confirm = use_context::<Signal<Option<PendingConfirm>>>();

    // 从 screen_defs 获取按钮列表
    let buttons = {
        let Ok(inner) = app_state.inner.lock() else {
            return rsx! {};
        };
        let Some(svc) = inner.services.as_ref() else {
            return rsx! {};
        };
        svc.screen_defs.ingame_menu.buttons.clone()
    };

    let app_close = app_state.clone();

    rsx! {
        div {
            class: "screen-ingame-menu",
            onclick: {
                let app = app_state.clone();
                move |_| {
                    // 点击遮罩区域关闭菜单
                    if let Ok(mut inner) = app.inner.lock() {
                        inner.execute_action(&ActionDef::GoBack);
                    }
                }
            },

            div {
                class: "screen-ingame-menu__panel",
                onclick: move |evt: Event<MouseData>| {
                    evt.stop_propagation();
                },

                for btn in &buttons {
                    {
                        let label = btn.label.clone();
                        let action = btn.action.clone();
                        let confirm_msg = btn.confirm.clone();
                        let app = app_close.clone();

                        rsx! {
                            button {
                                key: "{label}",
                                class: "screen-ingame-menu__btn",
                                onclick: move |_| {
                                    if let Some(ref msg) = confirm_msg {
                                        pending_confirm.set(Some(PendingConfirm {
                                            message: msg.clone(),
                                            on_confirm: action.clone(),
                                        }));
                                    } else if matches!(action, ActionDef::Exit) {
                                        dioxus::desktop::window().close();
                                    } else if let Ok(mut inner) = app.inner.lock() {
                                        inner.execute_action(&action);
                                    }
                                },
                                "{label}"
                            }
                        }
                    }
                }
            }
        }
    }
}
