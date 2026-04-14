//! 端到端存档 round-trip 测试：Runtime 状态 → SaveData → JSON → 恢复 → 行为等价。

mod common;

use common::ScriptTestHarness;
use vn_runtime::{Command, SaveData, VarValue};

#[test]
fn save_restore_produces_identical_tick_output() {
    let input = "\
changeBG <img src=\"bg/school.jpg\" />

羽艾：\"你好。\"

路汐：\"早上好。\"

：\"一切如常。\"";

    let mut h = ScriptTestHarness::new(input);

    let _ = h.tick();
    let _ = h.click();

    let mut restored = h.save_and_restore();

    let orig = h.click();
    let rest = restored.click();

    assert_eq!(orig.waiting, rest.waiting, "waiting reason should match");
    assert_eq!(
        orig.commands.len(),
        rest.commands.len(),
        "command count should match"
    );

    for (o, r) in orig.commands.iter().zip(rest.commands.iter()) {
        match (o, r) {
            (
                Command::ShowText {
                    speaker: os,
                    content: oc,
                    ..
                },
                Command::ShowText {
                    speaker: rs,
                    content: rc,
                    ..
                },
            ) => {
                assert_eq!(os, rs, "speaker should match");
                assert_eq!(oc, rc, "content should match");
            }
            _ => {
                assert_eq!(
                    std::mem::discriminant(o),
                    std::mem::discriminant(r),
                    "command variant should match"
                );
            }
        }
    }
}

#[test]
fn save_preserves_variable_state() {
    let input = "\
set $counter = 42

if $counter == 42
  ：\"正确\"
else
  ：\"错误\"
endif";

    let mut h = ScriptTestHarness::new(input);
    let result = h.tick();

    assert!(result.has_text("正确"), "pre-save: variable should work");

    let state = h.runtime().state().clone();
    let save = SaveData::new(1, state, 0);
    let json = save.to_json().unwrap();
    let loaded = SaveData::from_json(&json).unwrap();

    assert_eq!(
        loaded.runtime_state.variables.get("counter"),
        Some(&VarValue::Int(42)),
        "variable should survive round-trip"
    );
}

#[test]
fn save_json_contains_position() {
    let input = "\
changeBG <img src=\"bg.jpg\" />
：\"第一行\"";

    let mut h = ScriptTestHarness::new(input);
    let _ = h.tick();

    let state = h.runtime().state().clone();
    let json = SaveData::new(1, state, 0).to_json().unwrap();

    assert!(json.contains("test"), "save should contain script_id");
    assert!(
        json.contains("node_index"),
        "save should contain node_index"
    );
}
