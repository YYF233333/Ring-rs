use super::*;

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
