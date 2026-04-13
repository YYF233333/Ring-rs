//! callGame UI 模式：iframe 嵌入 HTML5 小游戏。
//!
//! 游戏文件从 `ring-asset://games/{game_id}/index.html` 加载，
//! JS SDK 由 ring-asset handler 自动注入（`window.engine.*`）。
//! 游戏通过 `window.engine.complete(result)` 回传结果，
//! 经同源 fetch → ring-asset `__game_complete` 端点 → static 全局 → Rust 轮询闭环。

use std::time::Duration;

use dioxus::prelude::*;

use crate::render_state::RenderState;
use crate::state::AppState;

/// callGame 全屏小游戏覆盖层
///
/// 当 `active_ui_mode.mode == "call_game"` 时渲染。
/// 使用 iframe 加载游戏页面，通过同源 fetch 桥接完成信号。
#[component]
pub fn MinigameOverlay(render_state: Signal<RenderState>) -> Element {
    let app_state = use_context::<AppState>();
    let rs = render_state.read();

    let ui_mode = match &rs.active_ui_mode {
        Some(m) if m.mode == "call_game" => m,
        _ => return rsx! {},
    };

    let game_id = ui_mode
        .params
        .get("game_id")
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .to_string();
    let ui_key = ui_mode.key.clone();

    let game_url = format!("http://ring-asset.localhost/games/{game_id}/index.html");

    // 轮询 static 全局，检测游戏完成
    {
        let app = app_state.clone();
        let key = ui_key.clone();
        use_future(move || {
            let app = app.clone();
            let key = key.clone();
            async move {
                // 清空可能残留的旧结果
                if let Ok(mut slot) = crate::GAME_COMPLETE_RESULT.lock() {
                    *slot = None;
                }
                loop {
                    tokio::time::sleep(Duration::from_millis(100)).await;
                    let result = crate::GAME_COMPLETE_RESULT
                        .lock()
                        .ok()
                        .and_then(|mut slot| slot.take());
                    if let Some(r) = result {
                        if let Ok(mut inner) = app.inner.lock() {
                            let _ =
                                inner.handle_ui_result(key.clone(), serde_json::Value::String(r));
                        }
                        break;
                    }
                }
            }
        });
    }

    rsx! {
        div {
            class: "vn-minigame-overlay",
            onclick: move |evt: Event<MouseData>| {
                evt.stop_propagation();
            },

            iframe {
                class: "vn-minigame-overlay__frame",
                src: "{game_url}",
            }
        }
    }
}
