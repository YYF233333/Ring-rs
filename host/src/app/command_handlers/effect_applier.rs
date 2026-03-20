//! # EffectApplier — 统一动画/过渡效果应用
//!
//! 消费 `CommandOutput.effect_requests`，将每个 `EffectRequest`
//! 分发到对应的动画子系统（AnimationSystem / TransitionManager / SceneTransitionManager）。
//!
//! 这是 command_handlers 层中处理所有动画/过渡效果的**唯一入口**。
//!
//! ## Capability 与回退策略
//!
//! 效果请求按 `capability_id` 由 [`crate::extensions::ExtensionRegistry`] 分发；若返回 MissingCapability 或 Failed，
//! 会按 target + effect.kind 尝试**回退**到更基础的 capability（见 `build_fallback_request`）。
//! 完整 capability 列表与回退表见 **`docs/engine/reference/extension-effects-capability.md`**。

use crate::extensions::{
    CAP_EFFECT_DISSOLVE, CAP_EFFECT_FADE, CAP_EFFECT_MOVE, CAP_EFFECT_RULE_MASK,
};
use crate::manifest::Manifest;
use crate::renderer::animation::EasingFunction;
use crate::renderer::effects::{EffectKind, EffectRequest, EffectTarget};
use tracing::{debug, info, warn};

use super::super::CoreSystems;

/// 应用所有效果请求
///
/// 遍历 `command_executor.last_output.effect_requests`，
/// 对每个请求执行 capability 路由；缺失或失败时回退到更基础的 capability。
pub fn apply_effect_requests(
    registry: &crate::extensions::ExtensionRegistry,
    core: &mut CoreSystems,
    manifest: &Manifest,
) {
    let requests = std::mem::take(&mut core.command_executor.last_output.effect_requests);
    if !requests.is_empty() {
        debug!(count = requests.len(), "开始应用效果请求");
    }

    let mut ctx = crate::extensions::EngineContext::new(
        core as &mut dyn crate::extensions::EngineServices,
        manifest,
    );
    for request in &requests {
        match registry.dispatch(request, &mut ctx) {
            crate::extensions::CapabilityDispatchResult::Handled { extension_name } => {
                info!(
                    capability_id = %request.capability_id,
                    extension = %extension_name,
                    "扩展 capability 处理成功"
                );
            }
            crate::extensions::CapabilityDispatchResult::MissingCapability { capability_id } => {
                warn!(
                    capability_id = %capability_id,
                    "未找到 capability 处理器，尝试 capability 级回退"
                );
                dispatch_fallback(registry, &mut ctx, request, "missing");
            }
            crate::extensions::CapabilityDispatchResult::Failed {
                capability_id,
                extension_name,
                error,
            } => {
                warn!(
                    capability_id = %capability_id,
                    extension = %extension_name,
                    error = %error,
                    "capability 执行失败，尝试 capability 级回退"
                );
                dispatch_fallback(registry, &mut ctx, request, "failed");
            }
        }
    }

    for diagnostic in ctx.take_diagnostics() {
        match diagnostic.level {
            crate::extensions::DiagnosticLevel::Info => info!(
                capability_id = %diagnostic.capability_id,
                extension = %diagnostic.extension_name,
                message = %diagnostic.message,
                "扩展诊断"
            ),
            crate::extensions::DiagnosticLevel::Warn => warn!(
                capability_id = %diagnostic.capability_id,
                extension = %diagnostic.extension_name,
                message = %diagnostic.message,
                "扩展诊断"
            ),
            crate::extensions::DiagnosticLevel::Error => warn!(
                capability_id = %diagnostic.capability_id,
                extension = %diagnostic.extension_name,
                message = %diagnostic.message,
                "扩展错误诊断"
            ),
        }
    }
}

