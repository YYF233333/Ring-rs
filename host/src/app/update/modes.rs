//! 各 AppMode 的更新逻辑

use macroquad::prelude::*;
use tracing::debug;
use vn_runtime::input::RuntimeInput;
use vn_runtime::state::WaitingReason;

use super::super::AppState;
use super::super::USER_SETTINGS_PATH;
use super::super::save::{
    load_continue, load_game, quick_load, quick_save, return_to_title_from_game, start_new_game,
};
use crate::PlaybackMode;
use crate::screens::history::HistoryAction;
use crate::screens::ingame_menu::InGameMenuAction;
use crate::screens::save_load::SaveLoadAction;
use crate::screens::settings::SettingsAction;
use crate::screens::title::TitleAction;
use crate::{AppMode, SaveLoadTab};

/// 更新主标题界面
pub(super) fn update_title(app_state: &mut AppState) {
    // 初始化界面
    if app_state.title_screen.needs_init() {
        app_state.title_screen.init(
            &app_state.save_manager,
            &app_state.ui_context.theme,
            app_state.ui_context.screen_width,
            app_state.ui_context.screen_height,
        );
    }

    // 处理用户操作
    match app_state.title_screen.update(&app_state.ui_context) {
        TitleAction::StartGame => {
            // 开始新游戏时删除旧的 Continue 存档
            let _ = app_state.save_manager.delete_continue();
            start_new_game(app_state);
        }
        TitleAction::Continue => {
            // 读取专用 Continue 存档
            if app_state.title_screen.has_continue() {
                load_continue(app_state);
            }
        }
        TitleAction::LoadGame => {
            app_state.save_load_screen =
                crate::screens::SaveLoadScreen::new().with_tab(SaveLoadTab::Load);
            app_state.save_load_screen.mark_needs_init();
            app_state.navigation.navigate_to(AppMode::SaveLoad);
        }
        TitleAction::Settings => {
            app_state.settings_screen.mark_needs_init();
            app_state.navigation.navigate_to(AppMode::Settings);
        }
        TitleAction::Exit => {
            app_state.host_state.stop();
        }
        TitleAction::None => {}
    }
}

/// 更新游戏进行中
pub(super) fn update_ingame(app_state: &mut AppState, dt: f32) {
    // ESC 打开系统菜单（同时退出 Auto/Skip 模式）
    if is_key_pressed(KeyCode::Escape) {
        app_state.playback_mode = PlaybackMode::Normal;
        app_state.auto_timer = 0.0;
        app_state.ingame_menu.mark_needs_init();
        app_state.navigation.navigate_to(AppMode::InGameMenu);
        return;
    }

    // 开发者快捷键（后续考虑 feature gate）
    #[cfg(debug_assertions)]
    {
        if is_key_pressed(KeyCode::F5) {
            quick_save(app_state);
        }
        if is_key_pressed(KeyCode::F9) {
            quick_load(app_state);
        }
    }

    // --- 播放推进模式检测 ---

    // A 键切换 Auto 模式
    if is_key_pressed(KeyCode::A) {
        app_state.playback_mode = match app_state.playback_mode {
            PlaybackMode::Normal => {
                debug!("切换到 Auto 模式");
                PlaybackMode::Auto
            }
            PlaybackMode::Auto => {
                debug!("退出 Auto 模式");
                PlaybackMode::Normal
            }
            // Skip 是临时模式，不受 A 键影响
            PlaybackMode::Skip => PlaybackMode::Skip,
        };
        app_state.auto_timer = 0.0;
    }

    // Ctrl 按住 → 临时 Skip 模式（松开恢复）
    let ctrl_held = is_key_down(KeyCode::LeftControl) || is_key_down(KeyCode::RightControl);
    let effective_mode = if ctrl_held {
        PlaybackMode::Skip
    } else {
        app_state.playback_mode
    };

    // --- 按模式分发 ---

    match effective_mode {
        PlaybackMode::Skip => {
            update_ingame_skip(app_state, dt);
        }
        PlaybackMode::Auto => {
            update_ingame_auto(app_state, dt);
        }
        PlaybackMode::Normal => {
            update_ingame_normal(app_state, dt);
        }
    }

    // --- 通用：同步选择索引到 RenderState ---
    if let Some(ref mut choices) = app_state.render_state.choices {
        let choice_rects = app_state.renderer.get_choice_rects(choices.choices.len());
        app_state.input_manager.set_choice_rects(choice_rects);
        choices.selected_index = app_state.input_manager.selected_index;
        choices.hovered_index = app_state.input_manager.hovered_index;
    }

    // --- 通用：更新打字机效果（Skip 模式下打字机已被 skip_all_active_effects 完成） ---
    if let Some(ref dialogue) = app_state.render_state.dialogue
        && !dialogue.is_complete
    {
        app_state.typewriter_timer += dt * app_state.user_settings.text_speed;
        while app_state.typewriter_timer >= 1.0 {
            app_state.typewriter_timer -= 1.0;
            if app_state.render_state.advance_typewriter() {
                break;
            }
        }
    }
}

