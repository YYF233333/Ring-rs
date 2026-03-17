use super::*;

#[test]
fn test_execute_show_character_without_path_uses_existing_binding() {
    let (mut executor, mut state, script) = test_ctx("");

    state.visible_characters.insert(
        "alice".to_string(),
        ("alice.png".to_string(), Position::Left),
    );

    let node = ScriptNode::ShowCharacter {
        path: None,
        alias: "alice".to_string(),
        position: Position::Right,
        transition: None,
    };

    let result = executor.execute(&node, &mut state, &script).unwrap();

    assert!(matches!(
        &result.commands[0],
        Command::ShowCharacter { path, alias, position, .. }
        if path == "alice.png" && alias == "alice" && *position == Position::Right
    ));
    assert_eq!(
        state.visible_characters.get("alice"),
        Some(&("alice.png".to_string(), Position::Right))
    );
}

#[test]
fn test_execute_show_character_without_path_errors_when_not_bound() {
    let (mut executor, mut state, script) = test_ctx("");

    let node = ScriptNode::ShowCharacter {
        path: None,
        alias: "alice".to_string(),
        position: Position::Center,
        transition: None,
    };

    let result = executor.execute(&node, &mut state, &script);
    assert!(matches!(result, Err(RuntimeError::InvalidState { .. })));
}

#[test]
fn test_execute_change_scene_resolves_mask_path() {
    let (mut executor, mut state, script) = test_ctx("scripts");

    let transition = Transition::with_named_args(
        "rule",
        vec![
            (
                Some("mask".to_string()),
                TransitionArg::String("masks/rule.png".to_string()),
            ),
            (Some("duration".to_string()), TransitionArg::Number(0.5)),
        ],
    );

    let node = ScriptNode::ChangeScene {
        path: "../backgrounds/bg.jpg".to_string(),
        transition: Some(transition),
    };

    let result = executor.execute(&node, &mut state, &script).unwrap();

    // 背景路径会被 resolve
    assert_eq!(
        state.current_background,
        Some("scripts/../backgrounds/bg.jpg".to_string())
    );

    // mask 参数会被 resolve
    fn extract_mask(cmd: &Command) -> Option<String> {
        match cmd {
            Command::ChangeScene { transition, .. } => {
                let transition = transition.as_ref()?;
                transition.args.iter().find_map(|(k, v)| {
                    if k.as_deref() != Some("mask") {
                        return None;
                    }
                    match v {
                        TransitionArg::String(s) => Some(s.clone()),
                        _ => None,
                    }
                })
            }
            _ => None,
        }
    }

    assert_eq!(
        extract_mask(&result.commands[0]),
        Some("scripts/masks/rule.png".to_string())
    );
    // 覆盖非 ChangeScene 分支
    assert_eq!(
        extract_mask(&Command::ShowText {
            speaker: None,
            content: "x".to_string(),
            inline_effects: vec![],
            no_wait: false,
        }),
        None
    );
}

#[test]
fn test_execute_conditional_with_multiple_body_nodes() {
    use crate::script::Expr;
    use crate::script::ast::ConditionalBranch;
    use crate::state::VarValue;

    let (mut executor, mut state) = test_env();
    state.set_var("flag", VarValue::Bool(true));

    let script = Script::new("test", vec![], "");

    let node = ScriptNode::Conditional {
        branches: vec![ConditionalBranch {
            condition: Some(Expr::var("flag")),
            body: vec![
                ScriptNode::ChangeBG {
                    path: "bg.png".to_string(),
                    transition: None,
                },
                ScriptNode::Dialogue {
                    speaker: Some("角色".to_string()),
                    content: "对话".to_string(),
                    inline_effects: vec![],
                    no_wait: false,
                },
            ],
        }],
    };

    let result = executor.execute(&node, &mut state, &script).unwrap();

    // ChangeBG 不等待，Dialogue 等待
    // 应该返回两个命令和 WaitForClick
    assert_eq!(result.commands.len(), 2);
    assert!(matches!(
        &result.commands[0],
        Command::ShowBackground { .. }
    ));
    assert!(matches!(&result.commands[1], Command::ShowText { .. }));
    assert!(matches!(result.waiting, Some(WaitingReason::WaitForClick)));
}

#[test]
fn test_execute_conditional_with_goto() {
    use crate::script::Expr;
    use crate::script::ast::ConditionalBranch;
    use crate::state::VarValue;

    let (mut executor, mut state) = test_env();
    state.set_var("flag", VarValue::Bool(true));

    let script = Script::new(
        "test",
        vec![
            ScriptNode::Label {
                name: "start".to_string(),
            },
            ScriptNode::Dialogue {
                speaker: None,
                content: "开始".to_string(),
                inline_effects: vec![],
                no_wait: false,
            },
            ScriptNode::Label {
                name: "end".to_string(),
            },
        ],
        "",
    );

    let node = ScriptNode::Conditional {
        branches: vec![ConditionalBranch {
            condition: Some(Expr::var("flag")),
            body: vec![
                ScriptNode::Goto {
                    target_label: "end".to_string(),
                },
                // 这个不应该被执行
                ScriptNode::Dialogue {
                    speaker: None,
                    content: "不会执行".to_string(),
                    inline_effects: vec![],
                    no_wait: false,
                },
            ],
        }],
    };

    let result = executor.execute(&node, &mut state, &script).unwrap();

    // 应该跳转到 "end" 标签
    assert!(result.commands.is_empty());
    assert!(result.waiting.is_none());
    assert_eq!(result.jump_to, Some(2)); // "end" 标签的索引
}

