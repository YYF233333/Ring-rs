use dioxus::prelude::*;

use crate::render_state::RenderState;

/// 资源 URL 构建（Windows wry 格���）
fn asset_url(path: &str) -> String {
    format!("http://ring-asset.localhost/{path}")
}

/// 立绘层：遍历 `visible_characters`，为每个角色渲染 `<img>`。
///
/// 位置、缩放、透明度、过渡时长全部由后端 `CharacterSprite` 提供，
/// 前端通过 CSS `transition` 实现平滑动���。
#[component]
pub fn CharacterLayer(render_state: Signal<RenderState>) -> Element {
    let rs = render_state.read();
    let characters = &rs.visible_characters;

    if characters.is_empty() {
        return rsx! {};
    }

    // 按 z-order 排序
    let mut sorted: Vec<_> = characters.iter().collect();
    sorted.sort_by_key(|(_, sprite)| sprite.z_order);

    rsx! {
        div { class: "vn-characters",
            for (alias, sprite) in sorted {
                {
                    let url = asset_url(&sprite.texture_path);
                    let z = sprite.z_order;
                    let opacity = sprite.target_alpha;

                    // 位置和缩放
                    let left_pct = sprite.pos_x * 100.0;
                    let top_pct = sprite.pos_y * 100.0;
                    let ax = sprite.anchor_x * 100.0;
                    let ay = sprite.anchor_y * 100.0;
                    let ox = sprite.offset_x;
                    let oy = sprite.offset_y;
                    let sx = sprite.scale_x;
                    let sy = sprite.scale_y;
                    let rs = sprite.render_scale;

                    // CSS transition 时长（秒）
                    let td = sprite.transition_duration.unwrap_or(0.0);
                    let transition = if td > 0.0 {
                        format!("transition: left {td}s ease, top {td}s ease, opacity {td}s ease, transform {td}s ease;")
                    } else {
                        String::new()
                    };

                    let style = format!(
                        "left: {left_pct}%; top: {top_pct}%; \
                         opacity: {opacity}; \
                         z-index: {z}; \
                         transform-origin: {ax}% {ay}%; \
                         transform: translate(-{ax}%, -{ay}%) translate({ox}px, {oy}px) scale({sx}, {sy}) scale({rs}); \
                         {transition}"
                    );

                    rsx! {
                        img {
                            key: "{alias}",
                            class: "vn-characters__sprite",
                            src: "{url}",
                            style: "{style}",
                        }
                    }
                }
            }
        }
    }
}
