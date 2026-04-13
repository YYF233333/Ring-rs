use dioxus::prelude::*;

use crate::render_state::{HostScreen, RenderState};
use crate::state::AppState;

/// 游内暂停菜单：半透明遮罩 + 菜单按钮
#[component]
pub fn InGameMenu(render_state: Signal<RenderState>) -> Element {
    let app_state = use_context::<AppState>();

    let app_resume = app_state.clone();
    let app_save = app_state.clone();
    let app_load = app_state.clone();
    let app_settings = app_state.clone();
    let app_history = app_state.clone();
    let app_title = app_state.clone();

    rsx! {
        div {
            class: "screen-ingame-menu",
            onclick: move |evt: Event<MouseData>| {
                // 点击遮罩区域关闭菜单
                evt.stop_propagation();
            },

            div { class: "screen-ingame-menu__panel",
                button {
                    class: "screen-ingame-menu__btn",
                    onclick: move |_| {
                        if let Ok(mut inner) = app_resume.inner.lock() {
                            inner.set_host_screen(HostScreen::InGame);
                        }
                    },
                    "Continue"
                }

                button {
                    class: "screen-ingame-menu__btn",
                    onclick: move |_| {
                        if let Ok(mut inner) = app_save.inner.lock() {
                            inner.set_host_screen(HostScreen::Save);
                        }
                    },
                    "Save"
                }

                button {
                    class: "screen-ingame-menu__btn",
                    onclick: move |_| {
                        if let Ok(mut inner) = app_load.inner.lock() {
                            inner.set_host_screen(HostScreen::Load);
                        }
                    },
                    "Load"
                }

                button {
                    class: "screen-ingame-menu__btn",
                    onclick: move |_| {
                        if let Ok(mut inner) = app_history.inner.lock() {
                            inner.set_host_screen(HostScreen::History);
                        }
                    },
                    "History"
                }

                button {
                    class: "screen-ingame-menu__btn",
                    onclick: move |_| {
                        if let Ok(mut inner) = app_settings.inner.lock() {
                            inner.set_host_screen(HostScreen::Settings);
                        }
                    },
                    "Settings"
                }

                button {
                    class: "screen-ingame-menu__btn",
                    onclick: move |_| {
                        if let Ok(mut inner) = app_title.inner.lock() {
                            inner.set_host_screen(HostScreen::Title);
                        }
                    },
                    "Return to Title"
                }

                button {
                    class: "screen-ingame-menu__btn",
                    onclick: move |_| {
                        dioxus::desktop::window().close();
                    },
                    "Exit"
                }
            }
        }
    }
}
