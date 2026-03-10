//! 内建效果扩展（Phase A 首批能力）。

use std::rc::Rc;
use std::sync::Arc;

use crate::renderer::animation::ObjectId;
use crate::renderer::effects::{EffectKind, EffectTarget, defaults};
use crate::renderer::{AnimatableCharacter, position_to_preset_name};

use super::capability::{EffectExtension, ExtensionError};
use super::context::EngineContext;
use super::manifest::ExtensionManifest;
use super::registry::ExtensionRegistry;

pub const CAP_EFFECT_DISSOLVE: &str = "effect.dissolve";
pub const CAP_EFFECT_FADE: &str = "effect.fade";
pub const CAP_EFFECT_RULE_MASK: &str = "effect.rule_mask";
pub const CAP_EFFECT_MOVE: &str = "effect.move";
pub const CAP_EFFECT_SCENE_SHAKE: &str = "effect.scene.shake";
pub const CAP_EFFECT_SCENE_BLUR: &str = "effect.scene.blur";
pub const CAP_EFFECT_SCENE_DIM: &str = "effect.scene.dim";
pub const CAP_EFFECT_SCENE_TITLE_CARD: &str = "effect.scene.title_card";

#[derive(Debug)]
struct BuiltinEffectExtension {
    manifest: ExtensionManifest,
}

impl BuiltinEffectExtension {
    fn new(name: &str, capabilities: Vec<&str>, engine_api_version: &str) -> Self {
        Self {
            manifest: ExtensionManifest {
                name: name.to_string(),
                version: "1.0.0".to_string(),
                engine_api_version: engine_api_version.to_string(),
                capabilities: capabilities.into_iter().map(|v| v.to_string()).collect(),
                dependencies: Vec::new(),
            },
        }
    }
}

impl EffectExtension for BuiltinEffectExtension {
    fn manifest(&self) -> &ExtensionManifest {
        &self.manifest
    }

    fn on_request(
        &self,
        request: &crate::renderer::effects::EffectRequest,
        ctx: &mut EngineContext<'_>,
    ) -> Result<(), ExtensionError> {
        match request.capability_id.as_str() {
            CAP_EFFECT_DISSOLVE => apply_dissolve(request, ctx),
            CAP_EFFECT_FADE => apply_fade_family(request, ctx),
            CAP_EFFECT_RULE_MASK => apply_rule_mask(request, ctx),
            CAP_EFFECT_MOVE => apply_character_move(request, ctx),
            CAP_EFFECT_SCENE_SHAKE => apply_scene_shake(request, ctx),
            CAP_EFFECT_SCENE_BLUR => apply_scene_blur(request, ctx),
            CAP_EFFECT_SCENE_DIM => apply_scene_dim(request, ctx),
            CAP_EFFECT_SCENE_TITLE_CARD => apply_title_card(request, ctx),
            other => Err(ExtensionError::CapabilityNotFound {
                capability_id: other.to_string(),
            }),
        }
    }
}

pub fn build_builtin_registry(
    engine_api_version: &str,
) -> Result<ExtensionRegistry, ExtensionError> {
    let mut registry = ExtensionRegistry::new(engine_api_version);
    let dissolve = Arc::new(BuiltinEffectExtension::new(
        "builtin.effect.dissolve",
        vec![CAP_EFFECT_DISSOLVE],
        engine_api_version,
    ));
    let fade = Arc::new(BuiltinEffectExtension::new(
        "builtin.effect.fade",
        vec![CAP_EFFECT_FADE],
        engine_api_version,
    ));
    let rule = Arc::new(BuiltinEffectExtension::new(
        "builtin.effect.rule_mask",
        vec![CAP_EFFECT_RULE_MASK],
        engine_api_version,
    ));
    let move_extension = Arc::new(BuiltinEffectExtension::new(
        "builtin.effect.move",
        vec![CAP_EFFECT_MOVE],
        engine_api_version,
    ));
    let scene_effects = Arc::new(BuiltinEffectExtension::new(
        "builtin.effect.scene",
        vec![
            CAP_EFFECT_SCENE_SHAKE,
            CAP_EFFECT_SCENE_BLUR,
            CAP_EFFECT_SCENE_DIM,
            CAP_EFFECT_SCENE_TITLE_CARD,
        ],
        engine_api_version,
    ));

    registry.register_extension(dissolve)?;
    registry.register_extension(fade)?;
    registry.register_extension(rule)?;
    registry.register_extension(move_extension)?;
    registry.register_extension(scene_effects)?;
    Ok(registry)
}

