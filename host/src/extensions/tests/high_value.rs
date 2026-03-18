use super::*;
use std::sync::Arc;

use crate::manifest::Manifest;

// 兼容性 / 能力校验 / 分发契约 / 错误传播

#[test]
fn reject_incompatible_engine_api_major() {
    let mut registry = ExtensionRegistry::new("1.0.0");
    let ext = DummyExtension {
        manifest: ExtensionManifest {
            name: "dummy.v2".to_string(),
            version: "0.1.0".to_string(),
            engine_api_version: "2.0.0".to_string(),
            capabilities: vec!["effect.dummy".to_string()],
            dependencies: Vec::new(),
        },
    };

    let result = registry.register_extension(Arc::new(ext));
    assert!(matches!(
        result,
        Err(ExtensionError::IncompatibleApiVersion { .. })
    ));
}

#[test]
fn reject_capability_conflict() {
    let mut registry = ExtensionRegistry::new("1.0.0");
    let ext_a = DummyExtension {
        manifest: ExtensionManifest {
            name: "dummy.a".to_string(),
            version: "0.1.0".to_string(),
            engine_api_version: "1.0.0".to_string(),
            capabilities: vec!["effect.same".to_string()],
            dependencies: Vec::new(),
        },
    };
    let ext_b = DummyExtension {
        manifest: ExtensionManifest {
            name: "dummy.b".to_string(),
            version: "0.1.0".to_string(),
            engine_api_version: "1.0.0".to_string(),
            capabilities: vec!["effect.same".to_string()],
            dependencies: Vec::new(),
        },
    };

    registry
        .register_extension(Arc::new(ext_a))
        .expect("first registration should succeed");
    let result = registry.register_extension(Arc::new(ext_b));
    assert!(matches!(
        result,
        Err(ExtensionError::CapabilityConflict { .. })
    ));
}

#[test]
fn register_builtin_capabilities() {
    let registry = build_builtin_registry(ENGINE_API_VERSION).expect("builtin registry");
    assert_eq!(
        registry.source_of(CAP_EFFECT_DISSOLVE),
        Some("builtin.effect.dissolve")
    );
    assert_eq!(
        registry.source_of(CAP_EFFECT_FADE),
        Some("builtin.effect.fade")
    );
    assert_eq!(
        registry.source_of(CAP_EFFECT_RULE_MASK),
        Some("builtin.effect.rule_mask")
    );
    assert_eq!(
        registry.source_of(CAP_EFFECT_MOVE),
        Some("builtin.effect.move")
    );
}

#[test]
fn registry_minor_version_difference_is_compatible() {
    let mut registry = ExtensionRegistry::new("1.5.0");
    let ext = DummyExtension {
        manifest: ExtensionManifest {
            name: "compat.ext".to_string(),
            version: "1.0.0".to_string(),
            engine_api_version: "1.0.0".to_string(),
            capabilities: vec!["effect.compat".to_string()],
            dependencies: Vec::new(),
        },
    };
    // major version matches (1 == 1), should succeed even with minor diff
    assert!(registry.register_extension(Arc::new(ext)).is_ok());
}

#[test]
fn registry_dispatch_missing_capability() {
    let registry = ExtensionRegistry::new("1.0.0");
    let manifest = Manifest::with_defaults();
    let mut svc = MockEngineServices::default();
    let mut ctx = make_engine_context(&mut svc, &manifest);

    let request = EffectRequest {
        capability_id: CapabilityId::new("effect.nonexistent"),
        params: Default::default(),
        target: EffectTarget::SceneEffect {
            effect_name: "noop".to_string(),
        },
        effect: dummy_resolved_effect(),
    };
    let result = registry.dispatch(&request, &mut ctx);
    assert!(matches!(
        result,
        CapabilityDispatchResult::MissingCapability { .. }
    ));
}

#[test]
fn registry_dispatch_success() {
    let mut registry = ExtensionRegistry::new("1.0.0");
    let ext = DummyExtension {
        manifest: make_manifest(),
    };
    registry.register_extension(Arc::new(ext)).unwrap();

    let manifest = Manifest::with_defaults();
    let mut svc = MockEngineServices::default();
    let mut ctx = make_engine_context(&mut svc, &manifest);

    let request = EffectRequest {
        capability_id: CapabilityId::new("effect.test"),
        params: Default::default(),
        target: EffectTarget::SceneEffect {
            effect_name: "noop".to_string(),
        },
        effect: dummy_resolved_effect(),
    };
    let result = registry.dispatch(&request, &mut ctx);
    assert!(matches!(
        result,
        CapabilityDispatchResult::Handled { extension_name } if extension_name == "test.ext"
    ));
}