/// Skip 模式更新：立即完成所有演出并推进
fn update_ingame_skip(app_state: &mut AppState, dt: f32) {
    // 1. 一次性跳过所有活跃效果
    super::skip_all_active_effects(app_state);

    // 2. 如果等待点击，自动推进
    if app_state.waiting_reason == WaitingReason::WaitForClick {
        super::run_script_tick(app_state, Some(RuntimeInput::Click));
        return;
    }

    // 3. 其他等待类型（选择/时间/信号）仍使用正常输入处理
    if let Some(input) = app_state
        .input_manager
        .update(&app_state.waiting_reason, dt)
    {
        super::handle_script_mode_input(app_state, input);
    }
}

/// Auto 模式更新：对话完成后等待 auto_delay 秒自动推进
fn update_ingame_auto(app_state: &mut AppState, dt: f32) {
    // 用户手动输入仍然有效（手动输入优先）
    if let Some(input) = app_state
        .input_manager
        .update(&app_state.waiting_reason, dt)
    {
        // 手动输入时重置 auto 计时器
        app_state.auto_timer = 0.0;
        super::handle_script_mode_input(app_state, input);
        return;
    }

    // Auto 推进条件：
    // - 等待点击
    // - 对话已完成（打字机结束）
    // - 无活跃动画/过渡
    let can_auto_advance = app_state.waiting_reason == WaitingReason::WaitForClick
        && app_state.render_state.is_dialogue_complete()
        && !app_state.animation_system.has_active_animations()
        && !app_state.renderer.transition.is_active()
        && !app_state.renderer.is_scene_transition_active();

    if can_auto_advance {
        app_state.auto_timer += dt;
        if app_state.auto_timer >= app_state.user_settings.auto_delay {
            app_state.auto_timer = 0.0;
            super::run_script_tick(app_state, Some(RuntimeInput::Click));
        }
    } else {
        // 条件不满足时重置计时器
        app_state.auto_timer = 0.0;
    }
}

/// Normal 模式更新：等待用户点击推进（原有行为）
fn update_ingame_normal(app_state: &mut AppState, dt: f32) {
    // 使用 InputManager 处理游戏输入（传入 dt 用于长按快进）
    if let Some(input) = app_state
        .input_manager
        .update(&app_state.waiting_reason, dt)
    {
        super::handle_script_mode_input(app_state, input);
    }
}