fn dispatch_fallback(
    registry: &crate::extensions::ExtensionRegistry,
    ctx: &mut crate::extensions::EngineContext<'_>,
    request: &EffectRequest,
    reason: &str,
) {
    let Some(fallback_request) = build_fallback_request(request) else {
        warn!(
            capability_id = %request.capability_id,
            reason = %reason,
            "无可用 capability 回退策略，保底跳过该效果请求"
        );
        return;
    };

    if fallback_request.capability_id == request.capability_id
        && fallback_request.effect.kind == request.effect.kind
    {
        warn!(
            capability_id = %request.capability_id,
            reason = %reason,
            "回退请求与原请求一致，避免重复分发"
        );
        return;
    }

    match registry.dispatch(&fallback_request, ctx) {
        crate::extensions::CapabilityDispatchResult::Handled { extension_name } => info!(
            original_capability = %request.capability_id,
            fallback_capability = %fallback_request.capability_id,
            extension = %extension_name,
            reason = %reason,
            "capability 回退执行成功"
        ),
        crate::extensions::CapabilityDispatchResult::MissingCapability { capability_id } => warn!(
            original_capability = %request.capability_id,
            fallback_capability = %capability_id,
            reason = %reason,
            "回退 capability 仍缺失，放弃该效果请求"
        ),
        crate::extensions::CapabilityDispatchResult::Failed {
            capability_id,
            extension_name,
            error,
        } => warn!(
            original_capability = %request.capability_id,
            fallback_capability = %capability_id,
            extension = %extension_name,
            error = %error,
            reason = %reason,
            "回退 capability 执行失败，放弃该效果请求"
        ),
    }
}

/// 根据 target + effect.kind 构造回退用 EffectRequest。
///
/// 回退表与文档一致，修改时请同步更新 `docs/engine/reference/extension-effects-capability.md`。
fn build_fallback_request(request: &EffectRequest) -> Option<EffectRequest> {
    match (&request.target, &request.effect.kind) {
        (EffectTarget::CharacterShow { .. }, _) => {
            Some(rewrite_capability(request, CAP_EFFECT_DISSOLVE, None))
        }
        (EffectTarget::CharacterHide { .. }, _) => {
            Some(rewrite_capability(request, CAP_EFFECT_DISSOLVE, None))
        }
        (EffectTarget::BackgroundTransition { .. }, _) => {
            Some(rewrite_capability(request, CAP_EFFECT_DISSOLVE, None))
        }
        (EffectTarget::CharacterMove { .. }, _) => {
            Some(rewrite_capability(request, CAP_EFFECT_MOVE, None))
        }
        (EffectTarget::SceneTransition { .. }, EffectKind::Rule { .. }) => {
            Some(rewrite_capability(request, CAP_EFFECT_RULE_MASK, None))
        }
        (EffectTarget::SceneTransition { .. }, EffectKind::Fade | EffectKind::FadeWhite) => {
            Some(rewrite_capability(request, CAP_EFFECT_FADE, None))
        }
        (EffectTarget::SceneTransition { .. }, _) => Some(rewrite_capability(
            request,
            CAP_EFFECT_FADE,
            Some(EffectKind::Fade),
        )),
        (EffectTarget::SceneEffect { .. }, _) | (EffectTarget::TitleCard { .. }, _) => None,
    }
}

