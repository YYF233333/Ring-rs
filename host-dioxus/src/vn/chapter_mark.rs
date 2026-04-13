use dioxus::prelude::*;

use crate::render_state::RenderState;

/// 章节标记组件：显示章节标题，alpha 由后端驱动。
#[component]
pub fn ChapterMark(render_state: Signal<RenderState>) -> Element {
    let rs = render_state.read();

    let mark = match &rs.chapter_mark {
        Some(m) => m,
        None => return rsx! {},
    };

    let alpha = mark.alpha;
    let title = &mark.title;
    let level = mark.level;

    let font_size = match level {
        1 => "2.2em",
        2 => "1.6em",
        _ => "1.3em",
    };

    rsx! {
        div {
            class: "vn-chapter-mark",
            style: "opacity: {alpha}; font-size: {font_size};",
            "{title}"
        }
    }
}
