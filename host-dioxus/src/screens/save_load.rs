use dioxus::prelude::*;
use tracing::error;

use crate::render_state::{HostScreen, RenderState};
use crate::state::AppState;

const SLOTS_PER_PAGE: u32 = 6;
const MAX_PAGES: u32 = 9;

/// 存档/读档 screen
///
/// `mode` 由当前 `HostScreen` 决定：Save 或 Load。
#[component]
pub fn SaveLoadScreen(render_state: Signal<RenderState>) -> Element {
    let app_state = use_context::<AppState>();
    let mut current_page = use_signal(|| 0u32);

    let rs = render_state.read();
    let is_save_mode = rs.host_screen == HostScreen::Save;
    let title = if is_save_mode { "Save" } else { "Load" };

    // 当前页的 slot 范围
    let page = current_page();
    let start_slot = page * SLOTS_PER_PAGE + 1;

    // 获取存档列表
    let saves_info: Vec<(u32, bool, Option<String>)> = {
        let inner = app_state.inner.lock().unwrap();
        let sm = &inner.services().saves;
        let existing: std::collections::HashSet<u32> =
            sm.list_saves().into_iter().map(|(slot, _)| slot).collect();
        (start_slot..start_slot + SLOTS_PER_PAGE)
            .map(|slot| {
                let exists = existing.contains(&slot);
                let thumb = if exists {
                    sm.load_thumbnail_base64(slot)
                } else {
                    None
                };
                (slot, exists, thumb)
            })
            .collect()
    };

    let app_back = app_state.clone();

    rsx! {
        div { class: "screen-save-load",
            // 顶部：标题 + 返回按钮
            div { class: "screen-save-load__header",
                h2 { "{title}" }
                button {
                    class: "screen-save-load__back-btn",
                    onclick: move |_| {
                        if let Ok(mut inner) = app_back.inner.lock() {
                            inner.set_host_screen(HostScreen::InGame);
                        }
                    },
                    "Back"
                }
            }

            // Slot 网格
            div { class: "screen-save-load__grid",
                for (slot, exists, thumb) in saves_info {
                    {
                        let app = app_state.clone();
                        let slot_label = format!("Slot {slot}");
                        rsx! {
                            div {
                                key: "{slot}",
                                class: if exists { "screen-save-load__slot screen-save-load__slot--filled" } else { "screen-save-load__slot" },
                                onclick: move |_| {
                                    if let Ok(mut inner) = app.inner.lock() {
                                        if is_save_mode {
                                            if let Err(e) = inner.save_to_slot(slot) {
                                                error!(error = %e, slot = slot, "Save failed");
                                            }
                                        } else if exists {
                                            match inner.services().saves.load(slot) {
                                                Ok(save) => {
                                                    if let Err(e) = inner.restore_from_save(save) {
                                                        error!(error = %e, "Restore failed");
                                                    }
                                                }
                                                Err(e) => error!(error = %e, "Load failed"),
                                            }
                                        }
                                    }
                                },

                                // 缩略图
                                if let Some(ref b64) = thumb {
                                    img {
                                        class: "screen-save-load__thumb",
                                        src: "data:image/png;base64,{b64}",
                                    }
                                }

                                div { class: "screen-save-load__slot-label",
                                    "{slot_label}"
                                }
                            }
                        }
                    }
                }
            }

            // 分页
            div { class: "screen-save-load__pagination",
                for p in 0..MAX_PAGES {
                    button {
                        class: if p == page { "screen-save-load__page-btn screen-save-load__page-btn--active" } else { "screen-save-load__page-btn" },
                        onclick: move |_| { current_page.set(p); },
                        "{p + 1}"
                    }
                }
            }
        }
    }
}
