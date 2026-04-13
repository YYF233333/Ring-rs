use dioxus::prelude::*;

use crate::render_state::RenderState;

/// 全屏字卡组件：黑底白字，淡入淡出。
///
/// alpha 从 elapsed/duration 计算，20% 淡入 + 20% 淡出 envelope。
#[component]
pub fn TitleCard(render_state: Signal<RenderState>) -> Element {
    let rs = render_state.read();

    let card = match &rs.title_card {
        Some(c) => c,
        None => return rsx! {},
    };

    let progress = if card.duration > 0.0 {
        (card.elapsed / card.duration).clamp(0.0, 1.0)
    } else {
        1.0
    };

    // 20% fade-in, 60% hold, 20% fade-out
    let alpha = if progress < 0.2 {
        progress / 0.2
    } else if progress > 0.8 {
        (1.0 - progress) / 0.2
    } else {
        1.0
    };

    let text = &card.text;

    rsx! {
        div {
            class: "vn-title-card",
            style: "opacity: {alpha};",
            div { class: "vn-title-card__text", "{text}" }
        }
    }
}