fn rewrite_capability(
    request: &EffectRequest,
    capability_id: &str,
    force_kind: Option<EffectKind>,
) -> EffectRequest {
    let effect = if let Some(kind) = force_kind {
        crate::renderer::effects::ResolvedEffect {
            kind,
            duration: request.effect.duration,
            easing: EasingFunction::EaseInOut,
        }
    } else {
        request.effect.clone()
    };

    let mut fallback = EffectRequest::new(request.target.clone(), effect);
    fallback.capability_id = crate::extensions::CapabilityId::new(capability_id);
    fallback
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::renderer::effects::ResolvedEffect;
    use vn_runtime::command::Position;

    fn req(target: EffectTarget, kind: EffectKind) -> EffectRequest {
        EffectRequest::new(
            target,
            ResolvedEffect {
                kind,
                duration: Some(0.5),
                easing: EasingFunction::EaseInOut,
            },
        )
    }

    #[test]
    fn fallback_scene_unknown_to_fade() {
        let request = req(
            EffectTarget::SceneTransition {
                pending_background: "bg.png".to_string(),
            },
            EffectKind::None,
        );
        let fallback = build_fallback_request(&request).expect("fallback should exist");
        assert_eq!(fallback.capability_id, CAP_EFFECT_FADE);
        assert_eq!(fallback.effect.kind, EffectKind::Fade);
    }

    #[test]
    fn fallback_move_keeps_move_capability() {
        let request = req(
            EffectTarget::CharacterMove {
                alias: "a".to_string(),
                old_position: Position::Left,
                new_position: Position::Right,
            },
            EffectKind::Move,
        );
        let fallback = build_fallback_request(&request).expect("fallback should exist");
        assert_eq!(fallback.capability_id, CAP_EFFECT_MOVE);
        assert_eq!(fallback.effect.kind, EffectKind::Move);
    }

    #[test]
    fn fallback_character_show_to_dissolve() {
        let request = req(
            EffectTarget::CharacterShow {
                alias: "hero".to_string(),
            },
            EffectKind::Dissolve,
        );
        let fallback = build_fallback_request(&request).expect("fallback should exist");
        assert_eq!(fallback.capability_id, CAP_EFFECT_DISSOLVE);
    }

    #[test]
    fn fallback_character_hide_to_dissolve() {
        let request = req(
            EffectTarget::CharacterHide {
                alias: "hero".to_string(),
            },
            EffectKind::Dissolve,
        );
        let fallback = build_fallback_request(&request).expect("fallback should exist");
        assert_eq!(fallback.capability_id, CAP_EFFECT_DISSOLVE);
    }

    #[test]
    fn fallback_background_transition_to_dissolve() {
        let request = req(
            EffectTarget::BackgroundTransition {
                old_background: Some("bg_old.png".to_string()),
            },
            EffectKind::Dissolve,
        );
        let fallback = build_fallback_request(&request).expect("fallback should exist");
        assert_eq!(fallback.capability_id, CAP_EFFECT_DISSOLVE);
    }

    #[test]
    fn fallback_scene_rule_to_rule_mask() {
        let request = req(
            EffectTarget::SceneTransition {
                pending_background: "bg.png".to_string(),
            },
            EffectKind::Rule {
                mask_path: "masks/wipe.png".to_string(),
                reversed: false,
            },
        );
        let fallback = build_fallback_request(&request).expect("fallback should exist");
        assert_eq!(fallback.capability_id, CAP_EFFECT_RULE_MASK);
    }

    #[test]
    fn fallback_scene_fade_keeps_fade_capability() {
        let request = req(
            EffectTarget::SceneTransition {
                pending_background: "bg.png".to_string(),
            },
            EffectKind::Fade,
        );
        let fallback = build_fallback_request(&request).expect("fallback should exist");
        assert_eq!(fallback.capability_id, CAP_EFFECT_FADE);
        assert_eq!(fallback.effect.kind, EffectKind::Fade);
    }

    #[test]
    fn fallback_scene_fade_white_keeps_fade_capability() {
        let request = req(
            EffectTarget::SceneTransition {
                pending_background: "bg.png".to_string(),
            },
            EffectKind::FadeWhite,
        );
        let fallback = build_fallback_request(&request).expect("fallback should exist");
        assert_eq!(fallback.capability_id, CAP_EFFECT_FADE);
        assert_eq!(fallback.effect.kind, EffectKind::FadeWhite);
    }

    #[test]
    fn fallback_scene_effect_returns_none() {
        let request = req(
            EffectTarget::SceneEffect {
                effect_name: "shakeSmall".to_string(),
            },
            EffectKind::None,
        );
        assert!(build_fallback_request(&request).is_none());
    }

    #[test]
    fn fallback_title_card_returns_none() {
        let request = req(
            EffectTarget::TitleCard {
                text: "Chapter 1".to_string(),
            },
            EffectKind::None,
        );
        assert!(build_fallback_request(&request).is_none());
    }

    #[test]
    fn rewrite_capability_preserves_duration() {
        let request = req(
            EffectTarget::SceneTransition {
                pending_background: "bg.png".to_string(),
            },
            EffectKind::None,
        );
        let fallback = build_fallback_request(&request).expect("fallback should exist");
        assert_eq!(fallback.effect.duration, Some(0.5));
    }
}
