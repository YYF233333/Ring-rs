//! 共享 UI 帧构建逻辑
//!
//! 将 egui 屏幕的模式分发逻辑抽取为独立函数，
//! 供 windowed（`host_app`）和 headless 模式共用。

use std::collections::HashMap;

use crate::app::AppState;
use crate::egui_actions::EguiAction;
use crate::egui_screens;
use crate::egui_screens::confirm::ConfirmDialog;
use crate::save_manager::SaveInfo;
use crate::ui_modes::{ActiveUiMode, UiModeStatus};
use crate::{AppMode, PlaybackMode, SaveLoadPage, SaveLoadTab, UiRenderContext, UserSettings};
use vn_runtime::state::VarValue;

/// UI 帧构建所需的跨帧持久状态
pub struct UiFrameState {
    pub settings_draft: Option<UserSettings>,
    pub save_load_tab: SaveLoadTab,
    pub save_load_page: SaveLoadPage,
    pub pending_confirm: Option<ConfirmDialog>,
}

impl Default for UiFrameState {
    fn default() -> Self {
        Self {
            settings_draft: None,
            save_load_tab: SaveLoadTab::Load,
            save_load_page: SaveLoadPage::default(),
            pending_confirm: None,
        }
    }
}

/// 构建所有 UI 屏幕并返回动作
///
/// `slot_thumbnails` 在 headless 模式下传空 HashMap（无 GPU 纹理）。
///
/// 返回 `(EguiAction, confirm_resolved)`。
pub fn build_frame_ui(
    ctx: &egui::Context,
    app_state: &AppState,
    ui_ctx: &UiRenderContext,
    frame_state: &mut UiFrameState,
    slot_thumbnails: &HashMap<u32, egui::TextureHandle>,
) -> (EguiAction, bool) {
    let current_mode = app_state.ui.navigation.current();

    let mut action = if app_state.core.video_player.is_playing() {
        EguiAction::None
    } else {
        let save_infos: Vec<Option<SaveInfo>> = if current_mode == AppMode::SaveLoad {
            frame_state
                .save_load_page
                .slot_range()
                .map(|s| app_state.save_manager.get_save_info(s))
                .collect()
        } else {
            Vec::new()
        };
        let can_save = app_state.session.vn_runtime.is_some();
        let sl_tab = frame_state.save_load_tab;
        let history_events = app_state.session.history_events();

        match current_mode {
            AppMode::Title => egui_screens::title::build_title_ui(ctx, ui_ctx),
            AppMode::InGame => {
                egui_screens::ingame::build_ingame_ui(ctx, &app_state.core.render_state, ui_ctx)
            }
            AppMode::InGameMenu => egui_screens::ingame_menu::build_ingame_menu_ui(ctx, ui_ctx),
            AppMode::Settings => {
                egui_screens::game_menu::build_game_menu_frame(ctx, "设置", ui_ctx, |ui| {
                    egui_screens::settings::build_settings_content(
                        ui,
                        &mut frame_state.settings_draft,
                        ui_ctx,
                    )
                })
            }
            AppMode::SaveLoad => egui_screens::game_menu::build_game_menu_frame(
                ctx,
                if sl_tab == SaveLoadTab::Save {
                    "保存"
                } else {
                    "读取"
                },
                ui_ctx,
                |ui| {
                    egui_screens::save_load::build_save_load_content(
                        ui,
                        sl_tab,
                        &mut frame_state.save_load_page,
                        &save_infos,
                        can_save,
                        ui_ctx,
                        slot_thumbnails,
                    )
                },
            ),
            AppMode::History => {
                egui_screens::game_menu::build_game_menu_frame(ctx, "历史", ui_ctx, |ui| {
                    egui_screens::history::build_history_content(ui, history_events, ui_ctx)
                })
            }
        }
    };

    if current_mode == AppMode::InGame && app_state.session.playback_mode == PlaybackMode::Skip {
        egui_screens::skip_indicator::build_skip_indicator(ctx, ui_ctx);
    }

    let mut confirm_resolved = false;
    if let Some(dialog) = frame_state.pending_confirm.as_ref()
        && let Some(confirm_action) =
            egui_screens::confirm::build_confirm_overlay(ctx, dialog, ui_ctx)
    {
        action = confirm_action;
        confirm_resolved = true;
    }

    egui_screens::toast::build_toast_overlay(ctx, &app_state.ui.toast_manager, ui_ctx);

    (action, confirm_resolved)
}

/// 渲染活跃的 UI 模式，返回完成结果（如有）
///
/// 在 egui context 内调用。调用方负责管理 `ActiveUiMode` 的 take/restore 生命周期。
pub fn render_active_ui_mode(
    ctx: &egui::Context,
    active: &mut ActiveUiMode,
    scale: &crate::ui::layout::ScaleContext,
) -> Option<(String, VarValue)> {
    match active.handler.render(ctx, scale) {
        UiModeStatus::Active => None,
        UiModeStatus::Completed(value) => Some((active.key.clone(), value)),
        UiModeStatus::Cancelled => Some((active.key.clone(), VarValue::String(String::new()))),
    }
}