#[test]
fn registry_dispatch_failure_propagates() {
    let mut registry = ExtensionRegistry::new("1.0.0");
    let ext = FailingExtension {
        manifest: make_manifest(),
    };
    registry.register_extension(Arc::new(ext)).unwrap();

    let manifest = Manifest::with_defaults();
    let mut svc = MockEngineServices::default();
    let mut ctx = make_engine_context(&mut svc, &manifest);

    let request = EffectRequest {
        capability_id: CapabilityId::new("effect.test"),
        params: Default::default(),
        target: EffectTarget::SceneEffect {
            effect_name: "noop".to_string(),
        },
        effect: dummy_resolved_effect(),
    };
    let result = registry.dispatch(&request, &mut ctx);
    assert!(matches!(result, CapabilityDispatchResult::Failed { .. }));
}

// ============ EngineContext 诊断契约与不变量 ============

#[test]
fn engine_context_emit_info_and_take() {
    let manifest = Manifest::with_defaults();
    let mut svc = MockEngineServices::default();
    let mut ctx = make_engine_context(&mut svc, &manifest);

    ctx.emit_info("effect.test", "test.ext", "info message");

    let diags = ctx.take_diagnostics();
    assert_eq!(diags.len(), 1);
    assert_eq!(diags[0].level, DiagnosticLevel::Info);
    assert_eq!(diags[0].capability_id.as_str(), "effect.test");
    assert_eq!(diags[0].extension_name, "test.ext");
    assert_eq!(diags[0].message, "info message");
}

#[test]
fn engine_context_emit_warn() {
    let manifest = Manifest::with_defaults();
    let mut svc = MockEngineServices::default();
    let mut ctx = make_engine_context(&mut svc, &manifest);

    ctx.emit_warn("effect.fade", "builtin.effect.fade", "warn message");

    let diags = ctx.take_diagnostics();
    assert_eq!(diags.len(), 1);
    assert_eq!(diags[0].level, DiagnosticLevel::Warn);
}

#[test]
fn engine_context_emit_error() {
    let manifest = Manifest::with_defaults();
    let mut svc = MockEngineServices::default();
    let mut ctx = make_engine_context(&mut svc, &manifest);

    ctx.emit_error("effect.dissolve", "builtin.effect.dissolve", "error!");

    let diags = ctx.take_diagnostics();
    assert_eq!(diags.len(), 1);
    assert_eq!(diags[0].level, DiagnosticLevel::Error);
}

#[test]
fn engine_context_take_diagnostics_drains() {
    let manifest = Manifest::with_defaults();
    let mut svc = MockEngineServices::default();
    let mut ctx = make_engine_context(&mut svc, &manifest);

    ctx.emit_info("cap", "ext", "msg1");
    ctx.emit_warn("cap", "ext", "msg2");

    let diags = ctx.take_diagnostics();
    assert_eq!(diags.len(), 2);

    // 再次 take 应为空
    let diags2 = ctx.take_diagnostics();
    assert!(diags2.is_empty());
}

#[test]
fn engine_context_multiple_diagnostics_preserve_order() {
    let manifest = Manifest::with_defaults();
    let mut svc = MockEngineServices::default();
    let mut ctx = make_engine_context(&mut svc, &manifest);

    ctx.emit_info("cap.a", "ext.a", "first");
    ctx.emit_warn("cap.b", "ext.b", "second");
    ctx.emit_error("cap.c", "ext.c", "third");

    let diags = ctx.take_diagnostics();
    assert_eq!(diags.len(), 3);
    assert_eq!(diags[0].message, "first");
    assert_eq!(diags[1].message, "second");
    assert_eq!(diags[2].message, "third");
}

// ============ 内建效果契约（通过 registry 分发） ============

#[test]
fn builtin_apply_fade_calls_start_scene_fade() {
    let request = EffectRequest::new(
        EffectTarget::SceneTransition {
            pending_background: "bg_new.png".to_string(),
        },
        ResolvedEffect {
            kind: EffectKind::Fade,
            duration: Some(1.0),
            easing: EasingFunction::Linear,
        },
    );
    let (result, svc) = dispatch_builtin(&request);
    assert!(matches!(result, CapabilityDispatchResult::Handled { .. }));
    assert_eq!(svc.start_scene_fade_calls.len(), 1);
    assert_eq!(svc.start_scene_fade_calls[0].0, 1.0);
    assert_eq!(svc.start_scene_fade_calls[0].1, "bg_new.png");
}

