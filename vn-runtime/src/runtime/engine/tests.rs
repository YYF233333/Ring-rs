use super::*;
use crate::script::ScriptNode;

fn create_test_script() -> Script {
    Script::new(
        "test",
        vec![
            ScriptNode::Dialogue {
                speaker: Some("Test".to_string()),
                content: "Hello".to_string(),
                inline_effects: vec![],
                no_wait: false,
            },
            ScriptNode::Dialogue {
                speaker: None,
                content: "World".to_string(),
                inline_effects: vec![],
                no_wait: false,
            },
        ],
        "",
    )
}

#[test]
fn test_runtime_creation() {
    let script = create_test_script();
    let runtime = VNRuntime::new(script);

    assert_eq!(runtime.state().position.node_index, 0);
    assert!(!runtime.state().waiting.is_waiting());
}

#[test]
fn test_runtime_tick_dialogue() {
    let script = create_test_script();
    let mut runtime = VNRuntime::new(script);

    // 第一次 tick，执行第一个对话
    let (commands, waiting) = runtime.tick(None).unwrap();

    assert_eq!(commands.len(), 1);
    assert!(matches!(
        &commands[0],
        Command::ShowText { speaker: Some(s), content, .. }
        if s == "Test" && content == "Hello"
    ));
    assert!(matches!(waiting, WaitingReason::WaitForClick));

    // 发送点击输入
    let (commands, waiting) = runtime.tick(Some(RuntimeInput::Click)).unwrap();

    assert_eq!(commands.len(), 1);
    assert!(matches!(
        &commands[0],
        Command::ShowText { speaker: None, content, .. }
        if content == "World"
    ));
    assert!(matches!(waiting, WaitingReason::WaitForClick));
}

#[test]
fn test_tick_accepts_input_when_not_waiting_and_waiting_getter() {
    let script = create_test_script();
    let mut runtime = VNRuntime::new(script);

    // 不在等待状态时传入 Click，应被忽略并正常推进（覆盖 handle_input 的 (None, _) 分支）
    let (commands, waiting) = runtime.tick(Some(RuntimeInput::Click)).unwrap();
    assert_eq!(commands.len(), 1);
    assert!(matches!(waiting, WaitingReason::WaitForClick));

    // waiting() getter 覆盖
    assert!(matches!(runtime.waiting(), WaitingReason::WaitForClick));
}
#[test]
fn test_tick_returns_early_when_still_waiting() {
    let script = create_test_script();
    let mut runtime = VNRuntime::new(script);

    // 第一次 tick 进入等待
    let (_commands, waiting) = runtime.tick(None).unwrap();
    assert!(matches!(waiting, WaitingReason::WaitForClick));

    // 不提供输入时应直接返回空命令，并保持等待状态
    let (commands, waiting2) = runtime.tick(None).unwrap();
    assert!(commands.is_empty());
    assert!(matches!(waiting2, WaitingReason::WaitForClick));
}

#[test]
fn test_runtime_script_end() {
    let script = Script::new(
        "test",
        vec![ScriptNode::ChangeBG {
            path: "bg.png".to_string(),
            transition: None,
        }],
        "",
    );
    let mut runtime = VNRuntime::new(script);

    // 执行完毕
    let (commands, waiting) = runtime.tick(None).unwrap();

    assert_eq!(commands.len(), 1);
    assert!(matches!(waiting, WaitingReason::None));
    assert!(runtime.is_finished());
}

#[test]
fn test_runtime_history_recording() {
    let script = Script::new(
        "test",
        vec![
            ScriptNode::Chapter {
                title: "第一章".to_string(),
                level: 1,
            },
            ScriptNode::Dialogue {
                speaker: Some("角色".to_string()),
                content: "你好".to_string(),
                inline_effects: vec![],
                no_wait: false,
            },
            ScriptNode::Dialogue {
                speaker: None,
                content: "旁白".to_string(),
                inline_effects: vec![],
                no_wait: false,
            },
        ],
        "",
    );
    let mut runtime = VNRuntime::new(script);

    // 执行所有节点
    runtime.tick(None).unwrap();
    runtime.tick(Some(RuntimeInput::Click)).unwrap();
    runtime.tick(Some(RuntimeInput::Click)).unwrap();

    // 验证历史记录
    let history = runtime.history();
    assert_eq!(history.len(), 3); // ChapterMark + 2 Dialogue
    assert_eq!(history.dialogue_count(), 2);
}

