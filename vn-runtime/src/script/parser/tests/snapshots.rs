//! Parser snapshot 测试
//!
//! 使用 insta 对 parser 输出做 YAML snapshot，覆盖语法规范中各语法形式。
//! 现有 high_value / low_value 测试验证具体字段；snapshot 验证"整体输出结构不意外变化"。

use super::*;

// ── 对话 ──────────────────────────────────────────────────────────────

#[test]
fn snapshot_dialogue_with_speaker() {
    let script = parse_ok(r#"羽艾："为什么会变成这样呢？""#);
    insta::assert_yaml_snapshot!(script.nodes);
}

#[test]
fn snapshot_narration() {
    let script = parse_ok(r#"："这是旁白文本。""#);
    insta::assert_yaml_snapshot!(script.nodes);
}

#[test]
fn snapshot_dialogue_no_wait() {
    let script = parse_ok(r#"："画面渐渐暗了下来..." -->"#);
    insta::assert_yaml_snapshot!(script.nodes);
}

#[test]
fn snapshot_extend() {
    let input = "路汐：\"这是第一段话。\"\nextend \"然后她继续说道。\"";
    let script = parse_ok(input);
    insta::assert_yaml_snapshot!(script.nodes);
}

// ── 内联标签 ──────────────────────────────────────────────────────────

#[test]
fn snapshot_inline_wait_tags() {
    let script = parse_ok(r#"："倒计时：三、{wait 1s}二、{wait 1s}一！""#);
    insta::assert_yaml_snapshot!(script.nodes);
}

#[test]
fn snapshot_inline_speed_tags() {
    let script = parse_ok(r#"："正常文本。{speed 2x}这段是两倍速。{/speed}""#);
    insta::assert_yaml_snapshot!(script.nodes);
}

// ── 章节与标签 ─────────────────────────────────────────────────────────

#[test]
fn snapshot_chapter_and_label() {
    let input = "# Chapter 1\n\n**intro**\n\n：\"开始。\"";
    let script = parse_ok(input);
    insta::assert_yaml_snapshot!(script.nodes);
}

// ── 背景与场景 ─────────────────────────────────────────────────────────

#[test]
fn snapshot_change_bg_with_dissolve() {
    let input = r#"changeBG <img src="bg/sky.jpg" /> with Dissolve(duration: 1.5)"#;
    let script = parse_ok(input);
    insta::assert_yaml_snapshot!(script.nodes);
}

#[test]
fn snapshot_change_scene_with_fade() {
    let input = r#"changeScene <img src="bg/night.jpg" /> with Fade(duration: 1)"#;
    let script = parse_ok(input);
    insta::assert_yaml_snapshot!(script.nodes);
}

#[test]
fn snapshot_change_scene_with_rule() {
    let input = r#"changeScene <img src="bg/new.jpg" /> with <img src="rule_10.png" /> (duration: 1, reversed: true)"#;
    let script = parse_ok(input);
    insta::assert_yaml_snapshot!(script.nodes);
}

// ── 角色显示/隐藏 ─────────────────────────────────────────────────────

#[test]
fn snapshot_show_character() {
    let input = r#"show <img src="char/royu.png" /> as royu at nearmiddle with dissolve"#;
    let script = parse_ok(input);
    insta::assert_yaml_snapshot!(script.nodes);
}

#[test]
fn snapshot_hide_character() {
    let input = "hide royu with fade";
    let script = parse_ok(input);
    insta::assert_yaml_snapshot!(script.nodes);
}

// ── 音频 ──────────────────────────────────────────────────────────────

#[test]
fn snapshot_audio_bgm_and_sfx() {
    let input =
        "<audio src=\"bgm/Signal.mp3\"></audio> loop\n<audio src=\"sfx/click.wav\"></audio>";
    let script = parse_ok(input);
    insta::assert_yaml_snapshot!(script.nodes);
}

#[test]
fn snapshot_stop_bgm() {
    let script = parse_ok("stopBGM");
    insta::assert_yaml_snapshot!(script.nodes);
}

// ── 控制流 ─────────────────────────────────────────────────────────────

#[test]
fn snapshot_goto() {
    let input = "goto **ending**";
    let script = parse_ok(input);
    insta::assert_yaml_snapshot!(script.nodes);
}

#[test]
fn snapshot_conditional_if_else() {
    let input = "\
if $has_key == true
  ：\"你用钥匙打开了门。\"
  set $door_unlocked = true
else
  ：\"门锁着。\"
endif";
    let script = parse_ok(input);
    insta::assert_yaml_snapshot!(script.nodes);
}

#[test]
fn snapshot_choice() {
    let input = "\
| 横排   |        |
| ------ | ------ |
| 选项1  | label1 |
| 选项2  | label2 |";
    let script = parse_ok(input);
    insta::assert_yaml_snapshot!(script.nodes);
}

#[test]
fn snapshot_set_variable() {
    let input = "set $player_name = \"Alice\"";
    let script = parse_ok(input);
    insta::assert_yaml_snapshot!(script.nodes);
}

// ── 节奏与等待 ─────────────────────────────────────────────────────────

#[test]
fn snapshot_wait() {
    let script = parse_ok("wait 1.5");
    insta::assert_yaml_snapshot!(script.nodes);
}

#[test]
fn snapshot_pause() {
    let script = parse_ok("pause");
    insta::assert_yaml_snapshot!(script.nodes);
}

// ── 复合场景 ──────────────────────────────────────────────────────────

#[test]
fn snapshot_composite_scene() {
    let input = "\
# 序章

**start**

changeBG <img src=\"bg/school.jpg\" /> with dissolve

羽艾：\"早上好。\"
：\"阳光洒进教室。\" -->
extend \"温暖而明亮。\"

wait 0.5

show <img src=\"char/royu.png\" /> as royu at center with dissolve

路汐：\"你来啦。{wait}我等你很久了。\"

| 横排   |        |
| ------ | ------ |
| 打招呼 | greet  |
| 无视   | ignore |

**greet**
：\"你挥了挥手。\"
goto **ending**

**ignore**
：\"你假装没看见。\"

**ending**
hide royu with fade
stopBGM";
    let script = parse_ok(input);
    insta::assert_yaml_snapshot!(script.nodes);
}
