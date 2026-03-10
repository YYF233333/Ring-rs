//! 各 AppMode 的更新逻辑
//!
//! Title / InGameMenu / SaveLoad / Settings / History 的 UI 交互
//! 由 egui 层处理；此处仅保留 InGame 的游戏逻辑。

use tracing::debug;
use vn_runtime::input::RuntimeInput;
use vn_runtime::state::WaitingReason;
use winit::keyboard::KeyCode;

use super::super::AppState;
use super::super::save::{quick_load, quick_save};
use crate::AppMode;
use crate::PlaybackMode;

/// 更新主标题界面（UI 由 egui 驱动，此处为 no-op）
pub(super) fn update_title(_app_state: &mut AppState) {}

/// 更新游戏进行中
pub(super) fn update_ingame(app_state: &mut AppState, dt: f32) {
    // ESC 打开系统菜单（同时退出 Auto/Skip 模式）
    if app_state.input_manager.is_key_just_pressed(KeyCode::Escape) {
        app_state.session.playback_mode = PlaybackMode::Normal;
        app_state.session.auto_timer = 0.0;
        app_state.ui.navigation.navigate_to(AppMode::InGameMenu);
        return;
    }

    #[cfg(debug_assertions)]
    {
        if app_state.input_manager.is_key_just_pressed(KeyCode::F5) {
            quick_save(app_state);
        }
        if app_state.input_manager.is_key_just_pressed(KeyCode::F9) {
            quick_load(app_state);
        }
    }

    // --- 播放推进模式检测 ---

    // A 键切换 Auto 模式
    if app_state.input_manager.is_key_just_pressed(KeyCode::KeyA) {
        app_state.session.playback_mode = match app_state.session.playback_mode {
            PlaybackMode::Normal => {
                debug!("切换到 Auto 模式");
                PlaybackMode::Auto
            }
            PlaybackMode::Auto => {
                debug!("退出 Auto 模式");
                PlaybackMode::Normal
            }
            PlaybackMode::Skip => PlaybackMode::Skip,
        };
        app_state.session.auto_timer = 0.0;
    }

    // Ctrl 按住 -> 临时 Skip 模式（松开恢复）
    let ctrl_held = app_state.input_manager.is_key_down(KeyCode::ControlLeft)
        || app_state.input_manager.is_key_down(KeyCode::ControlRight);
    let effective_mode = if ctrl_held {
        PlaybackMode::Skip
    } else {
        app_state.session.playback_mode
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
    if let Some(ref mut choices) = app_state.core.render_state.choices {
        let choice_rects = app_state
            .core
            .renderer
            .get_choice_rects(choices.choices.len());
        app_state.input_manager.set_choice_rects(choice_rects);
        choices.selected_index = app_state.input_manager.selected_index;
        choices.hovered_index = app_state.input_manager.hovered_index;
    }

    // --- 通用：更新打字机效果（Skip 模式下打字机已被 skip_all_active_effects 完成） ---
    if let Some(ref dialogue) = app_state.core.render_state.dialogue
        && !dialogue.is_complete
    {
        if app_state.core.render_state.has_inline_wait() {
            app_state.core.render_state.update_inline_wait(dt);
        } else {
            let effective_speed = app_state
                .core
                .render_state
                .effective_text_speed(app_state.user_settings.text_speed);
            app_state.session.typewriter_timer += dt * effective_speed;
            while app_state.session.typewriter_timer >= 1.0 {
                app_state.session.typewriter_timer -= 1.0;
                if app_state.core.render_state.advance_typewriter() {
                    break;
                }
                if app_state.core.render_state.has_inline_wait() {
                    break;
                }
            }
        }
    }

    // --- 通用：no_wait 自动推进 ---
    if app_state.session.waiting_reason == WaitingReason::WaitForClick
        && app_state.core.render_state.is_dialogue_complete()
        && app_state
            .core
            .render_state
            .dialogue
            .as_ref()
            .is_some_and(|d| d.no_wait)
    {
        super::run_script_tick(app_state, Some(RuntimeInput::Click));
    }
}

/// Skip 模式更新：立即完成所有演出并推进
fn update_ingame_skip(app_state: &mut AppState, dt: f32) {
    let typewriter_was_incomplete = !app_state.core.render_state.is_dialogue_complete();

    super::skip_all_active_effects(&mut app_state.core);

    if typewriter_was_incomplete {
        return;
    }

    if app_state.session.waiting_reason == WaitingReason::WaitForClick {
        super::run_script_tick(app_state, Some(RuntimeInput::Click));
        return;
    }

    if matches!(
        app_state.session.waiting_reason,
        WaitingReason::WaitForTime(_)
    ) {
        app_state.session.wait_timer = 0.0;
        super::run_script_tick(app_state, Some(RuntimeInput::Click));
        return;
    }

    if let Some(input) = app_state
        .input_manager
        .update(&app_state.session.waiting_reason, dt)
    {
        super::handle_script_mode_input(app_state, input);
    }
}

/// Auto 模式更新：对话完成后等待 auto_delay 秒自动推进
fn update_ingame_auto(app_state: &mut AppState, dt: f32) {
    if let Some(input) = app_state
        .input_manager
        .update(&app_state.session.waiting_reason, dt)
    {
        app_state.session.auto_timer = 0.0;
        super::handle_script_mode_input(app_state, input);
        return;
    }

    let can_auto_advance = app_state.session.waiting_reason == WaitingReason::WaitForClick
        && app_state.core.render_state.is_dialogue_complete()
        && !app_state.core.animation_system.has_active_animations()
        && !app_state.core.renderer.transition.is_active()
        && !app_state.core.renderer.is_scene_transition_active();

    if can_auto_advance {
        app_state.session.auto_timer += dt;
        if app_state.session.auto_timer >= app_state.user_settings.auto_delay {
            app_state.session.auto_timer = 0.0;
            super::run_script_tick(app_state, Some(RuntimeInput::Click));
        }
    } else {
        app_state.session.auto_timer = 0.0;
    }
}

/// Normal 模式更新：等待用户点击推进（原有行为）
fn update_ingame_normal(app_state: &mut AppState, dt: f32) {
    if let Some(input) = app_state
        .input_manager
        .update(&app_state.session.waiting_reason, dt)
    {
        super::handle_script_mode_input(app_state, input);
    }
}

/// 更新游戏内菜单（UI 由 egui 驱动，此处为 no-op）
pub(super) fn update_ingame_menu(_app_state: &mut AppState) {}

/// 更新存档/读档界面（UI 由 egui 驱动，此处为 no-op）
pub(super) fn update_save_load(_app_state: &mut AppState) {}

/// 更新设置界面（UI 由 egui 驱动，此处为 no-op）
pub(super) fn update_settings(_app_state: &mut AppState) {}

/// 更新历史界面（UI 由 egui 驱动，此处为 no-op）
pub(super) fn update_history(_app_state: &mut AppState) {}