#[test]
fn test_runtime_restore_ctor() {
    let script = create_test_script();
    let mut runtime = VNRuntime::new(script.clone());

    // 执行一步，产生等待
    runtime.tick(None).unwrap();

    let saved_state = runtime.state().clone();
    let saved_history = runtime.history().clone();

    let restored = VNRuntime::restore(script, saved_state.clone(), saved_history.clone());
    assert_eq!(
        restored.state().position.node_index,
        saved_state.position.node_index
    );
    assert_eq!(restored.history().len(), saved_history.len());
}

#[test]
fn test_runtime_state_restore() {
    let script = Script::new(
        "test",
        vec![
            ScriptNode::Dialogue {
                speaker: None,
                content: "1".to_string(),
                inline_effects: vec![],
                no_wait: false,
            },
            ScriptNode::Dialogue {
                speaker: None,
                content: "2".to_string(),
                inline_effects: vec![],
                no_wait: false,
            },
            ScriptNode::Dialogue {
                speaker: None,
                content: "3".to_string(),
                inline_effects: vec![],
                no_wait: false,
            },
        ],
        "",
    );
    let mut runtime = VNRuntime::new(script);

    // 推进到第二个对话
    runtime.tick(None).unwrap();
    runtime.tick(Some(RuntimeInput::Click)).unwrap();

    // 保存状态
    let saved_state = runtime.state().clone();
    let saved_history = runtime.history().clone();

    // 继续推进
    runtime.tick(Some(RuntimeInput::Click)).unwrap();
    assert_eq!(runtime.state().position.node_index, 3);

    // 恢复状态
    runtime.restore_state(saved_state);
    runtime.restore_history(saved_history);

    assert_eq!(runtime.state().position.node_index, 2);
    assert_eq!(runtime.history().dialogue_count(), 2);
}

#[test]
fn test_runtime_with_goto() {
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
            ScriptNode::Goto {
                target_label: "end".to_string(),
            },
            ScriptNode::Dialogue {
                speaker: None,
                content: "这句不应该执行".to_string(),
                inline_effects: vec![],
                no_wait: false,
            },
            ScriptNode::Label {
                name: "end".to_string(),
            },
            ScriptNode::Dialogue {
                speaker: None,
                content: "结束".to_string(),
                inline_effects: vec![],
                no_wait: false,
            },
        ],
        "",
    );
    let mut runtime = VNRuntime::new(script);

    // 第一次 tick：执行 Label（无命令）然后 Dialogue
    let (commands1, _) = runtime.tick(None).unwrap();
    assert_eq!(commands1.len(), 1);
    assert!(matches!(&commands1[0], Command::ShowText { content, .. } if content == "开始"));

    // 第二次 tick：执行 Goto 跳过中间对话，直接到 end
    let (commands2, _) = runtime.tick(Some(RuntimeInput::Click)).unwrap();
    assert_eq!(commands2.len(), 1);
    assert!(matches!(&commands2[0], Command::ShowText { content, .. } if content == "结束"));

    // 验证跳过了"这句不应该执行"
    assert_eq!(runtime.history().dialogue_count(), 2);
}

