//! 端到端集成测试：脚本文本 → Parser → VNRuntime → Command 输出验证。

mod common;

use common::ScriptTestHarness;
use vn_runtime::Command;

#[test]
fn dialogue_produces_show_text() {
    let mut h = ScriptTestHarness::new(r#"羽艾："为什么会变成这样呢？""#);
    let result = h.tick();

    assert!(
        result.has_text_from("羽艾", "为什么会变成这样呢？"),
        "expected ShowText with speaker 羽艾, got: {:?}",
        result.commands
    );
    result.assert_waiting_click();
}

#[test]
fn narration_produces_show_text_no_speaker() {
    let mut h = ScriptTestHarness::new(r#"："这是旁白。""#);
    let result = h.tick();

    assert!(
        result.has_narration("这是旁白。"),
        "expected narration, got: {:?}",
        result.commands
    );
}

#[test]
fn change_bg_produces_show_background() {
    let mut h = ScriptTestHarness::new(r#"changeBG <img src="bg/sky.jpg" />"#);
    let result = h.tick();

    assert!(
        result.has_background("bg/sky.jpg"),
        "expected ShowBackground, got: {:?}",
        result.commands
    );
    result.assert_not_waiting();
}

#[test]
fn show_character_produces_command() {
    let mut h =
        ScriptTestHarness::new(r#"show <img src="char/a.png" /> as alice at center with dissolve"#);
    let result = h.tick();

    assert!(
        result.has_character("alice"),
        "expected ShowCharacter with alias alice, got: {:?}",
        result.commands
    );
}

#[test]
fn multi_step_tick_sequence() {
    let input = "\
changeBG <img src=\"bg/school.jpg\" />

羽艾：\"你好。\"

路汐：\"早上好。\"";

    let mut h = ScriptTestHarness::new(input);

    let r1 = h.tick();
    assert!(
        r1.has_background("bg/school.jpg"),
        "tick 1 should contain ShowBackground"
    );
    assert!(
        r1.commands
            .iter()
            .any(|c| matches!(c, Command::ShowText { .. })),
        "tick 1 should contain ShowText"
    );
    r1.assert_waiting_click();

    let r2 = h.click();
    assert!(
        r2.has_text_from("路汐", "早上好。"),
        "tick 2 should show 路汐's dialogue"
    );
    r2.assert_waiting_click();
}

#[test]
fn script_finished_after_last_node() {
    let mut h = ScriptTestHarness::new(r#"changeBG <img src="bg.jpg" />"#);
    h.tick().assert_not_waiting();
}

#[test]
fn goto_jumps_to_label() {
    let input = "\
goto **target**

：\"这行不应执行\"

**target**
：\"跳转成功\"";

    let mut h = ScriptTestHarness::new(input);
    let result = h.tick();

    assert!(
        result.has_text("跳转成功"),
        "should jump to target label, got: {:?}",
        result.commands
    );
}

#[test]
fn conditional_branch_evaluates() {
    let input = "\
set $flag = true
if $flag == true
  ：\"条件成立\"
else
  ：\"条件不成立\"
endif";

    let mut h = ScriptTestHarness::new(input);
    let result = h.tick();

    assert!(
        result.has_text("条件成立"),
        "should take true branch, got: {:?}",
        result.commands
    );
}

#[test]
fn wait_instruction_produces_wait_reason() {
    let mut h = ScriptTestHarness::new("wait 1.5");
    h.tick().assert_waiting_time(1.5);
}
