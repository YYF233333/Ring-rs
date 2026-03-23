//! EguiAction 枚举与处理逻辑
//!
//! 将 egui UI 层产生的动作与 AppState 的状态变更解耦。

use crate::app::{self, AppState, USER_SETTINGS_PATH};
use crate::ui::screen_defs::{ActionDef, ButtonDef};
use crate::{AppMode, SaveLoadTab, UserSettings};
use winit::event_loop::ActiveEventLoop;

/// egui UI 层产生的动作
#[derive(Debug, Clone)]
pub enum EguiAction {
    None,
    StartGame,
    /// 从指定标签开始新游戏（泛化的章节入口）
    StartAtLabel(String),
    ContinueGame,
    NavigateTo(AppMode),
    /// 替换当前模式（不压栈），用于同级页面间切换
    ReplaceTo(AppMode),
    GoBack,
    /// 清空导航栈，直接回到 InGame
    ReturnToGame,
    ReturnToTitle,
    Exit,
    ApplySettings(UserSettings),
    OpenSave,
    OpenLoad,
    SaveToSlot(u32),
    LoadFromSlot(u32),
    DeleteSlot(u32),
    QuickSave,
    QuickLoad,
    ToggleSkip,
    ToggleAuto,
    ShowConfirm {
        message: String,
        on_confirm: Box<EguiAction>,
    },
}

pub fn handle_egui_action(
    app_state: &mut AppState,
    action: EguiAction,
    save_load_tab: &mut SaveLoadTab,
    el: Option<&ActiveEventLoop>,
) {
    match action {
        EguiAction::None => {}
        EguiAction::StartGame => {
            let _ = app_state.save_manager.delete_continue();
            app::start_new_game(app_state);
        }
        EguiAction::StartAtLabel(ref label) => {
            let _ = app_state.save_manager.delete_continue();
            app::start_new_game_at_label(app_state, label);
        }
        EguiAction::ContinueGame => {
            app::load_continue(app_state);
        }
        EguiAction::NavigateTo(mode) => {
            app_state.ui.navigation.navigate_to(mode);
        }
        EguiAction::ReplaceTo(mode) => {
            app_state.ui.navigation.replace_current(mode);
        }
        EguiAction::GoBack => {
            app_state.ui.navigation.go_back();
        }
        EguiAction::ReturnToGame => {
            app_state.ui.navigation.switch_to(AppMode::InGame);
        }
        EguiAction::ReturnToTitle => {
            app::return_to_title_from_game(app_state, true);
        }
        EguiAction::Exit => {
            if let Some(el) = el {
                el.exit();
            } else {
                app_state.host_state.exit_requested = true;
            }
        }
        EguiAction::ApplySettings(new_settings) => {
            app_state.user_settings = new_settings;
            if let Some(ref mut audio) = app_state.core.audio_manager {
                audio.set_bgm_volume(app_state.user_settings.bgm_volume);
                audio.set_sfx_volume(app_state.user_settings.sfx_volume);
                audio.set_muted(app_state.user_settings.muted);
            }
            if let Err(e) = app_state.user_settings.save(USER_SETTINGS_PATH) {
                tracing::warn!(error = %e, "Failed to save user settings");
                app_state.ui.toast_manager.error("Settings save failed");
            } else {
                app_state.ui.toast_manager.success("Settings saved");
            }
        }
        EguiAction::OpenSave => {
            *save_load_tab = SaveLoadTab::Save;
            let cur = app_state.ui.navigation.current();
            if matches!(
                cur,
                AppMode::SaveLoad | AppMode::Settings | AppMode::History
            ) {
                app_state.ui.navigation.replace_current(AppMode::SaveLoad);
            } else {
                app_state.ui.navigation.navigate_to(AppMode::SaveLoad);
            }
        }
        EguiAction::OpenLoad => {
            *save_load_tab = SaveLoadTab::Load;
            let cur = app_state.ui.navigation.current();
            if matches!(
                cur,
                AppMode::SaveLoad | AppMode::Settings | AppMode::History
            ) {
                app_state.ui.navigation.replace_current(AppMode::SaveLoad);
            } else {
                app_state.ui.navigation.navigate_to(AppMode::SaveLoad);
            }
        }
        EguiAction::SaveToSlot(slot) => {
            app_state.current_save_slot = slot;
            app::quick_save(app_state);
            app_state
                .ui
                .toast_manager
                .success(format!("Saved to slot {slot}"));
        }
        EguiAction::LoadFromSlot(slot) => {
            app::load_game(app_state, slot);
            app_state
                .ui
                .toast_manager
                .success(format!("Loaded slot {slot}"));
        }
        EguiAction::DeleteSlot(slot) => {
            if app_state.save_manager.delete(slot).is_ok() {
                app_state
                    .ui
                    .toast_manager
                    .info(format!("Deleted slot {slot}"));
            } else {
                app_state.ui.toast_manager.error("Delete failed");
            }
        }
        EguiAction::QuickSave => {
            app::quick_save(app_state);
            app_state.ui.toast_manager.success("Quick saved");
        }
        EguiAction::QuickLoad => {
            app::load_continue(app_state);
            app_state.ui.toast_manager.success("Quick loaded");
        }
        EguiAction::ToggleSkip => {
            use crate::PlaybackMode;
            app_state.session.playback_mode =
                if app_state.session.playback_mode == PlaybackMode::Skip {
                    PlaybackMode::Normal
                } else {
                    PlaybackMode::Skip
                };
        }
        EguiAction::ToggleAuto => {
            use crate::PlaybackMode;
            app_state.session.playback_mode =
                if app_state.session.playback_mode == PlaybackMode::Auto {
                    PlaybackMode::Normal
                } else {
                    PlaybackMode::Auto
                };
        }
        EguiAction::ShowConfirm { .. } => {
            unreachable!("ShowConfirm must be intercepted by the caller before handle_egui_action")
        }
    }
}