#[test]
fn test_runtime_with_choice() {
    use crate::script::ChoiceOption;

    let script = Script::new(
        "test",
        vec![
            ScriptNode::Choice {
                style: None,
                options: vec![
                    ChoiceOption {
                        text: "选项A".to_string(),
                        target_label: "a".to_string(),
                    },
                    ChoiceOption {
                        text: "选项B".to_string(),
                        target_label: "b".to_string(),
                    },
                ],
            },
            ScriptNode::Label {
                name: "a".to_string(),
            },
            ScriptNode::Dialogue {
                speaker: None,
                content: "选了A".to_string(),
                inline_effects: vec![],
                no_wait: false,
            },
            ScriptNode::Label {
                name: "b".to_string(),
            },
            ScriptNode::Dialogue {
                speaker: None,
                content: "选了B".to_string(),
                inline_effects: vec![],
                no_wait: false,
            },
        ],
        "",
    );
    let mut runtime = VNRuntime::new(script);

    // 执行选择
    let (commands, waiting) = runtime.tick(None).unwrap();
    assert_eq!(commands.len(), 1);
    assert!(matches!(
        waiting,
        WaitingReason::WaitForChoice { choice_count: 2 }
    ));

    // 选择第二个选项（索引1）-> 跳转到 "b"
    let (commands2, _) = runtime
        .tick(Some(RuntimeInput::ChoiceSelected { index: 1 }))
        .unwrap();

    // 应该跳转到 "b" 标签，执行 "选了B" 对话
    assert_eq!(commands2.len(), 1);
    assert!(matches!(&commands2[0], Command::ShowText { content, .. } if content == "选了B"));
}

#[test]
fn test_invalid_choice_index_error() {
    use crate::script::ChoiceOption;

    let script = Script::new(
        "test",
        vec![ScriptNode::Choice {
            style: None,
            options: vec![
                ChoiceOption {
                    text: "A".to_string(),
                    target_label: "a".to_string(),
                },
                ChoiceOption {
                    text: "B".to_string(),
                    target_label: "b".to_string(),
                },
            ],
        }],
        "",
    );
    let mut runtime = VNRuntime::new(script);

    // 进入 WaitForChoice
    let (_commands, waiting) = runtime.tick(None).unwrap();
    assert!(matches!(
        waiting,
        WaitingReason::WaitForChoice { choice_count: 2 }
    ));

    // index 越界
    let err = runtime
        .tick(Some(RuntimeInput::ChoiceSelected { index: 2 }))
        .unwrap_err();
    assert!(matches!(
        err,
        RuntimeError::InvalidChoiceIndex { index: 2, max: 2 }
    ));
}

#[test]
fn test_choice_selected_label_not_found_error() {
    use crate::script::ChoiceOption;

    let script = Script::new(
        "test",
        vec![
            ScriptNode::Choice {
                style: None,
                options: vec![ChoiceOption {
                    text: "A".to_string(),
                    target_label: "missing".to_string(),
                }],
            },
            // 故意不提供 label "missing"
            ScriptNode::Dialogue {
                speaker: None,
                content: "不会执行".to_string(),
                inline_effects: vec![],
                no_wait: false,
            },
        ],
        "",
    );
    let mut runtime = VNRuntime::new(script);

    // 进入 WaitForChoice
    runtime.tick(None).unwrap();

    // 选择后找不到 label
    let err = runtime
        .tick(Some(RuntimeInput::ChoiceSelected { index: 0 }))
        .unwrap_err();
    assert!(matches!(err, RuntimeError::LabelNotFound { label } if label == "missing"));
}

#[test]
fn test_wait_for_signal_clears_only_on_expected_id() {
    let script = create_test_script();
    let mut runtime = VNRuntime::new(script);

    runtime
        .state_mut()
        .wait(WaitingReason::WaitForSignal("ok".to_string()));

    // 错误信号：不解除等待
    let (commands, waiting) = runtime
        .tick(Some(RuntimeInput::Signal {
            id: "nope".to_string(),
        }))
        .unwrap();
    assert!(commands.is_empty());
    assert!(matches!(waiting, WaitingReason::WaitForSignal(id) if id == "ok"));

    // 正确信号：解除等待并继续执行脚本
    let (commands2, waiting2) = runtime
        .tick(Some(RuntimeInput::Signal {
            id: "ok".to_string(),
        }))
        .unwrap();
    assert_eq!(commands2.len(), 1);
    assert!(matches!(waiting2, WaitingReason::WaitForClick));
}

#[test]
fn test_wait_for_time_click_interrupts() {
    use std::time::Duration;

    let script = create_test_script();
    let mut runtime = VNRuntime::new(script);

    runtime
        .state_mut()
        .wait(WaitingReason::WaitForTime(Duration::from_millis(500)));

    // Click 可以打断 WaitForTime
    let (_commands, waiting) = runtime.tick(Some(RuntimeInput::Click)).unwrap();
    assert!(!waiting.is_waiting() || matches!(waiting, WaitingReason::WaitForClick));
}

