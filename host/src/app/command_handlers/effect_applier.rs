//! # EffectApplier — 统一动画/过渡效果应用
//!
//! 消费 `CommandOutput.effect_requests`，将每个 `EffectRequest`
//! 分发到对应的动画子系统（AnimationSystem / TransitionManager / SceneTransitionManager）。
//!
//! 这是 command_handlers 层中处理所有动画/过渡效果的**唯一入口**。

use crate::renderer::animation::ObjectId;
use crate::renderer::effects::{EffectKind, EffectRequest, EffectTarget, ResolvedEffect, defaults};
use crate::renderer::{AnimatableCharacter, position_to_preset_name};
use macroquad::prelude::screen_width;
use std::rc::Rc;
use tracing::{info, warn};

use super::super::AppState;

/// 应用所有效果请求
///
/// 遍历 `command_executor.last_output.effect_requests`，
/// 对每个请求调用 [`apply_single`] 分发到对应的动画子系统。
pub fn apply_effect_requests(app_state: &mut AppState) {
    let requests = app_state
        .command_executor
        .last_output
        .effect_requests
        .clone();

    for request in &requests {
        apply_single(request, app_state);
    }
}

/// 应用单个效果请求
fn apply_single(request: &EffectRequest, app_state: &mut AppState) {
    match &request.target {
        EffectTarget::CharacterShow { alias } => {
            apply_character_show(alias, &request.effect, app_state);
        }
        EffectTarget::CharacterHide { alias } => {
            apply_character_hide(alias, &request.effect, app_state);
        }
        EffectTarget::CharacterMove {
            alias,
            old_position,
            new_position,
        } => {
            apply_character_move(
                alias,
                *old_position,
                *new_position,
                &request.effect,
                app_state,
            );
        }
        EffectTarget::BackgroundTransition { old_background } => {
            apply_background_transition(old_background.as_deref(), &request.effect, app_state);
        }
        EffectTarget::SceneTransition { pending_background } => {
            apply_scene_transition(pending_background, &request.effect, app_state);
        }
    }
}

// ─── 角色动画 ───────────────────────────────────────────────────────────────────

/// 角色淡入：注册到 AnimationSystem → alpha 0→1
fn apply_character_show(alias: &str, effect: &ResolvedEffect, app_state: &mut AppState) {
    let character = app_state.render_state.get_character_anim(alias).cloned();
    if let Some(character) = character {
        let duration = effect.duration_or(defaults::CHARACTER_ALPHA_DURATION);
        let object_id = ensure_character_registered(app_state, alias, &character);

        if let Err(e) = app_state
            .animation_system
            .animate_object::<AnimatableCharacter>(object_id, "alpha", 0.0, 1.0, duration)
        {
            warn!(error = %e, "启动角色淡入动画失败");
        }
        info!(alias = %alias, duration = %duration, "角色淡入动画");
    }
}

/// 角色淡出：alpha 1→0
fn apply_character_hide(alias: &str, effect: &ResolvedEffect, app_state: &mut AppState) {
    if let Some(&object_id) = app_state.character_object_ids.get(alias) {
        let duration = effect.duration_or(defaults::CHARACTER_ALPHA_DURATION);

        if let Err(e) = app_state
            .animation_system
            .animate_object::<AnimatableCharacter>(object_id, "alpha", 1.0, 0.0, duration)
        {
            warn!(error = %e, "启动角色淡出动画失败");
        }
        info!(alias = %alias, duration = %duration, "角色淡出动画");
    }
}

/// 角色移动：计算位置偏移 → position_x 动画
fn apply_character_move(
    alias: &str,
    old_position: vn_runtime::command::Position,
    new_position: vn_runtime::command::Position,
    effect: &ResolvedEffect,
    app_state: &mut AppState,
) {
    let old_preset_name = position_to_preset_name(old_position);
    let new_preset_name = position_to_preset_name(new_position);
    let old_preset = app_state.manifest.get_preset(old_preset_name);
    let new_preset = app_state.manifest.get_preset(new_preset_name);

    let screen_w = screen_width();
    let offset_x = screen_w * (old_preset.x - new_preset.x);
    let duration = effect.duration_or(defaults::MOVE_DURATION);

    let character = app_state.render_state.get_character_anim(alias).cloned();
    if let Some(character) = character {
        let object_id = ensure_character_registered(app_state, alias, &character);

        // 设置初始偏移（角色视觉上仍在旧位置）
        character.set("position_x", offset_x);

        // 动画：从偏移移动到 0（角色平滑移到新位置）
        if let Err(e) = app_state
            .animation_system
            .animate_object::<AnimatableCharacter>(object_id, "position_x", offset_x, 0.0, duration)
        {
            warn!(error = %e, "启动角色移动动画失败");
        }
        info!(
            alias = %alias,
            from = %old_preset_name,
            to = %new_preset_name,
            duration = %duration,
            "角色移动动画"
        );
    }
}

// ─── 背景过渡 ───────────────────────────────────────────────────────────────────

/// 背景过渡（dissolve）：委托给 TransitionManager
fn apply_background_transition(
    old_background: Option<&str>,
    effect: &ResolvedEffect,
    app_state: &mut AppState,
) {
    app_state
        .renderer
        .start_background_transition_resolved(old_background.map(|s| s.to_string()), effect);
}

// ─── 场景遮罩过渡 ───────────────────────────────────────────────────────────────

/// 场景遮罩过渡：根据 effect.kind 分发到对应 renderer 方法
fn apply_scene_transition(
    pending_background: &str,
    effect: &ResolvedEffect,
    app_state: &mut AppState,
) {
    match &effect.kind {
        EffectKind::Fade => {
            let duration = effect.duration_or(defaults::FADE_DURATION);
            app_state
                .renderer
                .start_scene_fade(duration, pending_background.to_string());
        }
        EffectKind::FadeWhite => {
            let duration = effect.duration_or(defaults::FADE_WHITE_DURATION);
            app_state
                .renderer
                .start_scene_fade_white(duration, pending_background.to_string());
        }
        EffectKind::Rule {
            mask_path,
            reversed,
        } => {
            let duration = effect.duration_or(defaults::RULE_DURATION);
            app_state.renderer.start_scene_rule(
                duration,
                pending_background.to_string(),
                mask_path.clone(),
                *reversed,
            );
        }
        other => {
            warn!(kind = ?other, "SceneTransition 收到非预期效果类型，降级为 Fade");
            let duration = effect.duration_or(defaults::FADE_DURATION);
            app_state
                .renderer
                .start_scene_fade(duration, pending_background.to_string());
        }
    }
}

// ─── 辅助函数 ───────────────────────────────────────────────────────────────────

/// 确保角色已注册到动画系统，返回 ObjectId
fn ensure_character_registered(
    app_state: &mut AppState,
    alias: &str,
    character: &AnimatableCharacter,
) -> ObjectId {
    if let Some(&id) = app_state.character_object_ids.get(alias) {
        id
    } else {
        let id = app_state
            .animation_system
            .register(Rc::new(character.clone()));
        app_state.character_object_ids.insert(alias.to_string(), id);
        id
    }
}
