use super::*;

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
fn test_execute_full_restart_emits_command() {
    let (mut executor, mut state, script) = test_ctx("");
    let node = ScriptNode::FullRestart;
    let result = executor.execute(&node, &mut state, &script).unwrap();
    assert_eq!(result.commands, vec![Command::FullRestart]);
    assert!(result.waiting.is_none());
    assert!(result.jump_to.is_none());
    assert!(result.script_control.is_none());
}
