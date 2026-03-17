use super::*;
use crate::script::ScriptNode;

mod high_value;
mod low_value;

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