/// 更新游戏内菜单
pub(super) fn update_ingame_menu(app_state: &mut AppState) {
    if app_state.ingame_menu.needs_init() {
        app_state.ingame_menu.init(&app_state.ui_context);
    }

    match app_state.ingame_menu.update(&app_state.ui_context) {
        InGameMenuAction::Resume => {
            app_state.navigation.go_back();
        }
        InGameMenuAction::Save => {
            app_state.save_load_screen =
                crate::screens::SaveLoadScreen::new().with_tab(SaveLoadTab::Save);
            app_state.save_load_screen.mark_needs_init();
            app_state.navigation.navigate_to(AppMode::SaveLoad);
        }
        InGameMenuAction::Load => {
            app_state.save_load_screen =
                crate::screens::SaveLoadScreen::new().with_tab(SaveLoadTab::Load);
            app_state.save_load_screen.mark_needs_init();
            app_state.navigation.navigate_to(AppMode::SaveLoad);
        }
        InGameMenuAction::Settings => {
            app_state.settings_screen.mark_needs_init();
            app_state.navigation.navigate_to(AppMode::Settings);
        }
        InGameMenuAction::History => {
            app_state.history_screen.mark_needs_init();
            app_state.navigation.navigate_to(AppMode::History);
        }
        InGameMenuAction::ReturnToTitle => {
            // 用户主动返回，保存 Continue 存档
            return_to_title_from_game(app_state, true);
        }
        InGameMenuAction::Exit => {
            app_state.host_state.stop();
        }
        InGameMenuAction::None => {}
    }
}

/// 更新存档/读档界面
pub(super) fn update_save_load(app_state: &mut AppState) {
    if app_state.save_load_screen.needs_init() {
        app_state
            .save_load_screen
            .init(&app_state.ui_context, &app_state.save_manager);
    }
    if app_state.save_load_screen.needs_refresh() {
        app_state
            .save_load_screen
            .refresh_saves(&app_state.save_manager);
    }

    match app_state.save_load_screen.update(&app_state.ui_context) {
        SaveLoadAction::Back => {
            app_state.navigation.go_back();
        }
        SaveLoadAction::Save(slot) => {
            app_state.current_save_slot = slot;
            quick_save(app_state);
            app_state
                .toast_manager
                .success(format!("已保存到槽位 {}", slot));
            app_state
                .save_load_screen
                .refresh_saves(&app_state.save_manager);
        }
        SaveLoadAction::Load(slot) => {
            load_game(app_state, slot);
            app_state
                .toast_manager
                .success(format!("已读取槽位 {}", slot));
        }
        SaveLoadAction::Delete(slot) => {
            if app_state.save_manager.delete(slot).is_ok() {
                app_state.toast_manager.info(format!("已删除槽位 {}", slot));
                app_state
                    .save_load_screen
                    .refresh_saves(&app_state.save_manager);
            } else {
                app_state.toast_manager.error("删除失败");
            }
        }
        SaveLoadAction::None => {}
    }
}

/// 更新设置界面
pub(super) fn update_settings(app_state: &mut AppState) {
    if app_state.settings_screen.needs_init() {
        app_state
            .settings_screen
            .init(&app_state.ui_context, &app_state.user_settings);
    }

    match app_state.settings_screen.update(&app_state.ui_context) {
        SettingsAction::Back => {
            app_state.navigation.go_back();
        }
        SettingsAction::Apply => {
            // 应用设置
            app_state.user_settings = app_state.settings_screen.settings().clone();

            // 应用音量
            if let Some(ref mut audio) = app_state.audio_manager {
                audio.set_bgm_volume(app_state.user_settings.bgm_volume);
                audio.set_sfx_volume(app_state.user_settings.sfx_volume);
                audio.set_muted(app_state.user_settings.muted);
            }

            // 保存设置
            if let Err(e) = app_state.user_settings.save(USER_SETTINGS_PATH) {
                tracing::warn!(error = %e, "保存用户设置失败");
                app_state.toast_manager.error("设置保存失败");
            } else {
                app_state.toast_manager.success("设置已保存");
            }

            app_state.navigation.go_back();
        }
        SettingsAction::None => {}
    }
}

/// 更新历史界面
pub(super) fn update_history(app_state: &mut AppState) {
    if app_state.history_screen.needs_init()
        && let Some(ref runtime) = app_state.vn_runtime
    {
        app_state
            .history_screen
            .init(&app_state.ui_context, runtime.history());
    }

    match app_state.history_screen.update(&app_state.ui_context) {
        HistoryAction::Back => {
            app_state.navigation.go_back();
        }
        HistoryAction::None => {}
    }
}
