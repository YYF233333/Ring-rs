//! 更新逻辑
//!
//! - `modes`: 各 AppMode 的更新逻辑（Title/InGame/Menu/SaveLoad/Settings/History）
//! - `script`: VNRuntime tick 与脚本输入处理
//! - `scene_transition`: changeScene 相关的多阶段过渡驱动

mod modes;
mod scene_transition;
mod script;

pub use scene_transition::update_scene_transition;
pub use script::{
    finish_cutscene, handle_script_mode_input, run_script_tick, skip_all_active_effects,
};

use tracing::debug;
use vn_runtime::command::{
    SIGNAL_CUTSCENE, SIGNAL_SCENE_EFFECT, SIGNAL_SCENE_TRANSITION, SIGNAL_TITLE_CARD,
};
use vn_runtime::input::RuntimeInput;
use vn_runtime::state::WaitingReason;
use winit::keyboard::KeyCode;

use super::AppState;
use crate::AppMode;

/// 角色淡出完成判定阈值（alpha <= 此值视为完成）
const FADEOUT_ALPHA_THRESHOLD: f32 = 0.01;

/// 更新入口（每帧调用）
///
/// `dt` 由外部（winit 帧间隔）提供。
pub fn update(app_state: &mut AppState, dt: f32) {
    // 更新 UI 上下文
    app_state.ui.ui_context.update();

    // 更新 Toast
    app_state.ui.toast_manager.update(dt);

    // 切换调试模式（全局可用）
    if app_state.input_manager.is_key_just_pressed(KeyCode::F1) {
        app_state.host_state.debug_mode = !app_state.host_state.debug_mode;
        debug!(
            enabled = app_state.host_state.debug_mode,
            "Debug mode toggled"
        );
    }

    // 根据当前模式处理更新
    let current_mode = app_state.ui.navigation.current();
    match current_mode {
        AppMode::Title => modes::update_title(app_state),
        AppMode::InGame => modes::update_ingame(app_state, dt),
        AppMode::InGameMenu => modes::update_ingame_menu(app_state),
        AppMode::SaveLoad => modes::update_save_load(app_state),
        AppMode::Settings => modes::update_settings(app_state),
        AppMode::History => modes::update_history(app_state),
    }

    // 游戏进行时的通用更新（过渡效果、音频等）
    if current_mode.is_in_game() {
        // 更新过渡效果
        app_state.core.renderer.update_transition(dt);

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
            && id == SIGNAL_SCENE_TRANSITION
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
            && id == SIGNAL_SCENE_EFFECT
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
                    && id == SIGNAL_TITLE_CARD
                {
                    let signal_id = id.clone();
                    run_script_tick(app_state, Some(RuntimeInput::Signal { id: signal_id }));
                }
            }
        }

        // cutscene 视频播放推进
        if let WaitingReason::WaitForSignal(ref id) = app_state.session.waiting_reason {
            if id == SIGNAL_CUTSCENE {
                if app_state.video_player.is_playing() {
                    // 推进视频帧
                    app_state.video_player.update(dt);

                    // 尝试启动音频播放
                    try_start_video_audio(app_state);

                    if app_state.video_player.is_done() {
                        finish_cutscene(app_state);
                    }
                } else {
                    // 视频未在播放（启动失败或已完成），立即发信号跳过
                    finish_cutscene(app_state);
                }
            }
        }

        // 更新章节标记动画（非阻塞、不受快进影响、固定时间自动消失）
        app_state.core.render_state.update_chapter_mark(dt);

        // 更新动画系统
        let _events = app_state.core.animation_system.update(dt);

        // 检测淡出完成的角色并移除
        let completed_fadeouts: Vec<String> = app_state
            .core
            .render_state
            .visible_characters
            .iter()
            .filter(|(_alias, char)| {
                // 检查角色是否标记为淡出且透明度已降到 0
                if char.fading_out {
                    let alpha = char.anim.alpha();
                    alpha <= FADEOUT_ALPHA_THRESHOLD
                } else {
                    false
                }
            })
            .map(|(alias, _)| alias.clone())
            .collect();

        // 移除淡出完成的角色，并从动画系统注销
        for alias in &completed_fadeouts {
            if let Some(object_id) = app_state.core.character_object_ids.remove(alias) {
                app_state.core.animation_system.unregister(object_id);
            }
        }
        app_state
            .core
            .render_state
            .remove_fading_out_characters(&completed_fadeouts);
    }

    // 更新音频状态（所有模式都需要）
    if let Some(ref mut audio_manager) = app_state.core.audio_manager {
        audio_manager.update(dt);
    }
}

/// 检查视频音频提取是否完成，如完成则通过 AudioManager 播放。
fn try_start_video_audio(app_state: &mut AppState) {
    let Some(audio_mod) = app_state.video_player.audio_mut() else {
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
