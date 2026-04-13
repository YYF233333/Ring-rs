use dioxus::prelude::*;

use crate::components::GameMenuFrame;
use crate::render_state::{HostScreen, RenderState};
use crate::state::AppState;

/// 设置 screen（嵌入 GameMenuFrame）
///
/// 滑块参数对齐 egui host：文字速度 5-100 cps，自动延迟 0.5-5.0s，
/// BGM/SFX 0-100%，静音复选框，"应用"按钮。
#[component]
pub fn SettingsScreen(render_state: Signal<RenderState>) -> Element {
    let app_state = use_context::<AppState>();

    // 读取当前设置作为 draft 初始值
    let settings = {
        let Ok(inner) = app_state.inner.lock() else {
            return rsx! {};
        };
        inner.user_settings.clone()
    };

    let mut bgm_vol = use_signal(|| settings.bgm_volume);
    let mut sfx_vol = use_signal(|| settings.sfx_volume);
    let mut text_speed = use_signal(|| settings.text_speed);
    let mut auto_delay = use_signal(|| settings.auto_delay);
    let mut muted = use_signal(|| settings.muted);

    let app_apply = app_state.clone();

    rsx! {
        GameMenuFrame { title: "设置".to_string(), active_screen: HostScreen::Settings,
            div { class: "settings__body",
                // 文字速度
                div { class: "settings__row",
                    label { class: "settings__label", "文字速度" }
                    input {
                        class: "settings__slider",
                        r#type: "range",
                        min: "5",
                        max: "100",
                        step: "1",
                        value: "{text_speed}",
                        oninput: move |evt: Event<FormData>| {
                            if let Ok(v) = evt.value().parse::<f32>() {
                                text_speed.set(v);
                            }
                        },
                    }
                    span { class: "settings__value", "{text_speed:.0} cps" }
                }

                // 自动延迟
                div { class: "settings__row",
                    label { class: "settings__label", "自动延迟" }
                    input {
                        class: "settings__slider",
                        r#type: "range",
                        min: "0.5",
                        max: "5.0",
                        step: "0.1",
                        value: "{auto_delay}",
                        oninput: move |evt: Event<FormData>| {
                            if let Ok(v) = evt.value().parse::<f32>() {
                                auto_delay.set(v);
                            }
                        },
                    }
                    span { class: "settings__value", "{auto_delay:.1} s" }
                }

                // BGM 音量
                div { class: "settings__row",
                    label { class: "settings__label", "BGM 音量" }
                    input {
                        class: "settings__slider",
                        r#type: "range",
                        min: "0",
                        max: "100",
                        step: "1",
                        value: "{bgm_vol}",
                        oninput: move |evt: Event<FormData>| {
                            if let Ok(v) = evt.value().parse::<f32>() {
                                bgm_vol.set(v);
                            }
                        },
                    }
                    span { class: "settings__value", "{bgm_vol:.0}%" }
                }

                // SFX 音量
                div { class: "settings__row",
                    label { class: "settings__label", "SFX 音量" }
                    input {
                        class: "settings__slider",
                        r#type: "range",
                        min: "0",
                        max: "100",
                        step: "1",
                        value: "{sfx_vol}",
                        oninput: move |evt: Event<FormData>| {
                            if let Ok(v) = evt.value().parse::<f32>() {
                                sfx_vol.set(v);
                            }
                        },
                    }
                    span { class: "settings__value", "{sfx_vol:.0}%" }
                }

                // 静音
                div { class: "settings__row",
                    label { class: "settings__label", " " }
                    label { class: "settings__checkbox-label",
                        input {
                            r#type: "checkbox",
                            checked: "{muted}",
                            oninput: move |evt: Event<FormData>| {
                                muted.set(evt.value() == "true");
                            },
                        }
                        " 静音"
                    }
                }

                // 应用按钮
                div { class: "settings__apply-row",
                    button {
                        class: "settings__apply-btn",
                        onclick: move |_| {
                            if let Ok(mut inner) = app_apply.inner.lock() {
                                inner.user_settings.bgm_volume = bgm_vol();
                                inner.user_settings.sfx_volume = sfx_vol();
                                inner.user_settings.text_speed = text_speed();
                                inner.text_speed = text_speed();
                                inner.user_settings.auto_delay = auto_delay();
                                inner.user_settings.muted = muted();
                            }
                        },
                        "应用"
                    }
                }
            }
        }
    }
}
