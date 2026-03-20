//! 各 AppMode 的更新逻辑
//!
//! Title / InGameMenu / SaveLoad / Settings / History 的 UI 交互
//! 由 egui 层处理；此处仅保留 InGame 的游戏逻辑。
//!
//! InGame 分为两块：`update_ingame`（输入与推进模式分支）、`tick_ingame_shared`（每帧共享的状态推进：过渡、信号检测、动画、清理）。

use tracing::debug;
use vn_runtime::command::{
    SIGNAL_CUTSCENE, SIGNAL_SCENE_EFFECT, SIGNAL_SCENE_TRANSITION, SIGNAL_TITLE_CARD,
};
use vn_runtime::input::RuntimeInput;
use vn_runtime::state::WaitingReason;
use winit::keyboard::KeyCode;

use super::super::AppState;
use super::{finish_cutscene, run_script_tick, update_scene_transition};
use crate::AppMode;
use crate::PlaybackMode;

/// 更新主标题界面（UI 由 egui 驱动，此处为 no-op）
pub(super) fn update_title(_app_state: &mut AppState) {}

/// 更新游戏内菜单（UI 由 egui 驱动，此处为 no-op）
pub(super) fn update_ingame_menu(_app_state: &mut AppState) {}

/// 更新存档/读档界面（UI 由 egui 驱动，此处为 no-op）
pub(super) fn update_save_load(_app_state: &mut AppState) {}

/// 更新设置界面（UI 由 egui 驱动，此处为 no-op）
pub(super) fn update_settings(_app_state: &mut AppState) {}

/// 更新历史界面（UI 由 egui 驱动，此处为 no-op）
pub(super) fn update_history(_app_state: &mut AppState) {}

