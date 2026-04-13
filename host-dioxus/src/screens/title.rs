use dioxus::prelude::*;

use crate::components::PendingConfirm;
use crate::layout_config::UiAssetPaths;
use crate::render_state::RenderState;
use crate::screen_defs::{ActionDef, ButtonDef, ConditionalAsset};
use crate::state::AppState;

/// 标题画面（数据驱动，从 screens.json title 定义渲染）
///
/// - 背景图：条件切换（summer/winter）
/// - Overlay：叠加于背景上方
/// - 按钮：从 screens.json 加载，支持条件显隐和确认弹窗
#[component]
pub fn TitleScreen(render_state: Signal<RenderState>) -> Element {
    let app_state = use_context::<AppState>();

    // 从 screen_defs 获取标题页定义和条件上下文
    let (bg_url, overlay_url, buttons) = {
        let Ok(inner) = app_state.inner.lock() else {
            return rsx! {};
        };
        let Some(svc) = inner.services.as_ref() else {
            return rsx! {};
        };

        let ctx = inner.condition_context();
        let title_def = &svc.screen_defs.title;

        // 解析条件背景
        let bg_key =
            ConditionalAsset::resolve(&title_def.background, &ctx).unwrap_or("main_summer");
        let bg_path = svc.layout.assets.resolve_key(bg_key);
        let bg_url = UiAssetPaths::asset_url(&bg_path);

        // 解析 overlay
        let overlay_url = title_def.overlay.as_ref().map(|key| {
            let path = svc.layout.assets.resolve_key(key);
            UiAssetPaths::asset_url(&path)
        });

        // 过滤可见按钮
        let buttons: Vec<ButtonDef> = title_def
            .buttons
            .iter()
            .filter(|btn| btn.visible.as_ref().is_none_or(|cond| cond.evaluate(&ctx)))
            .cloned()
            .collect();

        (bg_url, overlay_url, buttons)
    };

    rsx! {
        div { class: "screen-title",
            // 背景图
            img {
                class: "screen-title__bg",
                src: "{bg_url}",
            }

            // Overlay
            if let Some(ref url) = overlay_url {
                img {
                    class: "screen-title__overlay",
                    src: "{url}",
                }
            }

            // 按钮列表
            div { class: "screen-title__nav",
                for btn in &buttons {
                    { render_button(btn, &app_state) }
                }
            }
        }
    }
}

/// 渲染数据驱动按钮（支持 confirm 弹窗和 Exit 特殊处理）
fn render_button(btn: &ButtonDef, app_state: &AppState) -> Element {
    let mut pending_confirm = use_context::<Signal<Option<PendingConfirm>>>();
    let label = btn.label.clone();
    let action = btn.action.clone();
    let confirm_msg = btn.confirm.clone();
    let app = app_state.clone();

    rsx! {
        button {
            key: "{label}",
            class: "screen-title__btn",
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
