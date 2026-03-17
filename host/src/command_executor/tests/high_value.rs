use super::*;

#[test]
fn test_execute_show_background_with_transition() {
    use crate::renderer::effects::{EffectKind, EffectTarget};

    let mut ctx = TestCtx::new();
    ctx.render_state.set_background("old_bg.png".to_string());

    let transition = Transition::simple("dissolve");
    let cmd = Command::ShowBackground {
        path: "new_bg.png".to_string(),
        transition: Some(transition),
    };

    let result = ctx.execute(&cmd);
    assert_eq!(result, ExecuteResult::Ok);
    assert_eq!(ctx.executor.last_output.effect_requests.len(), 1);

    let req = &ctx.executor.last_output.effect_requests[0];
    match &req.target {
        EffectTarget::BackgroundTransition { old_background } => {
            assert_eq!(*old_background, Some("old_bg.png".to_string()));
        }
        other => panic!("Expected BackgroundTransition, got {:?}", other),
    }
    assert_eq!(req.capability_id, "effect.dissolve");
    assert_eq!(req.effect.kind, EffectKind::Dissolve);
}

#[test]
fn test_execute_show_character_reposition_with_move_triggers_animation() {
    let mut ctx = TestCtx::new();

    let cmd = Command::ShowCharacter {
        path: "characters/char1.png".to_string(),
        alias: "char1".to_string(),
        position: Position::Center,
        transition: None,
    };
    let result = ctx.execute(&cmd);
    assert_eq!(result, ExecuteResult::Ok);

    let cmd = Command::ShowCharacter {
        path: "characters/char1.png".to_string(),
        alias: "char1".to_string(),
        position: Position::Right,
        transition: Some(Transition::simple("move")),
    };
    let result = ctx.execute(&cmd);
    assert_eq!(result, ExecuteResult::Ok);

    assert_eq!(ctx.executor.last_output.effect_requests.len(), 1);
    let req = &ctx.executor.last_output.effect_requests[0];
    match &req.target {
        crate::renderer::effects::EffectTarget::CharacterMove {
            alias,
            old_position,
            new_position,
        } => {
            assert_eq!(alias, "char1");
            assert_eq!(*old_position, Position::Center);
            assert_eq!(*new_position, Position::Right);
        }
        other => panic!("Expected CharacterMove, got {:?}", other),
    }
    assert_eq!(req.capability_id, "effect.move");
    assert!(req.effect.duration_or(0.0) > 0.0);

    let char_sprite = ctx.render_state.visible_characters.get("char1").unwrap();
    assert_eq!(char_sprite.position, Position::Right);
}

#[test]
fn test_execute_show_character_diff_and_move_uses_diff_then_move() {
    let mut ctx = TestCtx::new();

    let cmd = Command::ShowCharacter {
        path: "characters/char1.png".to_string(),
        alias: "char1".to_string(),
        position: Position::Center,
        transition: None,
    };
    let result = ctx.execute(&cmd);
    assert_eq!(result, ExecuteResult::Ok);

    let cmd = Command::ShowCharacter {
        path: "characters/char2.png".to_string(),
        alias: "char1".to_string(),
        position: Position::Left,
        transition: Some(Transition::simple("dissolve")),
    };
    let result = ctx.execute(&cmd);
    assert_eq!(result, ExecuteResult::Ok);

    assert_eq!(ctx.executor.last_output.effect_requests.len(), 1);
    let req = &ctx.executor.last_output.effect_requests[0];
    assert!(matches!(
        &req.target,
        crate::renderer::effects::EffectTarget::CharacterMove {
            alias,
            old_position: Position::Center,
            new_position: Position::Left
        } if alias == "char1"
    ));
    assert_eq!(req.effect.kind, crate::renderer::effects::EffectKind::Move);
    assert!(req.effect.duration_or(0.0) > 0.0);

    let char_sprite = ctx.render_state.visible_characters.get("char1").unwrap();
    assert_eq!(char_sprite.position, Position::Left);
    assert_eq!(char_sprite.texture_path, "characters/char2.png");
}