fn apply_dissolve(
    request: &crate::renderer::effects::EffectRequest,
    ctx: &mut EngineContext<'_>,
) -> Result<(), ExtensionError> {
    match &request.target {
        EffectTarget::CharacterShow { alias } => {
            let duration = request
                .effect
                .duration_or(defaults::CHARACTER_ALPHA_DURATION);
            let core = ctx.core_mut();
            let Some(character) = core.render_state.get_character_anim(alias).cloned() else {
                return Ok(());
            };
            let object_id = ensure_character_registered(core, alias, &character);
            core.animation_system
                .animate_object::<AnimatableCharacter>(object_id, "alpha", 0.0, 1.0, duration)
                .map_err(|error| ExtensionError::Runtime {
                    capability_id: request.capability_id.clone(),
                    message: format!("角色淡入动画失败: {error}"),
                })?;
            Ok(())
        }
        EffectTarget::CharacterHide { alias } => {
            let duration = request
                .effect
                .duration_or(defaults::CHARACTER_ALPHA_DURATION);
            let core = ctx.core_mut();
            let Some(&object_id) = core.character_object_ids.get(alias) else {
                return Ok(());
            };
            core.animation_system
                .animate_object::<AnimatableCharacter>(object_id, "alpha", 1.0, 0.0, duration)
                .map_err(|error| ExtensionError::Runtime {
                    capability_id: request.capability_id.clone(),
                    message: format!("角色淡出动画失败: {error}"),
                })?;
            Ok(())
        }
        EffectTarget::BackgroundTransition { old_background } => {
            let core = ctx.core_mut();
            core.renderer
                .start_background_transition_resolved(old_background.clone(), &request.effect);
            Ok(())
        }
        target => Err(ExtensionError::UnsupportedTarget {
            capability_id: request.capability_id.clone(),
            target: format!("{target:?}"),
        }),
    }
}

fn apply_fade_family(
    request: &crate::renderer::effects::EffectRequest,
    ctx: &mut EngineContext<'_>,
) -> Result<(), ExtensionError> {
    let EffectTarget::SceneTransition { pending_background } = &request.target else {
        return Err(ExtensionError::UnsupportedTarget {
            capability_id: request.capability_id.clone(),
            target: format!("{:?}", request.target),
        });
    };

    let core = ctx.core_mut();
    match &request.effect.kind {
        EffectKind::Fade => {
            let duration = request.effect.duration_or(defaults::FADE_DURATION);
            core.renderer
                .start_scene_fade(duration, pending_background.to_string());
            Ok(())
        }
        EffectKind::FadeWhite => {
            let duration = request.effect.duration_or(defaults::FADE_WHITE_DURATION);
            core.renderer
                .start_scene_fade_white(duration, pending_background.to_string());
            Ok(())
        }
        other => Err(ExtensionError::Runtime {
            capability_id: request.capability_id.clone(),
            message: format!("effect.fade 不支持效果类型: {other:?}"),
        }),
    }
}

fn apply_rule_mask(
    request: &crate::renderer::effects::EffectRequest,
    ctx: &mut EngineContext<'_>,
) -> Result<(), ExtensionError> {
    let EffectTarget::SceneTransition { pending_background } = &request.target else {
        return Err(ExtensionError::UnsupportedTarget {
            capability_id: request.capability_id.clone(),
            target: format!("{:?}", request.target),
        });
    };
    let EffectKind::Rule {
        mask_path,
        reversed,
    } = &request.effect.kind
    else {
        return Err(ExtensionError::Runtime {
            capability_id: request.capability_id.clone(),
            message: "effect.rule_mask 缺少 rule 参数".to_string(),
        });
    };

    let duration = request.effect.duration_or(defaults::RULE_DURATION);
    let core = ctx.core_mut();
    core.renderer.start_scene_rule(
        duration,
        pending_background.to_string(),
        mask_path.clone(),
        *reversed,
    );
    Ok(())
}