// ─── ActionDef → EguiAction 转换 ─────────────────────────────────────────────

/// 将声明式 [`ActionDef`] 转换为 [`EguiAction`]
pub fn action_def_to_egui(action: &ActionDef) -> EguiAction {
    match action {
        ActionDef::StartGame => EguiAction::StartGame,
        ActionDef::ContinueGame => EguiAction::ContinueGame,
        ActionDef::OpenLoad => EguiAction::OpenLoad,
        ActionDef::OpenSave => EguiAction::OpenSave,
        ActionDef::NavigateSettings => EguiAction::NavigateTo(AppMode::Settings),
        ActionDef::NavigateHistory => EguiAction::NavigateTo(AppMode::History),
        ActionDef::ReplaceSettings => EguiAction::ReplaceTo(AppMode::Settings),
        ActionDef::ReplaceHistory => EguiAction::ReplaceTo(AppMode::History),
        ActionDef::QuickSave => EguiAction::QuickSave,
        ActionDef::QuickLoad => EguiAction::QuickLoad,
        ActionDef::ToggleSkip => EguiAction::ToggleSkip,
        ActionDef::ToggleAuto => EguiAction::ToggleAuto,
        ActionDef::GoBack => EguiAction::GoBack,
        ActionDef::ReturnToTitle => EguiAction::ReturnToTitle,
        ActionDef::ReturnToGame => EguiAction::ReturnToGame,
        ActionDef::Exit => EguiAction::Exit,
        ActionDef::StartAtLabel(label) => EguiAction::StartAtLabel(label.clone()),
    }
}

/// 将 [`ButtonDef`] 转换为 [`EguiAction`]，自动处理 `confirm` 包装
pub fn button_def_to_egui(button: &ButtonDef) -> EguiAction {
    let base = action_def_to_egui(&button.action);
    match &button.confirm {
        Some(message) => EguiAction::ShowConfirm {
            message: message.clone(),
            on_confirm: Box::new(base),
        },
        None => base,
    }
}
