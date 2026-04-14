use dioxus::prelude::*;

use crate::render_state::RenderState;
use crate::state::AppState;

/// 资源 URL 构建
fn asset_url(path: &str) -> String {
    format!("http://ring-asset.localhost/{path}")
}

/// 视频覆盖层：HTML5 `<video>` 播放 cutscene。
///
/// 点击或视频结束时调用 `finish_cutscene()`。
#[component]
pub fn VideoOverlay(render_state: Signal<RenderState>) -> Element {
    let app_state = use_context::<AppState>();
    let cutscene = use_memo(move || render_state.read().cutscene.clone());

    let cutscene_ref = cutscene.read();
    let cutscene_state = match cutscene_ref.as_ref() {
        Some(c) if c.is_playing => c,
        _ => return rsx! {},
    };

    let video_url = asset_url(&cutscene_state.video_path);
    let app_click = app_state.clone();
    let app_ended = app_state.clone();

    rsx! {
        div {
            class: "vn-video-overlay",
            onclick: move |evt: Event<MouseData>| {
                evt.stop_propagation();
                if let Ok(mut inner) = app_click.inner.lock() {
                    inner.finish_cutscene();
                }
            },

            video {
                src: "{video_url}",
                autoplay: true,
                class: "vn-video-overlay__video",
                onended: move |_| {
                    if let Ok(mut inner) = app_ended.inner.lock() {
                        inner.finish_cutscene();
                    }
                },
            }
        }
    }
}
