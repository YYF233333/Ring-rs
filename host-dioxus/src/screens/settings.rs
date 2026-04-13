use dioxus::prelude::*;

use crate::render_state::{HostScreen, RenderState};
use crate::state::AppState;

/// 设置 screen：音量、文字速度、Auto 延迟
#[component]
pub fn SettingsScreen(render_state: Signal<RenderState>) -> Element {
    let app_state = use_context::<AppState>();

    // 读取当前设置
    let settings = {
        let inner = app_state.inner.lock().unwrap();
        inner.user_settings.clone()
    };

    let mut bgm_vol = use_signal(|| settings.bgm_volume);
    let mut sfx_vol = use_signal(|| settings.sfx_volume);
    let mut text_speed = use_signal(|| settings.text_speed);
    let mut auto_delay = use_signal(|| settings.auto_delay);

    let app_back = app_state.clone();
    let app_bgm = app_state.clone();
    let app_sfx = app_state.clone();
    let app_ts = app_state.clone();
    let app_ad = app_state.clone();

    rsx! {
        div { class: "screen-settings",
            div { class: "screen-settings__header",
                h2 { "Settings" }
                button {
                    class: "screen-settings__back-btn",
                    onclick: move |_| {
                        if let Ok(mut inner) = app_back.inner.lock() {
                            inner.set_host_screen(HostScreen::InGame);
                        }
                    },
                    "Back"
                }
            }

            div { class: "screen-settings__body",
                // BGM Volume
                div { class: "screen-settings__row",
                    label { "BGM Volume" }
                    input {
                        r#type: "range",
                        min: "0",
                        max: "100",
                        step: "1",
                        value: "{bgm_vol}",
                        oninput: move |evt: Event<FormData>| {
                            if let Ok(v) = evt.value().parse::<f32>() {
                                bgm_vol.set(v);
                                if let Ok(mut inner) = app_bgm.inner.lock() {
                                    inner.user_settings.bgm_volume = v;
                                }
                            }
                        },
                    }
                    span { "{bgm_vol:.0}%" }
                }

                // SFX Volume
                div { class: "screen-settings__row",
                    label { "SFX Volume" }
                    input {
                        r#type: "range",
                        min: "0",
                        max: "100",
                        step: "1",
                        value: "{sfx_vol}",
                        oninput: move |evt: Event<FormData>| {
                            if let Ok(v) = evt.value().parse::<f32>() {
                                sfx_vol.set(v);
                                if let Ok(mut inner) = app_sfx.inner.lock() {
                                    inner.user_settings.sfx_volume = v;
                                }
                            }
                        },
                    }
                    span { "{sfx_vol:.0}%" }
                }

                // Text Speed
                div { class: "screen-settings__row",
                    label { "Text Speed (CPS)" }
                    input {
                        r#type: "range",
                        min: "10",
                        max: "100",
                        step: "5",
                        value: "{text_speed}",
                        oninput: move |evt: Event<FormData>| {
                            if let Ok(v) = evt.value().parse::<f32>() {
                                text_speed.set(v);
                                if let Ok(mut inner) = app_ts.inner.lock() {
                                    inner.user_settings.text_speed = v;
                                    inner.text_speed = v;
                                }
                            }
                        },
                    }
                    span { "{text_speed:.0}" }
                }

                // Auto Delay
                div { class: "screen-settings__row",
                    label { "Auto Delay (sec)" }
                    input {
                        r#type: "range",
                        min: "0.5",
                        max: "5.0",
                        step: "0.5",
                        value: "{auto_delay}",
                        oninput: move |evt: Event<FormData>| {
                            if let Ok(v) = evt.value().parse::<f32>() {
                                auto_delay.set(v);
                                if let Ok(mut inner) = app_ad.inner.lock() {
                                    inner.user_settings.auto_delay = v;
                                }
                            }
                        },
                    }
                    span { "{auto_delay:.1}s" }
                }
            }
        }
    }
}
