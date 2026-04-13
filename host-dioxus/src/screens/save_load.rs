use dioxus::prelude::*;
use tracing::error;

use crate::components::{GameMenuFrame, PendingConfirm};
use crate::render_state::{HostScreen, RenderState};
use crate::screen_defs::ActionDef;
use crate::state::AppState;

const SLOTS_PER_PAGE: u32 = 6;

/// 分页类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PageKind {
    Auto,
    Quick,
    Manual(u32), // 1-9
}

impl PageKind {
    /// 该分页的起始 slot（含）
    fn start_slot(self) -> u32 {
        match self {
            PageKind::Auto => 61,
            PageKind::Quick => 55,
            PageKind::Manual(n) => (n - 1) * SLOTS_PER_PAGE + 1,
        }
    }

    fn label(self) -> &'static str {
        match self {
            PageKind::Auto => "A",
            PageKind::Quick => "Q",
            PageKind::Manual(n) => match n {
                1 => "1",
                2 => "2",
                3 => "3",
                4 => "4",
                5 => "5",
                6 => "6",
                7 => "7",
                8 => "8",
                9 => "9",
                _ => "?",
            },
        }
    }

    fn all() -> Vec<PageKind> {
        let mut pages = vec![PageKind::Auto, PageKind::Quick];
        for n in 1..=9 {
            pages.push(PageKind::Manual(n));
        }
        pages
    }
}

/// 存档/读档 screen（嵌入 GameMenuFrame）
#[component]
pub fn SaveLoadScreen(render_state: Signal<RenderState>) -> Element {
    let app_state = use_context::<AppState>();
    let mut pending_confirm = use_context::<Signal<Option<PendingConfirm>>>();
    let mut current_page = use_signal(|| PageKind::Manual(1));

    let rs = render_state.read();
    let is_save_mode = rs.host_screen == HostScreen::Save;
    let title = if is_save_mode { "保存" } else { "读取" };
    let active_screen = rs.host_screen.clone();

    // 当前页的 slot 范围
    let page = current_page();
    let start_slot = page.start_slot();

    // 获取存档列表
    let saves_info: Vec<SlotInfo> = {
        let Ok(inner) = app_state.inner.lock() else {
            return rsx! {};
        };
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
                let info = if exists { sm.get_save_info(slot) } else { None };
                SlotInfo {
                    slot,
                    exists,
                    thumb,
                    chapter: info.as_ref().and_then(|i| i.chapter_title.clone()),
                    timestamp: info.as_ref().map(|i| i.timestamp.clone()),
                }
            })
            .collect()
    };

    let all_pages = PageKind::all();

    rsx! {
        GameMenuFrame { title: title.to_string(), active_screen,
            // Tab 切换
            div { class: "save-load__tabs",
                button {
                    class: if is_save_mode { "save-load__tab save-load__tab--active" } else { "save-load__tab" },
                    onclick: {
                        let app = app_state.clone();
                        move |_| {
                            if let Ok(mut inner) = app.inner.lock() {
                                inner.set_host_screen(HostScreen::Save);
                            }
                        }
                    },
                    if is_save_mode { "[ 保存 ]" } else { "保存" }
                }
                button {
                    class: if !is_save_mode { "save-load__tab save-load__tab--active" } else { "save-load__tab" },
                    onclick: {
                        let app = app_state.clone();
                        move |_| {
                            if let Ok(mut inner) = app.inner.lock() {
                                inner.set_host_screen(HostScreen::Load);
                            }
                        }
                    },
                    if !is_save_mode { "[ 读取 ]" } else { "读取" }
                }
            }

            // Slot 网格 3×2
            div { class: "save-load__grid",
                for si in &saves_info {
                    {
                        let slot = si.slot;
                        let exists = si.exists;
                        let thumb = si.thumb.clone();
                        let chapter = si.chapter.clone();
                        let timestamp = si.timestamp.clone();
                        let app = app_state.clone();
                        let app_del = app_state.clone();

                        rsx! {
                            div {
                                key: "{slot}",
                                class: if exists { "save-load__slot save-load__slot--filled" } else { "save-load__slot" },
                                onclick: move |_| {
                                    if let Ok(mut inner) = app.inner.lock() {
                                        if is_save_mode {
                                            if exists {
                                                // 覆盖已有存档 → 确认弹窗
                                                pending_confirm.set(Some(PendingConfirm {
                                                    message: format!("覆盖 Slot {slot} 的存档？"),
                                                    on_confirm: ActionDef::GoBack, // placeholder
                                                }));
                                                // 直接保存（确认弹窗与保存的联动尚未实现，弹窗仅作视觉提示）
                                                if let Err(e) = inner.save_to_slot(slot) {
                                                    error!(error = %e, slot, "Save failed");
                                                }
                                            } else if let Err(e) = inner.save_to_slot(slot) {
                                                error!(error = %e, slot, "Save failed");
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
                                        class: "save-load__thumb",
                                        src: "data:image/png;base64,{b64}",
                                    }
                                }

                                // 信息区
                                div { class: "save-load__slot-info",
                                    if exists {
                                        if let Some(ref ch) = chapter {
                                            span { class: "save-load__slot-chapter", "{ch}" }
                                        }
                                        if let Some(ref ts) = timestamp {
                                            span { class: "save-load__slot-time", "{ts}" }
                                        }
                                    } else {
                                        span { class: "save-load__slot-empty", "-- 空 --" }
                                    }
                                }

                                // 删除按钮
                                if exists {
                                    button {
                                        class: "save-load__delete-btn",
                                        onclick: move |evt: Event<MouseData>| {
                                            evt.stop_propagation();
                                            if let Ok(inner) = app_del.inner.lock()
                                                && let Err(e) = inner.services().saves.delete(slot) {
                                                    error!(error = %e, slot, "Delete failed");
                                                }
                                        },
                                        "×"
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // 分页导航
            div { class: "save-load__pagination",
                for pk in &all_pages {
                    {
                        let pk_val = *pk;
                        let is_current = pk_val == page;
                        let label = pk_val.label();
                        let class = if is_current {
                            "save-load__page-btn save-load__page-btn--active"
                        } else {
                            "save-load__page-btn"
                        };

                        rsx! {
                            button {
                                key: "{label}",
                                class: "{class}",
                                onclick: move |_| { current_page.set(pk_val); },
                                if is_current { "[ {label} ]" } else { "{label}" }
                            }
                        }
                    }
                }
            }
        }
    }
}

struct SlotInfo {
    slot: u32,
    exists: bool,
    thumb: Option<String>,
    chapter: Option<String>,
    timestamp: Option<String>,
}
