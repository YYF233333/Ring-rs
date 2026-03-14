//! EguiAction 枚举与处理逻辑
//!
//! 将 egui UI 层产生的动作与 AppState 的状态变更解耦。

use host::app::{self, AppState, USER_SETTINGS_PATH};
use host::{AppMode, SaveLoadTab, UserSettings};
use winit::event_loop::ActiveEventLoop;

/// egui UI 层产生的动作
#[derive(Debug, Clone)]
pub enum EguiAction {
    None,
    StartGame,
    ContinueGame,
    NavigateTo(AppMode),
    GoBack,
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
    el: &ActiveEventLoop,
) {
    match action {
        EguiAction::None => {}
        EguiAction::StartGame => {
            let _ = app_state.save_manager.delete_continue();
            app::start_new_game(app_state);
        }
        EguiAction::ContinueGame => {
            app::load_continue(app_state);
        }
        EguiAction::NavigateTo(mode) => {
            app_state.ui.navigation.navigate_to(mode);
        }
        EguiAction::GoBack => {
            app_state.ui.navigation.go_back();
        }
        EguiAction::ReturnToTitle => {
            app::return_to_title_from_game(app_state, true);
        }
        EguiAction::Exit => {
            el.exit();
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
            app_state.ui.navigation.go_back();
        }
        EguiAction::OpenSave => {
            *save_load_tab = SaveLoadTab::Save;
            app_state.ui.navigation.navigate_to(AppMode::SaveLoad);
        }
        EguiAction::OpenLoad => {
            *save_load_tab = SaveLoadTab::Load;
            app_state.ui.navigation.navigate_to(AppMode::SaveLoad);
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
            use host::PlaybackMode;
            app_state.session.playback_mode =
                if app_state.session.playback_mode == PlaybackMode::Skip {
                    PlaybackMode::Normal
                } else {
                    PlaybackMode::Skip
                };
        }
        EguiAction::ToggleAuto => {
            use host::PlaybackMode;
            app_state.session.playback_mode =
                if app_state.session.playback_mode == PlaybackMode::Auto {
                    PlaybackMode::Normal
                } else {
                    PlaybackMode::Auto
                };
        }
        EguiAction::ShowConfirm { .. } => {
            // Handled in host_app.rs before reaching here
        }
    }
}