/// 更新游戏进行中
pub(super) fn update_ingame(app_state: &mut AppState, dt: f32) {
    // 视频播放中：拦截所有正常输入，仅响应跳过操作
    if app_state.core.video_player.is_playing() {
        let skip_requested = app_state.input_manager.is_key_just_pressed(KeyCode::Escape)
            || app_state.input_manager.is_key_just_pressed(KeyCode::Enter)
            || app_state.input_manager.is_key_just_pressed(KeyCode::Space)
            || app_state.input_manager.is_mouse_just_pressed()
            || matches!(app_state.session.playback_mode, PlaybackMode::Skip)
            || app_state.input_manager.is_key_down(KeyCode::ControlLeft)
            || app_state.input_manager.is_key_down(KeyCode::ControlRight);

        if skip_requested {
            debug!("用户请求跳过 cutscene");
            app_state.core.video_player.skip();
            finish_cutscene(app_state);
        }
        return;
    }

    // ESC 打开系统菜单（同时退出 Auto/Skip 模式）
    if app_state.input_manager.is_key_just_pressed(KeyCode::Escape) {
        app_state.session.playback_mode = PlaybackMode::Normal;
        app_state.session.auto_timer = 0.0;
        app_state.ui.navigation.navigate_to(AppMode::InGameMenu);
        return;
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

    update_ingame_common(app_state, dt);
}

/// InGame 下通用的打字机/选择框/no_wait 更新逻辑
///
/// 从 `update_ingame` 提取，供 GUI 和 headless 共用。
fn update_ingame_common(app_state: &mut AppState, dt: f32) {
    // 同步选择索引到 RenderState
    if let Some(ref mut choices) = app_state.core.render_state.choices {
        let choice_rects = app_state
            .core
            .renderer
            .get_choice_rects(choices.choices.len());
        app_state.input_manager.set_choice_rects(choice_rects);
        choices.selected_index = app_state.input_manager.choice.selected_index;
        choices.hovered_index = app_state.input_manager.choice.hovered_index;
    }

    // 更新打字机效果
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

    // no_wait 自动推进
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

/// InGame 下每帧共享更新（过渡、信号检测、动画、清理）
///
/// 由 `update::update()` 在模式分发后、当 `current_mode.is_in_game()` 时调用。
/// 不包含输入与推进模式分支，仅负责与时间推进相关的状态更新。
pub(super) fn tick_ingame_shared(app_state: &mut AppState, dt: f32) {
    let had_transition = app_state.core.renderer.transition.is_active();

    // 更新过渡效果
    app_state.core.renderer.update_transition(dt);

    if had_transition && !app_state.core.renderer.transition.is_active() {
        app_state
            .event_stream
            .on_transition_update("dissolve", "completed", 1.0);
    }

    // 更新场景过渡状态（基于动画系统）
    update_scene_transition(
        &mut app_state.core.renderer,
        &mut app_state.core.render_state,
        dt,
    );

    // WaitForTime 计时推进：每帧递减 wait_timer，到期后自动解除等待
    if matches!(
        app_state.session.waiting_reason,
        WaitingReason::WaitForTime(_)
    ) {
        app_state.session.wait_timer -= dt;
        if app_state.session.wait_timer <= 0.0 {
            app_state.session.wait_timer = 0.0;
            run_script_tick(app_state, Some(RuntimeInput::Click));
        }
    }

    // changeScene 过渡完成检测：当 Runtime 等待 scene_transition 信号
    // 且所有过渡动画均已结束时，自动发送信号解除等待
    if let WaitingReason::WaitForSignal(ref id) = app_state.session.waiting_reason
        && id.as_str() == SIGNAL_SCENE_TRANSITION
        && !app_state.core.renderer.is_scene_transition_active()
        && !app_state.core.renderer.transition.is_active()
    {
        let signal_id = id.clone();
        run_script_tick(app_state, Some(RuntimeInput::Signal { id: signal_id }));
    }

    // 场景效果更新（shake/blur 动画推进）
    app_state
        .core
        .renderer
        .update_scene_effects(dt, &mut app_state.core.render_state.scene_effect);

    // sceneEffect 完成检测：当 Runtime 等待 scene_effect 信号
    // 且所有场景效果动画均已结束时，自动发送信号
    if let WaitingReason::WaitForSignal(ref id) = app_state.session.waiting_reason
        && id.as_str() == SIGNAL_SCENE_EFFECT
        && !app_state.core.renderer.is_scene_effect_active()
    {
        let signal_id = id.clone();
        run_script_tick(app_state, Some(RuntimeInput::Signal { id: signal_id }));
    }

    // titleCard 计时更新与完成检测
    if let Some(ref mut tc) = app_state.core.render_state.title_card {
        tc.elapsed += dt;
        if tc.elapsed >= tc.duration {
            app_state.core.render_state.title_card = None;
            if let WaitingReason::WaitForSignal(ref id) = app_state.session.waiting_reason
                && id.as_str() == SIGNAL_TITLE_CARD
            {
                let signal_id = id.clone();
                run_script_tick(app_state, Some(RuntimeInput::Signal { id: signal_id }));
            }
        }
    }

    // cutscene 视频播放推进
    if let WaitingReason::WaitForSignal(ref id) = app_state.session.waiting_reason
        && id.as_str() == SIGNAL_CUTSCENE
    {
        if app_state.core.video_player.is_playing() {
            app_state.core.video_player.update(dt);
            try_start_video_audio(app_state);
            if app_state.core.video_player.is_done() {
                finish_cutscene(app_state);
            }
        } else {
            finish_cutscene(app_state);
        }
    }

    // 更新章节标记动画（非阻塞、不受快进影响、固定时间自动消失）
    app_state.core.render_state.update_chapter_mark(dt);

    // 更新动画系统
    let _events = app_state.core.animation_system.update(dt);

    // 检测淡出完成的角色并移除
    super::cleanup_fading_characters(&mut app_state.core);
}

/// 检查视频音频提取是否完成，如完成则通过 AudioManager 播放。
fn try_start_video_audio(app_state: &mut AppState) {
    let Some(audio_mod) = app_state.core.video_player.audio_mut() else {
        return;
    };
    audio_mod.try_start_playback();

    let channels = audio_mod.channels();
    let sample_rate = audio_mod.sample_rate();
    let Some(samples) = audio_mod.take_samples() else {
        return;
    };

    if let Some(ref audio_manager) = app_state.core.audio_manager {
        let _player = audio_manager.play_video_audio(samples, channels, sample_rate);
    }
}
