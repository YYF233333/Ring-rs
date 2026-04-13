use dioxus::prelude::*;
use tracing::error;

use crate::render_state::{HostScreen, RenderState};
use crate::state::AppState;

/// 标题画面：Start / Continue / Load / Settings / Exit
#[component]
pub fn TitleScreen(render_state: Signal<RenderState>) -> Element {
    let app_state = use_context::<AppState>();

    // 检查是否有 continue 存档
    let has_continue = {
        let inner = app_state.inner.lock().unwrap();
        inner.services().saves.load_continue().is_ok()
    };

    let app_start = app_state.clone();
    let app_continue = app_state.clone();
    let app_load = app_state.clone();
    let app_settings = app_state.clone();

    rsx! {
        div { class: "screen-title",
            h1 { class: "screen-title__heading", "Ring Engine" }

            button {
                class: "screen-title__btn",
                onclick: move |_| {
                    if let Ok(mut inner) = app_start.inner.lock() {
                        let start_path = inner.services().config.start_script_path.clone();
                        if let Err(e) = inner.init_game_from_resource(&start_path) {
                            error!(error = %e, "Start failed");
                        }
                    }
                },
                "Start"
            }

            if has_continue {
                button {
                    class: "screen-title__btn",
                    onclick: move |_| {
                        if let Ok(mut inner) = app_continue.inner.lock() {
                            match inner.services().saves.load_continue() {
                                Ok(save) => {
                                    if let Err(e) = inner.restore_from_save(save) {
                                        error!(error = %e, "Continue failed");
                                    }
                                }
                                Err(e) => error!(error = %e, "Load continue failed"),
                            }
                        }
                    },
                    "Continue"
                }
            }

            button {
                class: "screen-title__btn",
                onclick: move |_| {
                    if let Ok(mut inner) = app_load.inner.lock() {
                        inner.set_host_screen(HostScreen::Load);
                    }
                },
                "Load"
            }

            button {
                class: "screen-title__btn",
                onclick: move |_| {
                    if let Ok(mut inner) = app_settings.inner.lock() {
                        inner.set_host_screen(HostScreen::Settings);
                    }
                },
                "Settings"
            }

            button {
                class: "screen-title__btn",
                onclick: move |_| {
                    dioxus::desktop::window().close();
                },
                "Exit"
            }
        }
    }
}
