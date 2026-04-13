use dioxus::prelude::*;

use crate::components::PendingConfirm;
use crate::layout_config::UiAssetPaths;
use crate::render_state::HostScreen;
use crate::screen_defs::ActionDef;
use crate::state::AppState;

/// 游戏菜单通用框架（数据驱动，左导航 + 右内容）
///
/// 供 SaveLoad、Settings、History 等页面复用。
/// 背景图 + overlay 从 screens.json game_menu 定义加载。
/// 左侧导航按钮从 game_menu.nav_buttons 渲染。
#[component]
pub fn GameMenuFrame(
    /// 右侧内容区标题（如 "保存"、"设置"）
    title: String,
    /// 当前高亮的导航项对应的 HostScreen
    active_screen: HostScreen,
    /// 右侧内容区子元素
    children: Element,
) -> Element {
    let app_state = use_context::<AppState>();
    let mut pending_confirm = use_context::<Signal<Option<PendingConfirm>>>();

    // 从 screen_defs 获取 game_menu 定义
    let (bg_url, overlay_url, nav_buttons, return_button) = {
        let Ok(inner) = app_state.inner.lock() else {
            return rsx! {};
        };
        let Some(svc) = inner.services.as_ref() else {
            return rsx! {};
        };

        let ctx = inner.condition_context();
        let gm = &svc.screen_defs.game_menu;

        let bg_key = crate::screen_defs::ConditionalAsset::resolve(&gm.background, &ctx)
            .unwrap_or("game_menu_bg");
        let bg_path = svc.layout.assets.resolve_key(bg_key);
        let bg_url = UiAssetPaths::asset_url(&bg_path);

        let overlay_url = gm.overlay.as_ref().map(|key| {
            let path = svc.layout.assets.resolve_key(key);
            UiAssetPaths::asset_url(&path)
        });

        let nav_buttons: Vec<_> = gm
            .nav_buttons
            .iter()
            .filter(|btn| btn.visible.as_ref().is_none_or(|cond| cond.evaluate(&ctx)))
            .cloned()
            .collect();

        let return_button = gm.return_button.clone();

        (bg_url, overlay_url, nav_buttons, return_button)
    };

    rsx! {
        div { class: "game-menu",
            // 背景图
            img { class: "game-menu__bg", src: "{bg_url}" }

            // Overlay
            if let Some(ref url) = overlay_url {
                img { class: "game-menu__overlay", src: "{url}" }
            }

            // 左侧导航面板
            div { class: "game-menu__nav",
                div { class: "game-menu__nav-buttons",
                    for btn in &nav_buttons {
                        {
                            let label = btn.label.clone();
                            let action = btn.action.clone();
                            let confirm_msg = btn.confirm.clone();
                            let app = app_state.clone();
                            // 高亮当前页面对应的导航按钮
                            let is_active = matches!(
                                (&action, &active_screen),
                                (ActionDef::OpenSave, HostScreen::Save)
                                | (ActionDef::OpenLoad, HostScreen::Load)
                                | (ActionDef::ReplaceSettings | ActionDef::NavigateSettings, HostScreen::Settings)
                                | (ActionDef::ReplaceHistory | ActionDef::NavigateHistory, HostScreen::History)
                            );
                            let class = if is_active {
                                "game-menu__nav-btn game-menu__nav-btn--active"
                            } else {
                                "game-menu__nav-btn"
                            };

                            rsx! {
                                button {
                                    key: "{label}",
                                    class: "{class}",
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

                // 底部返回按钮
                {
                    let label = return_button.label.clone();
                    let action = return_button.action.clone();
                    let app = app_state.clone();

                    rsx! {
                        button {
                            class: "game-menu__return-btn",
                            onclick: move |_| {
                                if let Ok(mut inner) = app.inner.lock() {
                                    inner.execute_action(&action);
                                }
                            },
                            "{label}"
                        }
                    }
                }
            }

            // 右侧内容区
            div { class: "game-menu__content",
                h2 { class: "game-menu__title", "{title}" }
                div { class: "game-menu__body",
                    {children}
                }
            }
        }
    }
}
