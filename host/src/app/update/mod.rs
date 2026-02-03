//! 更新逻辑（聚合入口）
//!
//! 目标：把之前的“巨型 update.rs”按职责拆分：
//! - `modes`: 各 AppMode 的更新逻辑（Title/InGame/Menu/SaveLoad/Settings/History）
//! - `script`: VNRuntime tick 与脚本输入处理
//! - `scene_transition`: changeScene 相关的多阶段过渡驱动

mod modes;
mod scene_transition;
mod script;

pub use scene_transition::update_scene_transition;
pub use script::{handle_script_mode_input, run_script_tick};

use macroquad::prelude::*;

use super::AppState;
use crate::AppMode;

/// 更新入口（每帧调用）
pub fn update(app_state: &mut AppState) {
    let dt = get_frame_time();

    // 更新 UI 上下文
    app_state.ui_context.update();

    // 更新 Toast
    app_state.toast_manager.update(dt);

    // 切换调试模式（全局可用）
    if is_key_pressed(KeyCode::F1) {
        app_state.host_state.debug_mode = !app_state.host_state.debug_mode;
        println!(
            "🔧 调试模式: {}",
            if app_state.host_state.debug_mode {
                "开启"
            } else {
                "关闭"
            }
        );
    }

    // 根据当前模式处理更新
    let current_mode = app_state.navigation.current();
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
        app_state.command_executor.update_transition(dt);
        app_state.renderer.update_transition(dt);

        // 更新场景过渡状态（基于动画系统）
        update_scene_transition(&mut app_state.renderer, &mut app_state.render_state, dt);

        // 更新动画系统
        let _events = app_state.animation_system.update(dt);

        // 检测淡出完成的角色并移除
        let completed_fadeouts: Vec<String> = app_state
            .render_state
            .visible_characters
            .iter()
            .filter(|(_alias, char)| {
                // 检查角色是否标记为淡出且透明度已降到 0
                if char.fading_out {
                    let alpha = char.anim.alpha();
                    alpha <= 0.01
                } else {
                    false
                }
            })
            .map(|(alias, _)| alias.clone())
            .collect();

        // 移除淡出完成的角色，并从动画系统注销
        for alias in &completed_fadeouts {
            if let Some(object_id) = app_state.character_object_ids.remove(alias) {
                app_state.animation_system.unregister(object_id);
            }
        }
        app_state
            .render_state
            .remove_fading_out_characters(&completed_fadeouts);
    }

    // 更新音频状态（所有模式都需要）
    if let Some(ref mut audio_manager) = app_state.audio_manager {
        audio_manager.update(dt);
    }
}
