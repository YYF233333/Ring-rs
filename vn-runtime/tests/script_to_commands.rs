//! 端到端集成测试：脚本文本 → Parser → VNRuntime → Command 输出验证。

use vn_runtime::{Command, Parser, RuntimeInput, VNRuntime, WaitingReason};

fn parse_and_run(script_text: &str) -> (Vec<Command>, WaitingReason) {
    let mut parser = Parser::new();
    let script = parser.parse("test", script_text).unwrap();
    let mut runtime = VNRuntime::new(script);
    runtime.tick(None).unwrap()
}

#[test]
fn dialogue_produces_show_text() {
    let (commands, waiting) = parse_and_run(r#"羽艾："为什么会变成这样呢？""#);

    assert!(
        commands.iter().any(|c| matches!(c, Command::ShowText {
            speaker: Some(s), ..
        } if s == "羽艾")),
        "expected ShowText with speaker 羽艾, got: {commands:?}"
    );
    assert_eq!(waiting, WaitingReason::WaitForClick);
}

#[test]
fn narration_produces_show_text_no_speaker() {
    let (commands, _) = parse_and_run(r#"："这是旁白。""#);

    assert!(
        commands
            .iter()
            .any(|c| matches!(c, Command::ShowText { speaker: None, .. })),
        "expected ShowText with no speaker, got: {commands:?}"
    );
}

#[test]
fn change_bg_produces_show_background() {
    let (commands, waiting) = parse_and_run(r#"changeBG <img src="bg/sky.jpg" />"#);

    assert!(
        commands
            .iter()
            .any(|c| matches!(c, Command::ShowBackground { path, .. } if path == "bg/sky.jpg")),
        "expected ShowBackground, got: {commands:?}"
    );
    assert_eq!(waiting, WaitingReason::None);
}

#[test]
fn show_character_produces_command() {
    let (commands, _) =
        parse_and_run(r#"show <img src="char/a.png" /> as alice at center with dissolve"#);

    assert!(
        commands
            .iter()
            .any(|c| matches!(c, Command::ShowCharacter { alias, .. } if alias == "alice")),
        "expected ShowCharacter with alias alice, got: {commands:?}"
    );
}

#[test]
fn multi_step_tick_sequence() {
    let input = "\
changeBG <img src=\"bg/school.jpg\" />

羽艾：\"你好。\"

路汐：\"早上好。\"";

    let mut parser = Parser::new();
    let script = parser.parse("test", input).unwrap();
    let mut runtime = VNRuntime::new(script);

    // tick 1: changeBG（非等待指令）+ 对话
    let (cmds1, w1) = runtime.tick(None).unwrap();
    assert!(
        cmds1
            .iter()
            .any(|c| matches!(c, Command::ShowBackground { .. })),
        "tick 1 should contain ShowBackground"
    );
    assert!(
        cmds1.iter().any(|c| matches!(c, Command::ShowText { .. })),
        "tick 1 should contain ShowText"
    );
    assert_eq!(w1, WaitingReason::WaitForClick);

    // tick 2: 用户点击 → 下一行对话
    let (cmds2, w2) = runtime.tick(Some(RuntimeInput::Click)).unwrap();
    assert!(
        cmds2.iter().any(|c| matches!(c, Command::ShowText {
            speaker: Some(s), ..
        } if s == "路汐")),
        "tick 2 should show 路汐's dialogue"
    );
    assert_eq!(w2, WaitingReason::WaitForClick);
}

#[test]
fn script_finished_after_last_node() {
    let (_, waiting) = parse_and_run(r#"changeBG <img src="bg.jpg" />"#);
    assert_eq!(waiting, WaitingReason::None);
}

#[test]
fn goto_jumps_to_label() {
    let input = "\
goto **target**

：\"这行不应执行\"

**target**
：\"跳转成功\"";

    let mut parser = Parser::new();
    let script = parser.parse("test", input).unwrap();
    let mut runtime = VNRuntime::new(script);
    let (cmds, _) = runtime.tick(None).unwrap();

    assert!(
        cmds.iter().any(|c| matches!(c, Command::ShowText {
            content, ..
        } if content == "跳转成功")),
        "should jump to target label, got: {cmds:?}"
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

    let mut parser = Parser::new();
    let script = parser.parse("test", input).unwrap();
    let mut runtime = VNRuntime::new(script);
    let (cmds, _) = runtime.tick(None).unwrap();

    assert!(
        cmds.iter().any(|c| matches!(c, Command::ShowText {
            content, ..
        } if content == "条件成立")),
        "should take true branch, got: {cmds:?}"
    );
}

#[test]
fn wait_instruction_produces_wait_reason() {
    let (_, waiting) = parse_and_run("wait 1.5");

    assert!(
        matches!(waiting, WaitingReason::WaitForTime(d) if (d.as_secs_f64() - 1.5).abs() < 0.01),
        "expected WaitForTime(1.5s), got: {waiting:?}"
    );
}
