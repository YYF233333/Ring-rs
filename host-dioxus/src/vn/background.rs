use dioxus::prelude::*;

use crate::render_state::RenderState;

/// 资源 URL 构建（Windows wry 格式）
fn asset_url(path: &str) -> String {
    format!("http://ring-asset.localhost/{path}")
}

/// 背景渲染层：双 `<img>` 实现 dissolve 交叉淡化。
///
/// - `current_background`：当前背景，始终以 opacity 1 显示
/// - `background_transition`：过渡中时，旧背景通过 `@keyframes` 从 opacity 1 淡化到 0
///
/// 使用 CSS animation 而非 transition：因为旧背景 `<img>` 是新创建的元素，
/// CSS transition 没有先前状态可过渡，而 animation 自带起始值。
#[component]
pub fn BackgroundLayer(render_state: Signal<RenderState>) -> Element {
    let rs = render_state.read();
    let current_bg = rs.current_background.clone();
    let transition = rs.background_transition.clone();

    rsx! {
        div { class: "vn-background",
            // 旧背景层（过渡中显示，淡出后消失）
            if let Some(ref tr) = transition {
                if let Some(ref old_bg) = tr.old_background {
                    {
                        let old_url = asset_url(old_bg);
                        let duration = tr.duration;
                        rsx! {
                            img {
                                class: "vn-background__img vn-background__img--old",
                                src: "{old_url}",
                                style: "animation: vn-dissolve-out {duration}s ease forwards;",
                            }
                        }
                    }
                }
            }

            // 当前背景层
            if let Some(ref bg) = current_bg {
                {
                    let url = asset_url(bg);
                    rsx! {
                        img {
                            class: "vn-background__img vn-background__img--current",
                            src: "{url}",
                        }
                    }
                }
            }
        }
    }
}