#[test]
fn test_wait_for_time_ignores_non_click_input() {
    use std::time::Duration;

    let script = create_test_script();
    let mut runtime = VNRuntime::new(script);

    runtime
        .state_mut()
        .wait(WaitingReason::WaitForTime(Duration::from_millis(500)));

    // Signal 不解除 WaitForTime
    let (commands, waiting) = runtime
        .tick(Some(RuntimeInput::Signal {
            id: "test".to_string(),
        }))
        .unwrap();
    assert!(commands.is_empty());
    assert!(matches!(waiting, WaitingReason::WaitForTime(_)));
}

#[test]
fn test_state_mismatch_error() {
    let script = create_test_script();
    let mut runtime = VNRuntime::new(script);

    // 进入 WaitForClick
    runtime.tick(None).unwrap();

    // 在 WaitForClick 时发送 ChoiceSelected，应报错
    let err = runtime
        .tick(Some(RuntimeInput::ChoiceSelected { index: 0 }))
        .unwrap_err();
    assert!(matches!(err, RuntimeError::StateMismatch { .. }));
}

#[test]
fn test_record_history_for_background_and_bgm() {
    let script = Script::new(
        "test",
        vec![
            ScriptNode::ChangeBG {
                path: "bg.png".to_string(),
                transition: None,
            },
            ScriptNode::PlayAudio {
                path: "bgm.mp3".to_string(),
                is_bgm: true,
            },
            ScriptNode::StopBgm,
        ],
        "",
    );
    let mut runtime = VNRuntime::new(script);

    let (_commands, waiting) = runtime.tick(None).unwrap();
    assert!(matches!(waiting, WaitingReason::None));

    // BackgroundChange + BgmChange(Some) + BgmChange(None)
    assert_eq!(runtime.history().len(), 3);
    assert!(matches!(
        runtime.history().events()[0],
        HistoryEvent::BackgroundChange { .. }
    ));
    assert!(matches!(
        runtime.history().events()[1],
        HistoryEvent::BgmChange { path: Some(_), .. }
    ));
    assert!(matches!(
        runtime.history().events()[2],
        HistoryEvent::BgmChange { path: None, .. }
    ));
}

#[test]
fn test_call_script_and_return_flow() {
    let main_script = Script::new(
        "main",
        vec![
            ScriptNode::CallScript {
                path: "ring/child.md".to_string(),
                display_label: Some("entry".to_string()),
            },
            ScriptNode::Dialogue {
                speaker: None,
                content: "主线继续".to_string(),
                inline_effects: vec![],
                no_wait: false,
            },
        ],
        "scripts/remake",
    );

    let child_script = Script::new(
        "child",
        vec![
            ScriptNode::Label {
                name: "entry".to_string(),
            },
            ScriptNode::Dialogue {
                speaker: Some("子脚本".to_string()),
                content: "子流程".to_string(),
                inline_effects: vec![],
                no_wait: false,
            },
            ScriptNode::ReturnFromScript,
        ],
        "scripts/remake/ring",
    );

    let mut runtime = VNRuntime::new(main_script);
    runtime
        .state_mut()
        .position
        .set_path("scripts/remake/main.md".to_string());
    runtime.register_script("scripts/remake/ring/child.md", child_script);

    // 执行 callScript，立即进入子脚本并执行首句对话
    let (commands1, waiting1) = runtime.tick(None).unwrap();
    assert_eq!(commands1.len(), 1);
    assert!(matches!(
        &commands1[0],
        Command::ShowText { speaker: Some(s), content, .. } if s == "子脚本" && content == "子流程"
    ));
    assert!(matches!(waiting1, WaitingReason::WaitForClick));
    assert_eq!(runtime.state().position.script_id, "child");
    assert_eq!(runtime.state().call_stack.len(), 1);

    // 点击后执行 returnFromScript，回到主脚本并继续执行下一句对话
    let (commands2, waiting2) = runtime.tick(Some(RuntimeInput::Click)).unwrap();
    assert_eq!(commands2.len(), 1);
    assert!(matches!(
        &commands2[0],
        Command::ShowText { speaker: None, content, .. } if content == "主线继续"
    ));
    assert!(matches!(waiting2, WaitingReason::WaitForClick));
    assert_eq!(runtime.state().position.script_id, "main");
    assert!(runtime.state().call_stack.is_empty());
}

