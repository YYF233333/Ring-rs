use dioxus::prelude::*;

use crate::render_state::{PlaybackMode, RenderState};

/// Skip/Auto 模式浮动指示器
///
/// Skip: 绿色背景 + "正在快进 ›" 动画箭头
/// Auto: 蓝色背景 + "自动播放"
#[component]
pub fn SkipIndicator(render_state: Signal<RenderState>) -> Element {
    let rs = render_state.read();

    match rs.playback_mode {
        PlaybackMode::Normal => rsx! {},
        PlaybackMode::Skip => rsx! {
            div { class: "skip-indicator skip-indicator--skip",
                span { "正在快进 " }
                span { class: "skip-indicator__arrows", "›››" }
            }
        },
        PlaybackMode::Auto => rsx! {
            div { class: "skip-indicator skip-indicator--auto", "自动播放" }
        },
    }
}
