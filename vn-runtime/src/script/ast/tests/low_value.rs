//! 低价值测试：getter、简单容器语义（is_empty）。

use super::*;

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