fn ensure_character_registered(
    core: &mut crate::app::CoreSystems,
    alias: &str,
    character: &AnimatableCharacter,
) -> ObjectId {
    if let Some(&id) = core.character_object_ids.get(alias) {
        id
    } else {
        let id = core.animation_system.register(Rc::new(character.clone()));
        core.character_object_ids.insert(alias.to_string(), id);
        id
    }
}

pub fn apply_character_move(
    request: &crate::renderer::effects::EffectRequest,
    ctx: &mut EngineContext<'_>,
) -> Result<(), ExtensionError> {
    let EffectTarget::CharacterMove {
        alias,
        old_position,
        new_position,
    } = &request.target
    else {
        return Err(ExtensionError::UnsupportedTarget {
            capability_id: request.capability_id.clone(),
            target: format!("{:?}", request.target),
        });
    };

    let old_preset_name = position_to_preset_name(*old_position);
    let new_preset_name = position_to_preset_name(*new_position);
    let old_preset = ctx.manifest().get_preset(old_preset_name);
    let new_preset = ctx.manifest().get_preset(new_preset_name);
    let duration = request.effect.duration_or(defaults::MOVE_DURATION);

    let core = ctx.core_mut();
    let screen_w = core.renderer.screen_width();
    let screen_h = core.renderer.screen_height();
    let (offset_x, offset_y, start_scale) =
        compute_move_transition(&old_preset, &new_preset, screen_w, screen_h);
    let Some(character) = core.render_state.get_character_anim(alias).cloned() else {
        return Ok(());
    };
    let object_id = ensure_character_registered(core, alias, &character);
    character.set("position_x", offset_x);
    character.set("position_y", offset_y);
    character.set("scale_x", start_scale);
    character.set("scale_y", start_scale);
    core.animation_system
        .animate_object_with_easing::<AnimatableCharacter>(
            object_id,
            "position_x",
            offset_x,
            0.0,
            duration,
            request.effect.easing,
        )
        .map_err(|error| ExtensionError::Runtime {
            capability_id: request.capability_id.clone(),
            message: format!("角色 X 位移动画失败: {error}"),
        })?;
    core.animation_system
        .animate_object_with_easing::<AnimatableCharacter>(
            object_id,
            "position_y",
            offset_y,
            0.0,
            duration,
            request.effect.easing,
        )
        .map_err(|error| ExtensionError::Runtime {
            capability_id: request.capability_id.clone(),
            message: format!("角色 Y 位移动画失败: {error}"),
        })?;
    core.animation_system
        .animate_object_with_easing::<AnimatableCharacter>(
            object_id,
            "scale_x",
            start_scale,
            1.0,
            duration,
            request.effect.easing,
        )
        .map_err(|error| ExtensionError::Runtime {
            capability_id: request.capability_id.clone(),
            message: format!("角色 X 缩放动画失败: {error}"),
        })?;
    core.animation_system
        .animate_object_with_easing::<AnimatableCharacter>(
            object_id,
            "scale_y",
            start_scale,
            1.0,
            duration,
            request.effect.easing,
        )
        .map_err(|error| ExtensionError::Runtime {
            capability_id: request.capability_id.clone(),
            message: format!("角色 Y 缩放动画失败: {error}"),
        })?;
    Ok(())
}

// ============ 场景效果实现 ============

fn apply_scene_shake(
    request: &crate::renderer::effects::EffectRequest,
    ctx: &mut EngineContext<'_>,
) -> Result<(), ExtensionError> {
    let EffectTarget::SceneEffect { effect_name } = &request.target else {
        return Err(ExtensionError::UnsupportedTarget {
            capability_id: request.capability_id.clone(),
            target: format!("{:?}", request.target),
        });
    };

    let duration = request.effect.duration_or(defaults::SHAKE_DURATION);
    let core = ctx.core_mut();
    let name_lower = effect_name.to_lowercase();

    if name_lower.contains("vertical") {
        core.renderer.start_shake(0.0, 8.0, duration);
    } else if name_lower.contains("bounce") {
        core.renderer.start_shake(0.0, 5.0, duration);
    } else {
        core.renderer.start_shake(6.0, 4.0, duration);
    }

    Ok(())
}

