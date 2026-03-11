use super::*;
use crate::command::{Position, Transition, TransitionArg};
use crate::script::ChoiceOption;

fn test_ctx(script_root: &str) -> (Executor, RuntimeState, Script) {
    (
        Executor::new(),
        RuntimeState::new("test"),
        Script::new("test", vec![], script_root),
    )
}

fn test_env() -> (Executor, RuntimeState) {
    (Executor::new(), RuntimeState::new("test"))
}

#[test]
fn test_executor_default() {
    let _ = Executor::default();
}

#[test]
fn test_execute_dialogue() {
    let (mut executor, mut state, script) = test_ctx("");

    let node = ScriptNode::Dialogue {
        speaker: Some("Test".to_string()),
        content: "Hello".to_string(),
        inline_effects: vec![],
        no_wait: false,
    };

    let result = executor.execute(&node, &mut state, &script).unwrap();

    assert_eq!(result.commands.len(), 1);
    assert!(matches!(
        &result.commands[0],
        Command::ShowText { speaker: Some(s), content, .. }
        if s == "Test" && content == "Hello"
    ));
    assert!(matches!(result.waiting, Some(WaitingReason::WaitForClick)));
}

#[test]
fn test_execute_show_character() {
    let (mut executor, mut state, script) = test_ctx("");

    let node = ScriptNode::ShowCharacter {
        path: Some("char.png".to_string()),
        alias: "test_char".to_string(),
        position: Position::Center,
        transition: None,
    };

    let result = executor.execute(&node, &mut state, &script).unwrap();

    assert_eq!(result.commands.len(), 1);
    assert!(result.waiting.is_none());

    // 验证状态更新
    assert!(state.visible_characters.contains_key("test_char"));
}

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
fn test_execute_hide_character_updates_state() {
    let (mut executor, mut state, script) = test_ctx("");

    state.visible_characters.insert(
        "alice".to_string(),
        ("alice.png".to_string(), Position::Center),
    );

    let node = ScriptNode::HideCharacter {
        alias: "alice".to_string(),
        transition: None,
    };

    let result = executor.execute(&node, &mut state, &script).unwrap();
    assert!(matches!(
        &result.commands[0],
        Command::HideCharacter { alias, .. } if alias == "alice"
    ));
    assert!(!state.visible_characters.contains_key("alice"));
}

#[test]
fn test_execute_choice() {
    let (mut executor, mut state, script) = test_ctx("");

    let node = ScriptNode::Choice {
        style: Some("横排".to_string()),
        options: vec![
            ChoiceOption {
                text: "选项A".to_string(),
                target_label: "label_a".to_string(),
            },
            ChoiceOption {
                text: "选项B".to_string(),
                target_label: "label_b".to_string(),
            },
        ],
    };

    let result = executor.execute(&node, &mut state, &script).unwrap();

    assert_eq!(result.commands.len(), 1);
    assert!(matches!(
        &result.commands[0],
        Command::PresentChoices { choices, .. } if choices.len() == 2
    ));
    assert!(matches!(
        result.waiting,
        Some(WaitingReason::WaitForChoice { choice_count: 2 })
    ));
}

#[test]
fn test_execute_label_no_command() {
    let (mut executor, mut state, script) = test_ctx("");

    let node = ScriptNode::Label {
        name: "test".to_string(),
    };

    let result = executor.execute(&node, &mut state, &script).unwrap();

    assert!(result.commands.is_empty());
    assert!(result.waiting.is_none());
}

#[test]
fn test_execute_goto() {
    let (mut executor, mut state) = test_env();
    let script = Script::new(
        "test",
        vec![
            ScriptNode::Label {
                name: "start".to_string(),
            },
            ScriptNode::Dialogue {
                speaker: None,
                content: "Hello".to_string(),
                inline_effects: vec![],
                no_wait: false,
            },
            ScriptNode::Label {
                name: "end".to_string(),
            },
        ],
        "",
    );

    let node = ScriptNode::Goto {
        target_label: "end".to_string(),
    };

    let result = executor.execute(&node, &mut state, &script).unwrap();

    assert!(result.commands.is_empty());
    assert!(result.waiting.is_none());
    assert_eq!(result.jump_to, Some(2)); // 跳转到 "end" 标签
}

