use dioxus::prelude::*;

use vn_runtime::command::TextMode;

use crate::render_state::{HostScreen, PlaybackMode, RenderState};
use crate::screen_defs::ActionDef;
use crate::state::AppState;

/// 底部快捷菜单（数据驱动，从 screens.json quick_menu.buttons 渲染）
///
/// 位于对话框内部底边居中。仅在 InGame + ADV 模式且 UI 可见时显示。
#[component]
pub fn QuickMenu(render_state: Signal<RenderState>) -> Element {
    let app_state = use_context::<AppState>();
    let rs = render_state.read();

    // 仅在 InGame + ADV 模式 + ui_visible 时显示（NVL 模式无 quick menu）
    if rs.host_screen != HostScreen::InGame || rs.text_mode == TextMode::NVL || !rs.ui_visible {
        return rsx! {};
    }

    let is_skip = rs.playback_mode == PlaybackMode::Skip;
    let is_auto = rs.playback_mode == PlaybackMode::Auto;

    // 从 screen_defs 获取按钮列表
    let buttons = {
        let Ok(inner) = app_state.inner.lock() else {
            return rsx! {};
        };
        let Some(svc) = inner.services.as_ref() else {
            return rsx! {};
        };
        svc.screen_defs.quick_menu.buttons.clone()
    };

    rsx! {
        div { class: "vn-quick-menu",
            for btn in &buttons {
                {
                    let label = btn.label.clone();
                    let action = btn.action.clone();
                    let is_active = matches!(
                        (&action, is_skip, is_auto),
                        (ActionDef::ToggleSkip, true, _) | (ActionDef::ToggleAuto, _, true)
                    );
                    let class_name = if is_active {
                        "vn-quick-menu__btn vn-quick-menu__btn--active"
                    } else {
                        "vn-quick-menu__btn"
                    };
                    let app = app_state.clone();

                    rsx! {
                        button {
                            key: "{label}",
                            class: "{class_name}",
                            onclick: move |evt: Event<MouseData>| {
                                evt.stop_propagation();
                                if let Ok(mut inner) = app.inner.lock() {
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
