//! showMap UI 模式：全屏地图覆盖层。
//!
//! 从 `maps/{map_id}.json` 加载地图定义，渲染背景 + 位置按钮。
//! 玩家点击可用位置后，通过 `handle_ui_result` 回传选择结果。

use dioxus::prelude::*;

use crate::layout_config::UiAssetPaths;
use crate::map_data::MapDefinition;
use crate::render_state::RenderState;
use crate::state::AppState;

/// showMap 全屏地图覆盖层
///
/// 当 `active_ui_mode.mode == "show_map"` 时渲染。
/// 从资源系统加载 `maps/{map_id}.json`，渲染背景图和位置按钮。
#[component]
pub fn MapOverlay(render_state: Signal<RenderState>) -> Element {
    let app_state = use_context::<AppState>();
    let rs = render_state.read();

    let ui_mode = match &rs.active_ui_mode {
        Some(m) if m.mode == "show_map" => m,
        _ => return rsx! {},
    };

    let map_id = ui_mode
        .params
        .get("map_id")
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .to_string();
    let ui_key = ui_mode.key.clone();

    // 加载地图定义（每次 map_id 变化时重新加载）
    let map_def = use_memo({
        let app_state = app_state.clone();
        let map_id = map_id.clone();
        move || {
            let Ok(inner) = app_state.inner.lock() else {
                return None;
            };
            let svc = inner.services.as_ref()?;
            let map_path = format!("maps/{}.json", map_id);
            let logical = crate::resources::LogicalPath::new(&map_path);
            let json_text = svc.resources.read_text(&logical).ok()?;
            serde_json::from_str::<MapDefinition>(&json_text).ok()
        }
    });

    let Some(def) = map_def.read().clone() else {
        // 地图加载失败，返回空结果
        let app = app_state.clone();
        let key = ui_key.clone();
        return rsx! {
            div {
                class: "vn-map-overlay",
                onclick: move |evt: Event<MouseData>| {
                    evt.stop_propagation();
                    if let Ok(mut inner) = app.inner.lock()
                        && let Err(e) =
                            inner.handle_ui_result(key.clone(), serde_json::Value::String(String::new()))
                        {
                            tracing::warn!(error = %e, "handle_ui_result 失败（地图加载失败路径）");
                        }
                },
                div { class: "vn-map-overlay__title", "地图加载失败" }
            }
        };
    };

    let bg_url = def.background.as_ref().map(|p| UiAssetPaths::asset_url(p));
    let title = def.title.clone();
    let locations = def.locations.clone();

    rsx! {
        div {
            class: "vn-map-overlay",
            onclick: move |evt: Event<MouseData>| {
                evt.stop_propagation();
            },

            // 背景图
            if let Some(ref url) = bg_url {
                img {
                    class: "vn-map-overlay__bg",
                    src: "{url}",
                }
            }

            // 标题
            div { class: "vn-map-overlay__title", "{title}" }

            // 位置按钮
            for loc in &locations {
                {
                    let loc_id = loc.id.clone();
                    let loc_label = loc.label.clone();
                    let enabled = loc.enabled;
                    let x = loc.x;
                    let y = loc.y;
                    let app = app_state.clone();
                    let key = ui_key.clone();

                    let btn_class = if enabled {
                        "vn-map-overlay__btn"
                    } else {
                        "vn-map-overlay__btn vn-map-overlay__btn--disabled"
                    };

                    let style = format!("left: {x}px; top: {y}px;");

                    rsx! {
                        button {
                            key: "{loc_id}",
                            class: "{btn_class}",
                            style: "{style}",
                            disabled: !enabled,
                            onclick: move |evt: Event<MouseData>| {
                                evt.stop_propagation();
                                if enabled
                                    && let Ok(mut inner) = app.inner.lock()
                                        && let Err(e) = inner.handle_ui_result(
                                            key.clone(),
                                            serde_json::Value::String(loc_id.clone()),
                                        ) {
                                            tracing::warn!(error = %e, loc_id = %loc_id, "handle_ui_result 失败（地图选择）");
                                        }
                            },
                            "{loc_label}"
                        }
                    }
                }
            }
        }
    }
}