fn apply_scene_blur(
    request: &crate::renderer::effects::EffectRequest,
    ctx: &mut EngineContext<'_>,
) -> Result<(), ExtensionError> {
    let EffectTarget::SceneEffect { effect_name } = &request.target else {
        return Err(ExtensionError::UnsupportedTarget {
            capability_id: request.capability_id.clone(),
            target: format!("{:?}", request.target),
        });
    };

    let duration = request.effect.duration_or(0.5);
    let core = ctx.core_mut();
    let name_lower = effect_name.to_lowercase();

    if name_lower.contains("out") {
        core.render_state.scene_effect.blur_amount = 0.0;
        core.renderer.start_blur_transition(1.0, 0.0, duration);
    } else {
        core.render_state.scene_effect.blur_amount = 1.0;
        core.renderer.start_blur_transition(0.0, 1.0, duration);
    }

    Ok(())
}

fn apply_scene_dim(
    request: &crate::renderer::effects::EffectRequest,
    ctx: &mut EngineContext<'_>,
) -> Result<(), ExtensionError> {
    let EffectTarget::SceneEffect { effect_name } = &request.target else {
        return Err(ExtensionError::UnsupportedTarget {
            capability_id: request.capability_id.clone(),
            target: format!("{:?}", request.target),
        });
    };

    let core = ctx.core_mut();
    let name_lower = effect_name.to_lowercase();

    if name_lower.contains("reset") {
        core.render_state.scene_effect.dim_level = 0.0;
    } else {
        let level = request
            .params
            .get("level")
            .and_then(|v| match v {
                crate::renderer::effects::EffectParamValue::Number(n) => Some(*n),
                _ => None,
            })
            .unwrap_or(1.0);
        let dim = (level / 7.0).clamp(0.0, 1.0);
        core.render_state.scene_effect.dim_level = dim;
    }

    Ok(())
}

fn apply_title_card(
    _request: &crate::renderer::effects::EffectRequest,
    _ctx: &mut EngineContext<'_>,
) -> Result<(), ExtensionError> {
    // TitleCard 的状态已在 execute_title_card 中设置到 render_state.title_card，
    // 渲染和计时由 Renderer::render() 和 update loop 驱动。
    Ok(())
}

fn compute_move_transition(
    old_preset: &crate::manifest::PositionPreset,
    new_preset: &crate::manifest::PositionPreset,
    screen_w: f32,
    screen_h: f32,
) -> (f32, f32, f32) {
    let offset_x = screen_w * (old_preset.x - new_preset.x);
    let offset_y = screen_h * (old_preset.y - new_preset.y);
    // 保持视觉连续：切换到新 preset 后先用比例抵消，再插值回 1.0
    let start_scale = if new_preset.scale.abs() > f32::EPSILON {
        old_preset.scale / new_preset.scale
    } else {
        1.0
    };
    (offset_x, offset_y, start_scale)
}

#[cfg(test)]
mod tests {
    use super::compute_move_transition;
    use crate::manifest::PositionPreset;

    #[test]
    fn compute_move_transition_includes_offset_and_scale_ratio() {
        let old_preset = PositionPreset {
            x: 0.2,
            y: 0.9,
            scale: 0.9,
        };
        let new_preset = PositionPreset {
            x: 0.8,
            y: 0.7,
            scale: 0.6,
        };

        let (offset_x, offset_y, start_scale) =
            compute_move_transition(&old_preset, &new_preset, 1000.0, 500.0);

        assert!((offset_x + 600.0).abs() < 0.01);
        assert!((offset_y - 100.0).abs() < 0.01);
        assert!((start_scale - 1.5).abs() < 0.01);
    }

    #[test]
    fn compute_move_transition_handles_zero_target_scale() {
        let old_preset = PositionPreset {
            x: 0.1,
            y: 0.2,
            scale: 0.9,
        };
        let new_preset = PositionPreset {
            x: 0.1,
            y: 0.2,
            scale: 0.0,
        };

        let (_offset_x, _offset_y, start_scale) =
            compute_move_transition(&old_preset, &new_preset, 1280.0, 720.0);
        assert!((start_scale - 1.0).abs() < 0.01);
    }
}