#[test]
fn test_call_script_missing_target_returns_error() {
    let script = Script::new(
        "main",
        vec![ScriptNode::CallScript {
            path: "missing.md".to_string(),
            display_label: None,
        }],
        "scripts/remake",
    );
    let mut runtime = VNRuntime::new(script);
    runtime
        .state_mut()
        .position
        .set_path("scripts/remake/main.md".to_string());

    let err = runtime.tick(None).unwrap_err();
    assert!(matches!(err, RuntimeError::ScriptNotLoaded { .. }));
}

#[test]
fn test_call_script_auto_return_on_child_eof() {
    let main_script = Script::new(
        "main",
        vec![
            ScriptNode::CallScript {
                path: "ring/child_no_return.md".to_string(),
                display_label: Some("entry".to_string()),
            },
            ScriptNode::Dialogue {
                speaker: None,
                content: "主线恢复".to_string(),
                inline_effects: vec![],
                no_wait: false,
            },
        ],
        "scripts/remake",
    );

    let child_script = Script::new(
        "child_no_return",
        vec![
            ScriptNode::Label {
                name: "entry".to_string(),
            },
            ScriptNode::Dialogue {
                speaker: Some("子脚本".to_string()),
                content: "子结尾自动返回".to_string(),
                inline_effects: vec![],
                no_wait: false,
            },
        ],
        "scripts/remake/ring",
    );

    let mut runtime = VNRuntime::new(main_script);
    runtime
        .state_mut()
        .position
        .set_path("scripts/remake/main.md".to_string());
    runtime.register_script("scripts/remake/ring/child_no_return.md", child_script);

    let (commands1, waiting1) = runtime.tick(None).unwrap();
    assert_eq!(commands1.len(), 1);
    assert!(matches!(
        &commands1[0],
        Command::ShowText { speaker: Some(s), content, .. } if s == "子脚本" && content == "子结尾自动返回"
    ));
    assert!(matches!(waiting1, WaitingReason::WaitForClick));

    // 点击后子脚本到 EOF，应自动 return 并继续主脚本。
    let (commands2, waiting2) = runtime.tick(Some(RuntimeInput::Click)).unwrap();
    assert_eq!(commands2.len(), 1);
    assert!(matches!(
        &commands2[0],
        Command::ShowText { speaker: None, content, .. } if content == "主线恢复"
    ));
    assert!(matches!(waiting2, WaitingReason::WaitForClick));
    assert_eq!(runtime.state().position.script_id, "main");
}

#[test]
fn test_call_script_label_is_display_only() {
    let main_script = Script::new(
        "main",
        vec![ScriptNode::CallScript {
            path: "ring/child_label_display_only.md".to_string(),
            display_label: Some("entry".to_string()),
        }],
        "scripts/remake",
    );

    let child_script = Script::new(
        "child_label_display_only",
        vec![
            ScriptNode::Dialogue {
                speaker: Some("子脚本".to_string()),
                content: "从文件开头执行".to_string(),
                inline_effects: vec![],
                no_wait: false,
            },
            ScriptNode::Label {
                name: "entry".to_string(),
            },
            ScriptNode::Dialogue {
                speaker: Some("子脚本".to_string()),
                content: "旧语义会先到这里".to_string(),
                inline_effects: vec![],
                no_wait: false,
            },
        ],
        "scripts/remake/ring",
    );

    let mut runtime = VNRuntime::new(main_script);
    runtime
        .state_mut()
        .position
        .set_path("scripts/remake/main.md".to_string());
    runtime.register_script(
        "scripts/remake/ring/child_label_display_only.md",
        child_script,
    );

    let (commands, waiting) = runtime.tick(None).unwrap();
    assert_eq!(commands.len(), 1);
    assert!(matches!(
        &commands[0],
        Command::ShowText { speaker: Some(s), content, .. } if s == "子脚本" && content == "从文件开头执行"
    ));
    assert!(matches!(waiting, WaitingReason::WaitForClick));
    assert_eq!(
        runtime.state().position.script_id,
        "child_label_display_only"
    );
}