#[test]
fn builtin_apply_fade_white_calls_start_scene_fade_white() {
    let request = EffectRequest::new(
        EffectTarget::SceneTransition {
            pending_background: "bg_white.png".to_string(),
        },
        ResolvedEffect {
            kind: EffectKind::FadeWhite,
            duration: Some(0.8),
            easing: EasingFunction::Linear,
        },
    );
    let (result, svc) = dispatch_builtin(&request);
    assert!(matches!(result, CapabilityDispatchResult::Handled { .. }));
    assert_eq!(svc.start_scene_fade_white_calls.len(), 1);
    assert_eq!(svc.start_scene_fade_white_calls[0].1, "bg_white.png");
}

#[test]
fn builtin_apply_fade_wrong_target_returns_error() {
    let request = EffectRequest {
        capability_id: CapabilityId::new(CAP_EFFECT_FADE),
        params: Default::default(),
        target: EffectTarget::SceneEffect {
            effect_name: "blurIn".to_string(),
        },
        effect: ResolvedEffect {
            kind: EffectKind::Fade,
            duration: Some(0.5),
            easing: EasingFunction::Linear,
        },
    };
    let (result, _) = dispatch_builtin(&request);
    assert!(matches!(result, CapabilityDispatchResult::Failed { .. }));
}

#[test]
fn builtin_apply_rule_mask_calls_start_scene_rule() {
    let request = EffectRequest::new(
        EffectTarget::SceneTransition {
            pending_background: "bg.png".to_string(),
        },
        ResolvedEffect {
            kind: EffectKind::Rule {
                mask_path: "masks/wipe.png".to_string(),
                reversed: true,
            },
            duration: Some(1.5),
            easing: EasingFunction::Linear,
        },
    );
    let (result, svc) = dispatch_builtin(&request);
    assert!(matches!(result, CapabilityDispatchResult::Handled { .. }));
    assert_eq!(svc.start_scene_rule_calls.len(), 1);
    let (dur, bg, mask, reversed) = &svc.start_scene_rule_calls[0];
    assert_eq!(*dur, 1.5);
    assert_eq!(bg, "bg.png");
    assert_eq!(mask, "masks/wipe.png");
    assert!(*reversed);
}

#[test]
fn builtin_apply_dissolve_background_transition() {
    let request = EffectRequest::new(
        EffectTarget::BackgroundTransition {
            old_background: Some("bg_old.png".to_string()),
        },
        ResolvedEffect {
            kind: EffectKind::Dissolve,
            duration: Some(0.5),
            easing: EasingFunction::Linear,
        },
    );
    let (result, svc) = dispatch_builtin(&request);
    assert!(matches!(result, CapabilityDispatchResult::Handled { .. }));
    assert!(svc.start_background_transition_called);
}

#[test]
fn builtin_apply_dissolve_character_show_not_found_returns_ok() {
    let request = EffectRequest {
        capability_id: CapabilityId::new(CAP_EFFECT_DISSOLVE),
        params: Default::default(),
        target: EffectTarget::CharacterShow {
            alias: "alice".to_string(),
        },
        effect: ResolvedEffect {
            kind: EffectKind::Dissolve,
            duration: Some(0.3),
            easing: EasingFunction::Linear,
        },
    };
    let (result, _) = dispatch_builtin(&request);
    assert!(matches!(result, CapabilityDispatchResult::Handled { .. }));
}

#[test]
fn builtin_apply_dissolve_wrong_target_returns_error() {
    let request = EffectRequest {
        capability_id: CapabilityId::new(CAP_EFFECT_DISSOLVE),
        params: Default::default(),
        target: EffectTarget::TitleCard {
            text: "Episode 1".to_string(),
        },
        effect: ResolvedEffect {
            kind: EffectKind::Dissolve,
            duration: Some(0.3),
            easing: EasingFunction::Linear,
        },
    };
    let (result, _) = dispatch_builtin(&request);
    assert!(matches!(result, CapabilityDispatchResult::Failed { .. }));
}

#[test]
fn builtin_apply_scene_shake_default() {
    let request = EffectRequest::new(
        EffectTarget::SceneEffect {
            effect_name: "shakeSmall".to_string(),
        },
        ResolvedEffect {
            kind: EffectKind::SceneEffect {
                name: "shakeSmall".to_string(),
            },
            duration: Some(0.5),
            easing: EasingFunction::Linear,
        },
    );
    let (_, svc) = dispatch_builtin(&request);
    assert_eq!(svc.start_shake_calls.len(), 1);
    let (ax, ay, _) = svc.start_shake_calls[0];
    assert!((ax - 6.0).abs() < 0.01);
    assert!((ay - 4.0).abs() < 0.01);
}

