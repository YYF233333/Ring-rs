//! 过渡效果命令处理

use crate::command_executor::SceneTransitionCommand;
use crate::renderer::effects::ResolvedEffect;

use super::super::AppState;

/// 应用过渡效果（阶段 25：使用 ResolvedEffect）
pub fn apply_transition_effect(app_state: &mut AppState) {
    let transition_info = &app_state.command_executor.last_output.transition_info;

    if transition_info.has_background_transition {
        if let Some(ref effect) = transition_info.effect {
            app_state.renderer.start_background_transition_resolved(
                transition_info.old_background.clone(),
                effect,
            );
        } else {
            // 无显式效果：使用默认短暂 dissolve
            app_state.renderer.start_background_transition_resolved(
                transition_info.old_background.clone(),
                &ResolvedEffect::none(),
            );
        }
    }
}

/// 处理场景切换命令
pub fn handle_scene_transition(app_state: &mut AppState) {
    let scene_cmd = app_state
        .command_executor
        .last_output
        .scene_transition
        .clone();

    if let Some(cmd) = scene_cmd {
        match cmd {
            SceneTransitionCommand::Fade {
                duration,
                pending_background,
            } => {
                app_state
                    .renderer
                    .start_scene_fade(duration, pending_background);
            }
            SceneTransitionCommand::FadeWhite {
                duration,
                pending_background,
            } => {
                app_state
                    .renderer
                    .start_scene_fade_white(duration, pending_background);
            }
            SceneTransitionCommand::Rule {
                duration,
                pending_background,
                mask_path,
                reversed,
            } => {
                app_state.renderer.start_scene_rule(
                    duration,
                    pending_background,
                    mask_path,
                    reversed,
                );
            }
        }
    }
}