#[test]
fn test_execute_call_script_control_flow() {
    let (mut executor, mut state, script) = test_ctx("scripts/remake");
    let node = ScriptNode::CallScript {
        path: "ring/summer/prologue.md".to_string(),
        display_label: Some("start".to_string()),
    };

    let result = executor.execute(&node, &mut state, &script).unwrap();
    assert!(result.commands.is_empty());
    assert!(result.waiting.is_none());
    assert!(result.jump_to.is_none());
    assert!(matches!(
        result.script_control,
        Some(ScriptControlFlow::Call { target_path, display_label: Some(label) })
            if target_path == "ring/summer/prologue.md" && label == "start"
    ));
}

#[test]
fn test_execute_return_from_script_control_flow() {
    let (mut executor, mut state, script) = test_ctx("");
    let node = ScriptNode::ReturnFromScript;
    let result = executor.execute(&node, &mut state, &script).unwrap();
    assert!(matches!(
        result.script_control,
        Some(ScriptControlFlow::Return)
    ));
}

#[test]
fn test_execute_wait() {
    let (mut executor, mut state, script) = test_ctx("");
    let node = ScriptNode::Wait { duration: 1.5 };
    let result = executor.execute(&node, &mut state, &script).unwrap();
    assert!(result.commands.is_empty());
    assert_eq!(
        result.waiting,
        Some(WaitingReason::WaitForTime(
            std::time::Duration::from_secs_f64(1.5)
        ))
    );
}

#[test]
fn test_execute_set_var_persistent_prefix_routes_to_persistent_variables() {
    let (mut executor, mut state, script) = test_ctx("");
    let node = ScriptNode::SetVar {
        name: "persistent.complete_summer".to_string(),
        value: crate::script::Expr::Literal(crate::state::VarValue::Bool(true)),
    };
    let result = executor.execute(&node, &mut state, &script).unwrap();
    assert!(result.commands.is_empty());
    // 写入了 persistent_variables，bare key 不含前缀
    assert_eq!(
        state.get_persistent_var("complete_summer"),
        Some(&crate::state::VarValue::Bool(true))
    );
    // 会话变量未被污染
    assert_eq!(state.get_var("complete_summer"), None);
    assert_eq!(state.get_var("persistent.complete_summer"), None);
}

#[test]
fn test_execute_scene_effect_with_duration_waits() {
    let (mut executor, mut state, script) = test_ctx("");
    let node = ScriptNode::SceneEffect {
        effect: Transition::with_named_args(
            "blurIn",
            vec![(Some("duration".to_string()), TransitionArg::Number(0.5))],
        ),
    };
    let result = executor.execute(&node, &mut state, &script).unwrap();
    assert_eq!(result.commands.len(), 1);
    assert!(matches!(
        &result.commands[0],
        Command::SceneEffect { name, .. } if name == "blurIn"
    ));
    assert!(matches!(
        result.waiting,
        Some(WaitingReason::WaitForSignal(ref id)) if id.as_str() == "scene_effect"
    ));
}

#[test]
fn test_execute_title_card_waits_for_signal() {
    let (mut executor, mut state, script) = test_ctx("");
    let node = ScriptNode::TitleCard {
        text: "Chapter 1".to_string(),
        duration: 1.5,
    };
    let result = executor.execute(&node, &mut state, &script).unwrap();
    assert_eq!(result.commands.len(), 1);
    assert!(matches!(
        &result.commands[0],
        Command::TitleCard { text, duration }
            if text == "Chapter 1" && (*duration - 1.5).abs() < f64::EPSILON
    ));
    assert!(matches!(
        result.waiting,
        Some(WaitingReason::WaitForSignal(ref id)) if id.as_str() == "title_card"
    ));
}

#[test]
fn test_execute_cutscene_waits_for_signal() {
    let (mut executor, mut state, script) = test_ctx("");
    let node = ScriptNode::Cutscene {
        path: "audio/ending.webm".to_string(),
    };
    let result = executor.execute(&node, &mut state, &script).unwrap();
    assert_eq!(result.commands.len(), 1);
    assert!(matches!(
        &result.commands[0],
        Command::Cutscene { path } if path == "audio/ending.webm"
    ));
    assert!(matches!(
        result.waiting,
        Some(WaitingReason::WaitForSignal(ref id)) if id.as_str() == "cutscene"
    ));
}

#[test]
fn test_execute_cutscene_resolves_path() {
    let mut executor = super::Executor::new();
    let script = crate::script::Script::new(
        "test",
        vec![ScriptNode::Cutscene {
            path: "video/ending.webm".to_string(),
        }],
        "scripts/winter",
    );
    let mut state = RuntimeState::new("test");
    let node = script.get_node(0).unwrap();
    let result = executor.execute(node, &mut state, &script).unwrap();
    assert!(matches!(
        &result.commands[0],
        Command::Cutscene { path } if path == "scripts/winter/video/ending.webm"
    ));
}
