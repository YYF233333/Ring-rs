use dioxus::prelude::*;

use crate::render_state::{HostScreen, PlaybackMode, RenderState};
use crate::state::AppState;

/// 底部快捷菜单：Skip / Auto / Save / Load / History / Settings
///
/// 仅在 InGame 模式且有对话时显示。
#[component]
pub fn QuickMenu(render_state: Signal<RenderState>) -> Element {
    let app_state = use_context::<AppState>();
    let rs = render_state.read();

    // 仅在 InGame + ui_visible 时显示
    if rs.host_screen != HostScreen::InGame || !rs.ui_visible {
        return rsx! {};
    }

    let is_skip = rs.playback_mode == PlaybackMode::Skip;
    let is_auto = rs.playback_mode == PlaybackMode::Auto;

    let app_skip = app_state.clone();
    let app_auto = app_state.clone();
    let app_save = app_state.clone();
    let app_load = app_state.clone();
    let app_history = app_state.clone();
    let app_settings = app_state.clone();

    rsx! {
        div { class: "vn-quick-menu",
            button {
                class: if is_skip { "vn-quick-menu__btn vn-quick-menu__btn--active" } else { "vn-quick-menu__btn" },
                onclick: move |evt: Event<MouseData>| {
                    evt.stop_propagation();
                    if let Ok(mut inner) = app_skip.inner.lock() {
                        let mode = if is_skip { PlaybackMode::Normal } else { PlaybackMode::Skip };
                        inner.set_playback_mode(mode);
                    }
                },
                "Skip"
            }
            button {
                class: if is_auto { "vn-quick-menu__btn vn-quick-menu__btn--active" } else { "vn-quick-menu__btn" },
                onclick: move |evt: Event<MouseData>| {
                    evt.stop_propagation();
                    if let Ok(mut inner) = app_auto.inner.lock() {
                        let mode = if is_auto { PlaybackMode::Normal } else { PlaybackMode::Auto };
                        inner.set_playback_mode(mode);
                    }
                },
                "Auto"
            }
            button {
                class: "vn-quick-menu__btn",
                onclick: move |evt: Event<MouseData>| {
                    evt.stop_propagation();
                    if let Ok(mut inner) = app_save.inner.lock() {
                        inner.set_host_screen(HostScreen::Save);
                    }
                },
                "Save"
            }
            button {
                class: "vn-quick-menu__btn",
                onclick: move |evt: Event<MouseData>| {
                    evt.stop_propagation();
                    if let Ok(mut inner) = app_load.inner.lock() {
                        inner.set_host_screen(HostScreen::Load);
                    }
                },
                "Load"
            }
            button {
                class: "vn-quick-menu__btn",
                onclick: move |evt: Event<MouseData>| {
                    evt.stop_propagation();
                    if let Ok(mut inner) = app_history.inner.lock() {
                        inner.set_host_screen(HostScreen::History);
                    }
                },
                "History"
            }
            button {
                class: "vn-quick-menu__btn",
                onclick: move |evt: Event<MouseData>| {
                    evt.stop_propagation();
                    if let Ok(mut inner) = app_settings.inner.lock() {
                        inner.set_host_screen(HostScreen::Settings);
                    }
                },
                "Settings"
            }
        }
    }
}
