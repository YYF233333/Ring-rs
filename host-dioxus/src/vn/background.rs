use dioxus::prelude::*;

use crate::render_state::RenderState;

/// 资源 URL 构建（Windows wry 格式）
fn asset_url(path: &str) -> String {
    format!("http://ring-asset.localhost/{path}")
}

/// 背景渲染层：双 `<img>` 实现 dissolve 交叉淡化。
///
/// - `current_background`：当前背景，始终以 opacity 1 显示
/// - `background_transition`：过渡中时，旧背景从 opacity 1 淡化到 0
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
                                style: "transition: opacity {duration}s ease; opacity: 0;",
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