#[test]
fn test_execute_goto_label_not_found() {
    let (mut executor, mut state, script) = test_ctx("");

    let node = ScriptNode::Goto {
        target_label: "missing".to_string(),
    };

    let result = executor.execute(&node, &mut state, &script);
    assert!(matches!(
        result,
        Err(RuntimeError::LabelNotFound { label }) if label == "missing"
    ));
}

#[test]
fn test_execute_chapter_mark() {
    let (mut executor, mut state, script) = test_ctx("");

    let node = ScriptNode::Chapter {
        title: "第一章".to_string(),
        level: 1,
    };

    let result = executor.execute(&node, &mut state, &script).unwrap();
    assert!(matches!(
        &result.commands[0],
        Command::ChapterMark { title, level } if title == "第一章" && *level == 1
    ));
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
fn test_execute_play_bgm() {
    let (mut executor, mut state, script) = test_ctx("scripts");

    // BGM: 有 loop 标识
    let node = ScriptNode::PlayAudio {
        path: "../bgm/music.mp3".to_string(),
        is_bgm: true,
    };

    let result = executor.execute(&node, &mut state, &script).unwrap();

    assert_eq!(result.commands.len(), 1);
    assert!(matches!(
        &result.commands[0],
        Command::PlayBgm { path, looping: true }
        if path == "scripts/../bgm/music.mp3"
    ));
}

#[test]
fn test_execute_play_sfx() {
    let (mut executor, mut state, script) = test_ctx("scripts");

    // SFX: 无 loop 标识
    let node = ScriptNode::PlayAudio {
        path: "../sfx/click.mp3".to_string(),
        is_bgm: false,
    };

    let result = executor.execute(&node, &mut state, &script).unwrap();

    assert_eq!(result.commands.len(), 1);
    assert!(matches!(
        &result.commands[0],
        Command::PlaySfx { path }
        if path == "scripts/../sfx/click.mp3"
    ));
}

#[test]
fn test_execute_stop_bgm() {
    let (mut executor, mut state, script) = test_ctx("");

    let node = ScriptNode::StopBgm;

    let result = executor.execute(&node, &mut state, &script).unwrap();

    assert_eq!(result.commands.len(), 1);
    assert!(matches!(
        &result.commands[0],
        Command::StopBgm { fade_out: Some(_) }
    ));
}

#[test]
fn test_execute_bgm_duck() {
    let (mut executor, mut state, script) = test_ctx("");

    let result = executor
        .execute(&ScriptNode::BgmDuck, &mut state, &script)
        .unwrap();
    assert_eq!(result.commands.len(), 1);
    assert!(matches!(&result.commands[0], Command::BgmDuck));
}

#[test]
fn test_execute_bgm_unduck() {
    let (mut executor, mut state, script) = test_ctx("");

    let result = executor
        .execute(&ScriptNode::BgmUnduck, &mut state, &script)
        .unwrap();
    assert_eq!(result.commands.len(), 1);
    assert!(matches!(&result.commands[0], Command::BgmUnduck));
}

#[test]
fn test_path_resolution() {
    let (mut executor, mut state, script) = test_ctx("assets/scripts");

    let node = ScriptNode::ChangeBG {
        path: "../backgrounds/bg.jpg".to_string(),
        transition: None,
    };

    let result = executor.execute(&node, &mut state, &script).unwrap();

    assert!(matches!(
        &result.commands[0],
        Command::ShowBackground { path, .. }
        if path == "assets/scripts/../backgrounds/bg.jpg"
    ));
}

//=========================================================================
// SetVar 测试
//=========================================================================

#[test]
fn test_execute_set_var_string() {
    use crate::script::Expr;
    use crate::state::VarValue;

    let (mut executor, mut state, script) = test_ctx("");

    let node = ScriptNode::SetVar {
        name: "name".to_string(),
        value: Expr::string("Alice"),
    };

    let result = executor.execute(&node, &mut state, &script).unwrap();

    // SetVar 不产生命令
    assert!(result.commands.is_empty());
    assert!(result.waiting.is_none());

    // 变量应该被设置
    assert_eq!(
        state.get_var("name"),
        Some(&VarValue::String("Alice".to_string()))
    );
}

#[test]
fn test_execute_set_var_bool() {
    use crate::script::Expr;
    use crate::state::VarValue;

    let (mut executor, mut state, script) = test_ctx("");

    let node = ScriptNode::SetVar {
        name: "flag".to_string(),
        value: Expr::bool(true),
    };

    executor.execute(&node, &mut state, &script).unwrap();
    assert_eq!(state.get_var("flag"), Some(&VarValue::Bool(true)));
}

#[test]
fn test_execute_set_var_from_expression() {
    use crate::script::Expr;
    use crate::state::VarValue;

    let (mut executor, mut state) = test_env();
    state.set_var("a", VarValue::Bool(true));
    state.set_var("b", VarValue::Bool(false));

    let script = Script::new("test", vec![], "");

    // 设置变量为表达式结果
    let node = ScriptNode::SetVar {
        name: "result".to_string(),
        value: Expr::and(Expr::var("a"), Expr::bool(true)),
    };

    executor.execute(&node, &mut state, &script).unwrap();
    assert_eq!(state.get_var("result"), Some(&VarValue::Bool(true)));
}

#[test]
fn test_execute_set_var_undefined_variable_error() {
    use crate::script::Expr;

    let (mut executor, mut state, script) = test_ctx("");

    let node = ScriptNode::SetVar {
        name: "result".to_string(),
        value: Expr::var("undefined"),
    };

    let result = executor.execute(&node, &mut state, &script);
    assert!(matches!(
        result,
        Err(crate::error::RuntimeError::EvalError { .. })
    ));
}

//=========================================================================
// Conditional 测试
//=========================================================================

#[test]
fn test_execute_conditional_true_branch() {
    use crate::script::Expr;
    use crate::script::ast::ConditionalBranch;
    use crate::state::VarValue;

    let (mut executor, mut state) = test_env();
    state.set_var("flag", VarValue::Bool(true));

    let script = Script::new("test", vec![], "");

    let node = ScriptNode::Conditional {
        branches: vec![ConditionalBranch {
            condition: Some(Expr::var("flag")),
            body: vec![ScriptNode::Dialogue {
                speaker: None,
                content: "条件为真".to_string(),
                inline_effects: vec![],
                no_wait: false,
            }],
        }],
    };

    let result = executor.execute(&node, &mut state, &script).unwrap();

    // 应该执行条件为真的分支体
    assert_eq!(result.commands.len(), 1);
    assert!(matches!(
        &result.commands[0],
        Command::ShowText { content, .. } if content == "条件为真"
    ));
    assert!(matches!(result.waiting, Some(WaitingReason::WaitForClick)));
}

#[test]
fn test_execute_conditional_false_branch() {
    use crate::script::Expr;
    use crate::script::ast::ConditionalBranch;
    use crate::state::VarValue;

    let (mut executor, mut state) = test_env();
    state.set_var("flag", VarValue::Bool(false));

    let script = Script::new("test", vec![], "");

    let node = ScriptNode::Conditional {
        branches: vec![ConditionalBranch {
            condition: Some(Expr::var("flag")),
            body: vec![ScriptNode::Dialogue {
                speaker: None,
                content: "条件为真".to_string(),
                inline_effects: vec![],
                no_wait: false,
            }],
        }],
    };

    let result = executor.execute(&node, &mut state, &script).unwrap();

    // 条件为假，没有 else 分支，不执行任何内容
    assert!(result.commands.is_empty());
    assert!(result.waiting.is_none());
}

#[test]
fn test_execute_conditional_else_branch() {
    use crate::script::Expr;
    use crate::script::ast::ConditionalBranch;
    use crate::state::VarValue;

    let (mut executor, mut state) = test_env();
    state.set_var("flag", VarValue::Bool(false));

    let script = Script::new("test", vec![], "");

    let node = ScriptNode::Conditional {
        branches: vec![
            ConditionalBranch {
                condition: Some(Expr::var("flag")),
                body: vec![ScriptNode::Dialogue {
                    speaker: None,
                    content: "条件为真".to_string(),
                    inline_effects: vec![],
                    no_wait: false,
                }],
            },
            ConditionalBranch {
                condition: None, // else 分支
                body: vec![ScriptNode::Dialogue {
                    speaker: None,
                    content: "条件为假".to_string(),
                    inline_effects: vec![],
                    no_wait: false,
                }],
            },
        ],
    };

    let result = executor.execute(&node, &mut state, &script).unwrap();

    // 应该执行 else 分支
    assert_eq!(result.commands.len(), 1);
    assert!(matches!(
        &result.commands[0],
        Command::ShowText { content, .. } if content == "条件为假"
    ));
}

#[test]
fn test_execute_conditional_elseif() {
    use crate::script::Expr;
    use crate::script::ast::ConditionalBranch;
    use crate::state::VarValue;

    let (mut executor, mut state) = test_env();
    state.set_var("role", VarValue::String("user".to_string()));

    let script = Script::new("test", vec![], "");

    let node = ScriptNode::Conditional {
        branches: vec![
            ConditionalBranch {
                condition: Some(Expr::eq(Expr::var("role"), Expr::string("admin"))),
                body: vec![ScriptNode::Dialogue {
                    speaker: None,
                    content: "管理员".to_string(),
                    inline_effects: vec![],
                    no_wait: false,
                }],
            },
            ConditionalBranch {
                condition: Some(Expr::eq(Expr::var("role"), Expr::string("user"))),
                body: vec![ScriptNode::Dialogue {
                    speaker: None,
                    content: "用户".to_string(),
                    inline_effects: vec![],
                    no_wait: false,
                }],
            },
            ConditionalBranch {
                condition: None,
                body: vec![ScriptNode::Dialogue {
                    speaker: None,
                    content: "访客".to_string(),
                    inline_effects: vec![],
                    no_wait: false,
                }],
            },
        ],
    };

    let result = executor.execute(&node, &mut state, &script).unwrap();

    // 应该执行第二个分支（elseif）
    assert_eq!(result.commands.len(), 1);
    assert!(matches!(
        &result.commands[0],
        Command::ShowText { content, .. } if content == "用户"
    ));
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

// =========================================================================
// 阶段 24：TextBox / ClearCharacters 命令测试
// =========================================================================

#[test]
fn test_execute_textbox_hide() {
    let (mut executor, mut state, script) = test_ctx("");
    let node = ScriptNode::TextBoxHide;
    let result = executor.execute(&node, &mut state, &script).unwrap();
    assert_eq!(result.commands.len(), 1);
    assert!(matches!(result.commands[0], Command::TextBoxHide));
    assert!(result.waiting.is_none());
}

#[test]
fn test_execute_textbox_show() {
    let (mut executor, mut state, script) = test_ctx("");
    let node = ScriptNode::TextBoxShow;
    let result = executor.execute(&node, &mut state, &script).unwrap();
    assert_eq!(result.commands.len(), 1);
    assert!(matches!(result.commands[0], Command::TextBoxShow));
    assert!(result.waiting.is_none());
}

#[test]
fn test_execute_textbox_clear() {
    let (mut executor, mut state, script) = test_ctx("");
    let node = ScriptNode::TextBoxClear;
    let result = executor.execute(&node, &mut state, &script).unwrap();
    assert_eq!(result.commands.len(), 1);
    assert!(matches!(result.commands[0], Command::TextBoxClear));
    assert!(result.waiting.is_none());
}

#[test]
fn test_execute_clear_characters() {
    let (mut executor, mut state, script) = test_ctx("");

    // 先添加一些角色
    state.visible_characters.insert(
        "alice".to_string(),
        ("alice.png".to_string(), Position::Left),
    );
    state
        .visible_characters
        .insert("bob".to_string(), ("bob.png".to_string(), Position::Right));
    assert_eq!(state.visible_characters.len(), 2);

    let node = ScriptNode::ClearCharacters;
    let result = executor.execute(&node, &mut state, &script).unwrap();

    assert_eq!(result.commands.len(), 1);
    assert!(matches!(result.commands[0], Command::ClearCharacters));
    assert!(result.waiting.is_none());
    // 状态中的角色应该被清除
    assert!(state.visible_characters.is_empty());
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
fn test_execute_set_var_regular_does_not_write_persistent() {
    let (mut executor, mut state, script) = test_ctx("");
    let node = ScriptNode::SetVar {
        name: "foo".to_string(),
        value: crate::script::Expr::Literal(crate::state::VarValue::Int(99)),
    };
    executor.execute(&node, &mut state, &script).unwrap();
    assert_eq!(state.get_var("foo"), Some(&crate::state::VarValue::Int(99)));
    // 持久变量未被污染
    assert_eq!(state.get_persistent_var("foo"), None);
}

#[test]
fn test_execute_full_restart_emits_command() {
    let (mut executor, mut state, script) = test_ctx("");
    let node = ScriptNode::FullRestart;
    let result = executor.execute(&node, &mut state, &script).unwrap();
    assert_eq!(result.commands, vec![Command::FullRestart]);
    assert!(result.waiting.is_none());
    assert!(result.jump_to.is_none());
    assert!(result.script_control.is_none());
}

#[test]
fn test_execute_pause_waits_for_click() {
    let (mut executor, mut state, script) = test_ctx("");
    let node = ScriptNode::Pause;
    let result = executor.execute(&node, &mut state, &script).unwrap();
    assert!(result.commands.is_empty());
    assert_eq!(result.waiting, Some(WaitingReason::WaitForClick));
}

#[test]
fn test_execute_scene_effect_no_duration() {
    let (mut executor, mut state, script) = test_ctx("");
    let node = ScriptNode::SceneEffect {
        effect: Transition::simple("shakeSmall"),
    };
    let result = executor.execute(&node, &mut state, &script).unwrap();
    assert_eq!(result.commands.len(), 1);
    assert!(matches!(
        &result.commands[0],
        Command::SceneEffect { name, .. } if name == "shakeSmall"
    ));
    assert!(result.waiting.is_none());
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
        Some(WaitingReason::WaitForSignal(ref id)) if id == "scene_effect"
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
        Some(WaitingReason::WaitForSignal(ref id)) if id == "title_card"
    ));
}

#[test]
fn test_execute_extend_text() {
    let (mut executor, mut state, script) = test_ctx("");
    let node = ScriptNode::Extend {
        content: "追加文本".to_string(),
        inline_effects: vec![],
        no_wait: false,
    };
    let result = executor.execute(&node, &mut state, &script).unwrap();
    assert_eq!(result.commands.len(), 1);
    assert!(matches!(
        &result.commands[0],
        Command::ExtendText { content, no_wait: false, .. } if content == "追加文本"
    ));
    assert!(matches!(result.waiting, Some(WaitingReason::WaitForClick)));
}

#[test]
fn test_execute_extend_text_no_wait() {
    let (mut executor, mut state, script) = test_ctx("");
    let node = ScriptNode::Extend {
        content: "自动推进".to_string(),
        inline_effects: vec![],
        no_wait: true,
    };
    let result = executor.execute(&node, &mut state, &script).unwrap();
    assert!(matches!(
        &result.commands[0],
        Command::ExtendText { no_wait: true, .. }
    ));
}

#[test]
fn test_execute_conditional_no_match_returns_empty() {
    let (mut executor, mut state, script) = test_ctx("");
    state.set_var("flag".to_string(), crate::state::VarValue::Bool(false));
    let branches = vec![crate::script::ast::ConditionalBranch {
        condition: Some(crate::script::Expr::var("flag")),
        body: vec![ScriptNode::TextBoxHide],
    }];
    let node = ScriptNode::Conditional { branches };
    let result = executor.execute(&node, &mut state, &script).unwrap();
    assert!(result.commands.is_empty());
    assert!(result.waiting.is_none());
    assert!(result.jump_to.is_none());
}

#[test]
fn test_execute_change_scene_without_transition() {
    let (mut executor, mut state, script) = test_ctx("");
    let node = ScriptNode::ChangeScene {
        path: "bg.png".to_string(),
        transition: None,
    };
    let result = executor.execute(&node, &mut state, &script).unwrap();
    assert_eq!(result.commands.len(), 1);
    assert!(matches!(
        &result.commands[0],
        Command::ChangeScene {
            transition: None,
            ..
        }
    ));
    assert!(result.waiting.is_none());
}

#[test]
fn test_execute_change_scene_non_mask_named_arg_preserved() {
    let (mut executor, mut state, script) = test_ctx("");
    let node = ScriptNode::ChangeScene {
        path: "bg.png".to_string(),
        transition: Some(Transition::with_named_args(
            "Fade",
            vec![(Some("duration".to_string()), TransitionArg::Number(1.5))],
        )),
    };
    let result = executor.execute(&node, &mut state, &script).unwrap();
    if let Command::ChangeScene {
        transition: Some(t),
        ..
    } = &result.commands[0]
    {
        assert_eq!(t.get_duration(), Some(1.5));
    } else {
        panic!("Expected ChangeScene with transition");
    }
}

#[test]
fn test_execute_conditional_with_call_script_in_branch() {
    let (mut executor, mut state, script) = test_ctx("");
    state.set_var("flag".to_string(), crate::state::VarValue::Bool(true));
    let branches = vec![crate::script::ast::ConditionalBranch {
        condition: Some(crate::script::Expr::var("flag")),
        body: vec![ScriptNode::CallScript {
            path: "other.md".to_string(),
            display_label: None,
        }],
    }];
    let node = ScriptNode::Conditional { branches };
    let result = executor.execute(&node, &mut state, &script).unwrap();
    assert!(result.script_control.is_some());
    assert!(matches!(
        result.script_control,
        Some(ScriptControlFlow::Call { .. })
    ));
}

#[test]
fn test_execute_conditional_with_wait_in_branch() {
    let (mut executor, mut state, script) = test_ctx("");
    state.set_var("flag".to_string(), crate::state::VarValue::Bool(true));
    let branches = vec![crate::script::ast::ConditionalBranch {
        condition: Some(crate::script::Expr::var("flag")),
        body: vec![
            ScriptNode::TextBoxHide,
            ScriptNode::Dialogue {
                speaker: None,
                content: "test".to_string(),
                inline_effects: vec![],
                no_wait: false,
            },
            ScriptNode::TextBoxShow,
        ],
    }];
    let node = ScriptNode::Conditional { branches };
    let result = executor.execute(&node, &mut state, &script).unwrap();
    assert_eq!(result.commands.len(), 2);
    assert!(matches!(result.waiting, Some(WaitingReason::WaitForClick)));
}

#[test]
fn test_execute_dialogue_no_wait() {
    let (mut executor, mut state, script) = test_ctx("");
    let node = ScriptNode::Dialogue {
        speaker: Some("A".to_string()),
        content: "auto".to_string(),
        inline_effects: vec![],
        no_wait: true,
    };
    let result = executor.execute(&node, &mut state, &script).unwrap();
    assert!(matches!(
        &result.commands[0],
        Command::ShowText { no_wait: true, .. }
    ));
}
