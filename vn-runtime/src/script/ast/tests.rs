use super::*;

#[test]
fn test_script_node_causes_wait() {
    let dialogue = ScriptNode::Dialogue {
        speaker: Some("Test".to_string()),
        content: "Hello".to_string(),
        inline_effects: vec![],
        no_wait: false,
    };
    assert!(dialogue.causes_wait());

    let choice = ScriptNode::Choice {
        style: None,
        options: vec![],
    };
    assert!(choice.causes_wait());

    let bg = ScriptNode::ChangeBG {
        path: "bg.png".to_string(),
        transition: None,
    };
    assert!(!bg.causes_wait());
}

#[test]
fn test_script_node_is_jump_target() {
    let label = ScriptNode::Label {
        name: "start".to_string(),
    };
    assert!(label.is_jump_target());

    let dialogue = ScriptNode::Dialogue {
        speaker: None,
        content: "hi".to_string(),
        inline_effects: vec![],
        no_wait: false,
    };
    assert!(!dialogue.is_jump_target());
}

#[test]
fn test_script_node_is_control_flow() {
    assert!(
        ScriptNode::Label {
            name: "start".to_string()
        }
        .is_control_flow()
    );
    assert!(
        ScriptNode::Goto {
            target_label: "end".to_string()
        }
        .is_control_flow()
    );
    assert!(
        ScriptNode::CallScript {
            path: "chapter1.md".to_string(),
            display_label: Some("entry".to_string()),
        }
        .is_control_flow()
    );
    assert!(ScriptNode::ReturnFromScript.is_control_flow());
    assert!(
        ScriptNode::SetVar {
            name: "flag".to_string(),
            value: crate::script::Expr::bool(true),
        }
        .is_control_flow()
    );
    assert!(
        ScriptNode::Conditional {
            branches: vec![ConditionalBranch {
                condition: Some(crate::script::Expr::bool(true)),
                body: vec![],
            }],
        }
        .is_control_flow()
    );

    let dialogue = ScriptNode::Dialogue {
        speaker: None,
        content: "hi".to_string(),
        inline_effects: vec![],
        no_wait: false,
    };
    assert!(!dialogue.is_control_flow());
}

#[test]
fn test_script_label_index() {
    let nodes = vec![
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
    ];

    let script = Script::new("test", nodes, "");

    assert_eq!(script.find_label("start"), Some(0));
    assert_eq!(script.find_label("end"), Some(2));
    assert_eq!(script.find_label("nonexistent"), None);
}

#[test]
fn test_script_get_node() {
    let nodes = vec![ScriptNode::Dialogue {
        speaker: None,
        content: "Test".to_string(),
        inline_effects: vec![],
        no_wait: false,
    }];

    let script = Script::new("test", nodes, "");

    assert!(script.get_node(0).is_some());
    assert!(script.get_node(1).is_none());
}

#[test]
fn test_script_is_empty() {
    let s = Script::new("empty", vec![], "");
    assert!(s.is_empty());

    let s = Script::new(
        "not_empty",
        vec![ScriptNode::Dialogue {
            speaker: None,
            content: "x".to_string(),
            inline_effects: vec![],
            no_wait: false,
        }],
        "",
    );
    assert!(!s.is_empty());
}

#[test]
fn test_script_has_source_map_matches_actual_map() {
    let no_map = Script::new("plain", vec![], "");
    assert!(!no_map.has_source_map());

    let with_map = Script::with_source_map(
        "mapped",
        vec![ScriptNode::Label {
            name: "start".to_string(),
        }],
        "",
        vec![7],
    );
    assert!(with_map.has_source_map());
    assert_eq!(with_map.get_source_line(0), Some(7));
}

#[test]
fn test_script_resolve_path() {
    let script = Script::new("test", vec![], "scripts");

    // 相对路径
    assert_eq!(
        script.resolve_path("../bgm/music.mp3"),
        "scripts/../bgm/music.mp3"
    );
    assert_eq!(
        script.resolve_path("images/bg.png"),
        "scripts/images/bg.png"
    );

    // 绝对路径不变
    assert_eq!(
        script.resolve_path("/absolute/path.png"),
        "/absolute/path.png"
    );
    assert_eq!(
        script.resolve_path("http://example.com/img.png"),
        "http://example.com/img.png"
    );

    // 空 base_path
    let script_no_base = Script::new("test", vec![], "");
    assert_eq!(
        script_no_base.resolve_path("images/bg.png"),
        "images/bg.png"
    );
}