#[test]
fn test_change_scene_rule_produces_scene_transition() {
    let mut ctx = TestCtx::new();

    let cmd = Command::ChangeScene {
        path: "new_bg.png".to_string(),
        transition: Some(Transition::with_named_args(
            "rule",
            vec![
                (
                    Some("duration".to_string()),
                    vn_runtime::command::TransitionArg::Number(0.8),
                ),
                (
                    Some("mask".to_string()),
                    vn_runtime::command::TransitionArg::String("masks/wipe.png".to_string()),
                ),
                (
                    Some("reversed".to_string()),
                    vn_runtime::command::TransitionArg::Bool(true),
                ),
            ],
        )),
    };

    let result = ctx.execute(&cmd);
    assert_eq!(result, ExecuteResult::Ok);

    assert_eq!(ctx.executor.last_output.effect_requests.len(), 1);
    let req = &ctx.executor.last_output.effect_requests[0];
    match &req.target {
        crate::renderer::effects::EffectTarget::SceneTransition { pending_background } => {
            assert_eq!(pending_background, "new_bg.png");
        }
        other => panic!("Expected SceneTransition, got {:?}", other),
    }
    match &req.effect.kind {
        crate::renderer::effects::EffectKind::Rule {
            mask_path,
            reversed,
        } => {
            assert!(mask_path.contains("wipe.png") || mask_path.contains("masks"));
            assert!(*reversed);
        }
        other => panic!("Expected Rule effect, got {:?}", other),
    }
    assert_eq!(req.capability_id, "effect.rule_mask");
    assert!((req.effect.duration_or(0.0) - 0.8).abs() < 0.01);
}

#[test]
fn test_explicit_duration_overrides_default_for_all_targets() {
    let mut ctx = TestCtx::new();

    let cmd = Command::ShowCharacter {
        path: "characters/char1.png".to_string(),
        alias: "char1".to_string(),
        position: Position::Center,
        transition: Some(Transition::with_args(
            "dissolve",
            vec![vn_runtime::command::TransitionArg::Number(2.0)],
        )),
    };
    ctx.execute(&cmd);

    assert_eq!(ctx.executor.last_output.effect_requests.len(), 1);
    let dur = ctx.executor.last_output.effect_requests[0]
        .effect
        .duration_or(0.0);
    assert!((dur - 2.0).abs() < 0.01);

    let cmd = Command::ShowBackground {
        path: "bg.png".to_string(),
        transition: Some(Transition::with_args(
            "dissolve",
            vec![vn_runtime::command::TransitionArg::Number(2.0)],
        )),
    };
    ctx.execute(&cmd);

    assert_eq!(ctx.executor.last_output.effect_requests.len(), 1);
    assert_eq!(
        ctx.executor.last_output.effect_requests[0].effect.duration,
        Some(2.0)
    );

    let cmd = Command::ChangeScene {
        path: "bg2.png".to_string(),
        transition: Some(Transition::with_args(
            "fade",
            vec![vn_runtime::command::TransitionArg::Number(2.0)],
        )),
    };
    ctx.execute(&cmd);

    assert_eq!(ctx.executor.last_output.effect_requests.len(), 1);
    let scene_dur = ctx.executor.last_output.effect_requests[0]
        .effect
        .duration_or(0.0);
    assert!((scene_dur - 2.0).abs() < 0.01);
}

#[test]
fn test_scene_effect_dim_produces_effect_request() {
    use crate::renderer::effects::{EffectKind, EffectTarget};
    use vn_runtime::command::TransitionArg;

    let mut ctx = TestCtx::new();
    let cmd = Command::SceneEffect {
        name: "dim".to_string(),
        args: vec![
            (Some("duration".to_string()), TransitionArg::Number(1.5)),
            (Some("level".to_string()), TransitionArg::Number(0.7)),
        ],
    };

    let result = ctx.execute(&cmd);
    assert_eq!(result, ExecuteResult::Ok);
    assert_eq!(ctx.executor.last_output.effect_requests.len(), 1);

    let req = &ctx.executor.last_output.effect_requests[0];
    assert!(matches!(
        &req.target,
        EffectTarget::SceneEffect { effect_name } if effect_name == "dim"
    ));
    assert!(matches!(
        &req.effect.kind,
        EffectKind::SceneEffect { name } if name == "dim"
    ));
    assert_eq!(req.effect.duration, Some(1.5));
}

#[test]
fn test_batch_error_stops_execution() {
    let mut ctx = TestCtx::new();

    let commands = vec![
        Command::ShowBackground {
            path: "bg.png".to_string(),
            transition: None,
        },
        Command::ShowCharacter {
            path: "char.png".to_string(),
            alias: "c".to_string(),
            position: Position::Center,
            transition: None,
        },
        Command::ShowText {
            speaker: None,
            content: "text".to_string(),
            inline_effects: vec![],
            no_wait: false,
        },
    ];

    let result = ctx.execute_batch(&commands);
    assert_eq!(result, ExecuteResult::WaitForClick);
    assert!(ctx.render_state.visible_characters.contains_key("c"));
}
