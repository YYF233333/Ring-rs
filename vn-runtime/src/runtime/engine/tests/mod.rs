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
