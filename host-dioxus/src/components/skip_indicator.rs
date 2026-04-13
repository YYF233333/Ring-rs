use dioxus::prelude::*;

use crate::render_state::{PlaybackMode, RenderState};

/// Skip/Auto 模式浮动指示器
#[component]
pub fn SkipIndicator(render_state: Signal<RenderState>) -> Element {
    let rs = render_state.read();

    let label = match rs.playback_mode {
        PlaybackMode::Skip => "SKIP",
        PlaybackMode::Auto => "AUTO",
        PlaybackMode::Normal => return rsx! {},
    };

    let modifier = match rs.playback_mode {
        PlaybackMode::Skip => "skip-indicator--skip",
        PlaybackMode::Auto => "skip-indicator--auto",
        PlaybackMode::Normal => "",
    };

    rsx! {
        div { class: "skip-indicator {modifier}", "{label}" }
    }
}