#[test]
fn test_runtime_is_finished() {
    let script = Script::new(
        "test",
        vec![ScriptNode::Dialogue {
            speaker: None,
            content: "only line".to_string(),
            inline_effects: vec![],
            no_wait: false,
        }],
        "",
    );
    let mut runtime = VNRuntime::new(script);
    assert!(!runtime.is_finished());

    let (_cmds, _waiting) = runtime.tick(None).unwrap();
    assert!(!runtime.is_finished());

    runtime.tick(Some(RuntimeInput::Click)).unwrap();
    assert!(runtime.is_finished());
}

#[test]
fn test_runtime_restore_history() {
    let script = create_test_script();
    let mut runtime = VNRuntime::new(script);

    let mut history = History::new();
    history.push(HistoryEvent::dialogue(
        Some("A".to_string()),
        "hi".to_string(),
    ));

    runtime.restore_history(history);
    assert_eq!(runtime.history().events().len(), 1);
}

#[test]
fn test_record_history_for_extend_text() {
    let script = Script::new(
        "test",
        vec![
            ScriptNode::Dialogue {
                speaker: Some("A".to_string()),
                content: "First".to_string(),
                inline_effects: vec![],
                no_wait: false,
            },
            ScriptNode::Extend {
                content: " continued".to_string(),
                inline_effects: vec![],
                no_wait: false,
            },
        ],
        "",
    );
    let mut runtime = VNRuntime::new(script);

    let (cmds, _) = runtime.tick(None).unwrap();
    assert_eq!(cmds.len(), 1);

    runtime.tick(Some(RuntimeInput::Click)).unwrap();

    let recent = runtime.history().recent_dialogues(5);
    assert!(!recent.is_empty());
    if let HistoryEvent::Dialogue { content, .. } = recent[0] {
        assert!(content.contains("First"));
        assert!(content.contains("continued"));
    } else {
        panic!("Expected Dialogue event");
    }
}

#[test]
fn test_record_history_for_chapter_mark() {
    let script = Script::new(
        "test",
        vec![ScriptNode::Chapter {
            title: "Chapter 1".to_string(),
            level: 1,
        }],
        "",
    );
    let mut runtime = VNRuntime::new(script);
    runtime.tick(None).unwrap();

    assert!(
        runtime
            .history()
            .events()
            .iter()
            .any(|e| matches!(e, HistoryEvent::ChapterMark { .. }))
    );
}

#[test]
fn test_record_history_for_stop_bgm() {
    let script = Script::new("test", vec![ScriptNode::StopBgm], "");
    let mut runtime = VNRuntime::new(script);
    runtime.tick(None).unwrap();

    assert!(
        runtime
            .history()
            .events()
            .iter()
            .any(|e| matches!(e, HistoryEvent::BgmChange { path: None, .. }))
    );
}

#[test]
fn test_runtime_with_extend() {
    let script = Script::new(
        "test",
        vec![
            ScriptNode::Dialogue {
                speaker: None,
                content: "Hello".to_string(),
                inline_effects: vec![],
                no_wait: false,
            },
            ScriptNode::Extend {
                content: " world".to_string(),
                inline_effects: vec![],
                no_wait: false,
            },
        ],
        "",
    );
    let mut runtime = VNRuntime::new(script);

    let (cmds, waiting) = runtime.tick(None).unwrap();
    assert_eq!(cmds.len(), 1);
    assert!(matches!(
        &cmds[0],
        Command::ShowText { content, .. } if content == "Hello"
    ));
    assert!(matches!(waiting, WaitingReason::WaitForClick));

    let (cmds2, waiting2) = runtime.tick(Some(RuntimeInput::Click)).unwrap();
    assert_eq!(cmds2.len(), 1);
    assert!(matches!(
        &cmds2[0],
        Command::ExtendText { content, .. } if content == " world"
    ));
    assert!(matches!(waiting2, WaitingReason::WaitForClick));
}