#[test]
fn builtin_apply_scene_shake_vertical() {
    let request = EffectRequest::new(
        EffectTarget::SceneEffect {
            effect_name: "shakeVertical".to_string(),
        },
        ResolvedEffect {
            kind: EffectKind::SceneEffect {
                name: "shakeVertical".to_string(),
            },
            duration: Some(0.5),
            easing: EasingFunction::Linear,
        },
    );
    let (_, svc) = dispatch_builtin(&request);
    let (ax, ay, _) = svc.start_shake_calls[0];
    assert!((ax - 0.0).abs() < 0.01);
    assert!((ay - 8.0).abs() < 0.01);
}

#[test]
fn builtin_apply_scene_shake_bounce() {
    let request = EffectRequest::new(
        EffectTarget::SceneEffect {
            effect_name: "shakeBounce".to_string(),
        },
        ResolvedEffect {
            kind: EffectKind::SceneEffect {
                name: "shakeBounce".to_string(),
            },
            duration: Some(0.3),
            easing: EasingFunction::Linear,
        },
    );
    let (_, svc) = dispatch_builtin(&request);
    let (ax, ay, _) = svc.start_shake_calls[0];
    assert!((ax - 0.0).abs() < 0.01);
    assert!((ay - 5.0).abs() < 0.01);
}

#[test]
fn builtin_apply_scene_shake_wrong_target_returns_error() {
    let request = EffectRequest {
        capability_id: CapabilityId::new(CAP_EFFECT_SCENE_SHAKE),
        params: Default::default(),
        target: EffectTarget::BackgroundTransition {
            old_background: None,
        },
        effect: ResolvedEffect {
            kind: EffectKind::None,
            duration: Some(0.5),
            easing: EasingFunction::Linear,
        },
    };
    let (result, _) = dispatch_builtin(&request);
    assert!(matches!(result, CapabilityDispatchResult::Failed { .. }));
}

#[test]
fn builtin_apply_scene_blur_in() {
    let request = EffectRequest::new(
        EffectTarget::SceneEffect {
            effect_name: "blurIn".to_string(),
        },
        ResolvedEffect {
            kind: EffectKind::SceneEffect {
                name: "blurIn".to_string(),
            },
            duration: Some(0.5),
            easing: EasingFunction::Linear,
        },
    );
    let (_, svc) = dispatch_builtin(&request);
    assert_eq!(svc.start_blur_calls.len(), 1);
    let (from, to, _dur) = svc.start_blur_calls[0];
    assert!((from - 0.0).abs() < 0.01);
    assert!((to - 1.0).abs() < 0.01);
    assert!((svc.scene_blur - 1.0).abs() < 0.01);
}

#[test]
fn builtin_apply_scene_blur_out() {
    let request = EffectRequest::new(
        EffectTarget::SceneEffect {
            effect_name: "blurOut".to_string(),
        },
        ResolvedEffect {
            kind: EffectKind::SceneEffect {
                name: "blurOut".to_string(),
            },
            duration: Some(0.5),
            easing: EasingFunction::Linear,
        },
    );
    let (_, svc) = dispatch_builtin(&request);
    assert_eq!(svc.start_blur_calls.len(), 1);
    let (from, to, _dur) = svc.start_blur_calls[0];
    assert!((from - 1.0).abs() < 0.01);
    assert!((to - 0.0).abs() < 0.01);
    assert!((svc.scene_blur - 0.0).abs() < 0.01);
}

#[test]
fn builtin_apply_scene_dim_with_level() {
    let mut params = std::collections::BTreeMap::new();
    params.insert("level".to_string(), EffectParamValue::Number(7.0));
    let request = EffectRequest {
        capability_id: CapabilityId::new(CAP_EFFECT_SCENE_DIM),
        params,
        target: EffectTarget::SceneEffect {
            effect_name: "dimStep".to_string(),
        },
        effect: ResolvedEffect {
            kind: EffectKind::SceneEffect {
                name: "dimStep".to_string(),
            },
            duration: None,
            easing: EasingFunction::Linear,
        },
    };
    let (_, svc) = dispatch_builtin(&request);
    assert!((svc.scene_dim - 1.0).abs() < 0.01);
}

