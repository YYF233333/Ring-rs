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
use winit::keyboard::KeyCode;

use super::AppState;
use super::CoreSystems;
use crate::AppMode;

/// 角色淡出完成判定阈值（alpha <= 此值视为完成；skip_all 后对象 alpha 已置 0，故与每帧清理可共用此条件）
const FADEOUT_ALPHA_THRESHOLD: f32 = 0.01;

/// 清理淡出完成的角色（从动画系统注销并从 render_state 移除）
///
/// 统一条件：`fading_out && anim.alpha() <= FADEOUT_ALPHA_THRESHOLD`。
/// 每帧更新时仅透明度已淡到底的会被清理；skip 后 `skip_all()` 已将 alpha 置 0，同样满足条件。
pub(super) fn cleanup_fading_characters(core: &mut CoreSystems) {
    let completed: Vec<String> = core
        .render_state
        .visible_characters
        .iter()
        .filter(|(_, c)| c.fading_out && c.anim.alpha() <= FADEOUT_ALPHA_THRESHOLD)
        .map(|(alias, _)| alias.clone())
        .collect();

    for alias in &completed {
        if let Some(object_id) = core.character_object_ids.remove(alias) {
            core.animation_system.unregister(object_id);
        }
    }
    core.render_state.remove_fading_out_characters(&completed);
}

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

    // 游戏进行时的通用更新（过渡效果、信号检测、动画、清理）
    if current_mode.is_in_game() {
        modes::tick_ingame_shared(app_state, dt);
    }

    // 更新音频状态（所有模式都需要）
    if let Some(ref mut audio_manager) = app_state.core.audio_manager {
        audio_manager.update(dt);
    }
}
