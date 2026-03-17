use super::*;

#[test]
fn test_runtime_creation() {
    let script = create_test_script();
    let runtime = VNRuntime::new(script);

    assert_eq!(runtime.state().position.node_index, 0);
    assert!(!runtime.state().waiting.is_waiting());
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
fn test_runtime_find_label_delegates_to_current_script() {
    let script = Script::new(
        "test",
        vec![
            ScriptNode::Label {
                name: "first".to_string(),
            },
            ScriptNode::Label {
                name: "second".to_string(),
            },
            ScriptNode::Dialogue {
                speaker: None,
                content: "tail".to_string(),
                inline_effects: vec![],
                no_wait: false,
            },
        ],
        "",
    );
    let runtime = VNRuntime::new(script);

    assert_eq!(runtime.find_label("first"), Some(0));
    assert_eq!(runtime.find_label("second"), Some(1));
    assert_eq!(runtime.find_label("missing"), None);
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
