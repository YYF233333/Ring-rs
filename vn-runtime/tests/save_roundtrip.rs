//! 端到端存档 round-trip 测试：Runtime 状态 → SaveData → JSON → 恢复 → 行为等价。

use vn_runtime::{Command, Parser, RuntimeInput, SaveData, VNRuntime};

fn make_runtime(script_text: &str) -> VNRuntime {
    let mut parser = Parser::new();
    let script = parser.parse("test", script_text).unwrap();
    VNRuntime::new(script)
}

#[test]
fn save_restore_produces_identical_tick_output() {
    let input = "\
changeBG <img src=\"bg/school.jpg\" />

羽艾：\"你好。\"

路汐：\"早上好。\"

：\"一切如常。\"";

    let mut runtime = make_runtime(input);

    // tick 到第二行对话
    let _ = runtime.tick(None).unwrap();
    let _ = runtime.tick(Some(RuntimeInput::Click)).unwrap();

    // 此时 runtime 等待在第三行对话前——保存状态
    let saved_state = runtime.state().clone();
    let saved_history = runtime.history().clone();
    let save_data = SaveData::new(1, saved_state.clone());
    let json = save_data.to_json().unwrap();

    // 从 JSON 恢复
    let loaded = SaveData::from_json(&json).unwrap();

    // 用恢复的状态重建 runtime
    let mut parser = Parser::new();
    let script = parser.parse("test", input).unwrap();
    let mut restored = VNRuntime::restore(script, loaded.runtime_state, saved_history);

    // 原始 runtime 和恢复后的 runtime 从同一位置 tick，应产出相同结果
    let (orig_cmds, orig_wait) = runtime.tick(Some(RuntimeInput::Click)).unwrap();
    let (rest_cmds, rest_wait) = restored.tick(Some(RuntimeInput::Click)).unwrap();

    assert_eq!(orig_wait, rest_wait, "waiting reason should match");
    assert_eq!(
        orig_cmds.len(),
        rest_cmds.len(),
        "command count should match"
    );

    for (o, r) in orig_cmds.iter().zip(rest_cmds.iter()) {
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

    let mut runtime = make_runtime(input);
    let (cmds, _) = runtime.tick(None).unwrap();

    // 验证变量生效
    assert!(
        cmds.iter().any(|c| matches!(c, Command::ShowText {
            content, ..
        } if content == "正确")),
        "pre-save: variable should work"
    );

    // 保存并恢复
    let state = runtime.state().clone();
    let save = SaveData::new(1, state);
    let json = save.to_json().unwrap();
    let loaded = SaveData::from_json(&json).unwrap();

    assert_eq!(
        loaded.runtime_state.variables.get("counter"),
        Some(&vn_runtime::VarValue::Int(42)),
        "variable should survive round-trip"
    );
}

#[test]
fn save_json_contains_position() {
    let input = "\
changeBG <img src=\"bg.jpg\" />
：\"第一行\"";

    let mut runtime = make_runtime(input);
    let _ = runtime.tick(None).unwrap();

    let state = runtime.state().clone();
    let json = SaveData::new(1, state).to_json().unwrap();

    assert!(json.contains("test"), "save should contain script_id");
    assert!(
        json.contains("node_index"),
        "save should contain node_index"
    );
}