#[test]
fn builtin_apply_scene_dim_reset() {
    let registry = builtin_registry();
    let manifest = Manifest::with_defaults();
    let mut svc = MockEngineServices {
        scene_dim: 0.8,
        ..Default::default()
    };

    let request = EffectRequest {
        capability_id: CapabilityId::new(CAP_EFFECT_SCENE_DIM),
        params: Default::default(),
        target: EffectTarget::SceneEffect {
            effect_name: "dimReset".to_string(),
        },
        effect: ResolvedEffect {
            kind: EffectKind::SceneEffect {
                name: "dimReset".to_string(),
            },
            duration: None,
            easing: EasingFunction::Linear,
        },
    };

    let mut ctx = make_engine_context(&mut svc, &manifest);
    registry.dispatch(&request, &mut ctx);
    drop(ctx);

    assert!((svc.scene_dim - 0.0).abs() < 0.01);
}

#[test]
fn builtin_apply_title_card_is_noop() {
    let request = EffectRequest {
        capability_id: CapabilityId::new(CAP_EFFECT_SCENE_TITLE_CARD),
        params: Default::default(),
        target: EffectTarget::TitleCard {
            text: "Episode 1".to_string(),
        },
        effect: ResolvedEffect {
            kind: EffectKind::None,
            duration: None,
            easing: EasingFunction::Linear,
        },
    };
    let (result, _) = dispatch_builtin(&request);
    assert!(matches!(result, CapabilityDispatchResult::Handled { .. }));
}

#[test]
fn builtin_apply_move_uses_manifest_presets_for_offset_and_scale() {
    let registry = builtin_registry();
    let mut manifest = Manifest::with_defaults();
    manifest.presets.insert(
        "left".to_string(),
        crate::manifest::PositionPreset {
            x: 0.2,
            y: 0.9,
            scale: 0.9,
        },
    );
    manifest.presets.insert(
        "right".to_string(),
        crate::manifest::PositionPreset {
            x: 0.8,
            y: 0.7,
            scale: 0.6,
        },
    );

    let mut svc = MockEngineServices {
        screen_size: (1000.0, 500.0),
        character_to_return: Some(AnimatableCharacter::new("alice")),
        ..Default::default()
    };

    let request = EffectRequest::new(
        EffectTarget::CharacterMove {
            alias: "alice".to_string(),
            old_position: vn_runtime::command::Position::Left,
            new_position: vn_runtime::command::Position::Right,
        },
        ResolvedEffect {
            kind: EffectKind::Move,
            duration: Some(0.5),
            easing: EasingFunction::Linear,
        },
    );

    let mut ctx = make_engine_context(&mut svc, &manifest);
    let result = registry.dispatch(&request, &mut ctx);
    drop(ctx);

    assert!(matches!(result, CapabilityDispatchResult::Handled { .. }));
    assert_eq!(svc.animate_character_with_easing_calls.len(), 4);
    let (prop_x, from_x, to_x, dur_x) = svc.animate_character_with_easing_calls[0];
    assert_eq!(prop_x, "position_x");
    assert!((from_x + 600.0).abs() < 0.01);
    assert!((to_x - 0.0).abs() < 0.01);
    assert!((dur_x - 0.5).abs() < 0.01);

    let (prop_y, from_y, to_y, dur_y) = svc.animate_character_with_easing_calls[1];
    assert_eq!(prop_y, "position_y");
    assert!((from_y - 100.0).abs() < 0.01);
    assert!((to_y - 0.0).abs() < 0.01);
    assert!((dur_y - 0.5).abs() < 0.01);

    let (prop_sx, from_sx, to_sx, dur_sx) = svc.animate_character_with_easing_calls[2];
    assert_eq!(prop_sx, "scale_x");
    assert!((from_sx - 1.5).abs() < 0.01);
    assert!((to_sx - 1.0).abs() < 0.01);
    assert!((dur_sx - 0.5).abs() < 0.01);

    let (prop_sy, from_sy, to_sy, dur_sy) = svc.animate_character_with_easing_calls[3];
    assert_eq!(prop_sy, "scale_y");
    assert!((from_sy - 1.5).abs() < 0.01);
    assert!((to_sy - 1.0).abs() < 0.01);
    assert!((dur_sy - 0.5).abs() < 0.01);
}

#[test]
fn builtin_scene_shake_source_is_registered() {
    let registry = builtin_registry();
    assert_eq!(
        registry.source_of(CAP_EFFECT_SCENE_SHAKE),
        Some("builtin.effect.scene")
    );
    assert_eq!(
        registry.source_of(CAP_EFFECT_SCENE_BLUR),
        Some("builtin.effect.scene")
    );
    assert_eq!(
        registry.source_of(CAP_EFFECT_SCENE_DIM),
        Some("builtin.effect.scene")
    );
    assert_eq!(
        registry.source_of(CAP_EFFECT_SCENE_TITLE_CARD),
        Some("builtin.effect.scene")
    );
}
