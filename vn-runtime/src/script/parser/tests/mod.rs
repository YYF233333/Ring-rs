//! # Parser 测试

use super::*;
use crate::command::{Position, TransitionArg};
use crate::script::ast::ScriptNode;

mod high_value;
mod low_value;

fn parse_ok(input: &str) -> crate::script::ast::Script {
    let mut parser = Parser::new();
    parser.parse("test", input).unwrap()
}

fn parse_single_node(input: &str) -> ScriptNode {
    let script = parse_ok(input);
    assert_eq!(script.nodes.len(), 1, "Expected exactly one node");
    script.nodes.into_iter().next().unwrap()
}

fn parse_err(input: &str) -> crate::error::ParseError {
    let mut parser = Parser::new();
    parser.parse("test", input).unwrap_err()
}

// -------------------------------------------------------------------------
// 辅助函数测试
// -------------------------------------------------------------------------

#[test]
fn test_parse_dialogue_function() {
    // 中文冒号和引号
    let chinese_dialogue = "羽艾：\u{201C}你好\u{201D}";
    let result = parse_dialogue(chinese_dialogue);
    assert!(result.is_some());
    let (speaker, content) = result.unwrap();
    assert_eq!(speaker, Some("羽艾".to_string()));
    assert_eq!(content, "你好");

    // 英文冒号和引号
    let result = parse_dialogue(r#"Test: "Hello""#);
    assert!(result.is_some());
    let (speaker, content) = result.unwrap();
    assert_eq!(speaker, Some("Test".to_string()));
    assert_eq!(content, "Hello");

    // 旁白
    let narration = "：\u{201C}这是旁白\u{201D}";
    let result = parse_dialogue(narration);
    assert!(result.is_some());
    let (speaker, content) = result.unwrap();
    assert_eq!(speaker, None);
    assert_eq!(content, "这是旁白");
}

// -------------------------------------------------------------------------
// Parser 集成测试
// -------------------------------------------------------------------------

/// 测试 chapter 解析：
/// - `# Chapter 1` 生成 level=1 的 Chapter 节点
/// - 超过 6 级标题（7 个 `#`）会被忽略
#[test]
fn test_parse_chapter() {
    let node = parse_single_node("# Chapter 1");
    assert!(matches!(
        node,
        ScriptNode::Chapter { title, level: 1 } if title == "Chapter 1"
    ));

    let script = parse_ok("####### too deep");
    assert_eq!(script.len(), 0);
}

/// 测试 label 解析：
/// - 英文标签：`**start**`
/// - 中文标签：`**选择支1**`
#[test]
fn test_parse_label() {
    let cases = [("**start**", "start"), ("**选择支1**", "选择支1")];
    for (input, expected) in cases {
        let node = parse_single_node(input);
        assert!(
            matches!(node, ScriptNode::Label { name } if name == expected),
            "input={input}"
        );
    }
}

#[test]
fn test_parse_dialogue() {
    let mut parser = Parser::new();

    // 中文冒号和引号
    let chinese_dialogue = "羽艾：\u{201C}你好\u{201D}";
    let script = parser.parse("test", chinese_dialogue).unwrap();
    assert!(matches!(
        &script.nodes[0],
        ScriptNode::Dialogue { speaker: Some(s), content, .. } if s == "羽艾" && content == "你好"
    ));

    // 英文冒号和引号
    let script = parser.parse("test", r#"Test: "Hello""#).unwrap();
    assert!(matches!(
        &script.nodes[0],
        ScriptNode::Dialogue { speaker: Some(s), content, .. } if s == "Test" && content == "Hello"
    ));

    // 旁白
    let narration = "：\u{201C}这是旁白\u{201D}";
    let script = parser.parse("test", narration).unwrap();
    assert!(matches!(
        &script.nodes[0],
        ScriptNode::Dialogue { speaker: None, content, .. } if content == "这是旁白"
    ));
}

/// 测试 show 指令：
/// - 完整格式（含 path/alias/position/transition）
/// - 简化格式（无 path）
/// - 空格容错（标准/无空格/多空格）
/// - 无过渡效果
/// - 行内代码 effect
/// - 相对路径
#[test]
fn test_parse_show_character() {
    let node = parse_single_node(
        r#"show <img src="assets/char.png" /> as royu at center with Dissolve(1.5)"#,
    );
    assert!(matches!(
        node,
        ScriptNode::ShowCharacter { path: Some(path), alias, position: Position::Center, transition: Some(t) }
        if path.as_str() == "assets/char.png" && alias == "royu" && t.name == "Dissolve"
    ));

    let node = parse_single_node(r#"show beifeng at left"#);
    assert!(matches!(
        node,
        ScriptNode::ShowCharacter { path: None, alias, position: Position::Left, transition: None }
        if alias == "beifeng"
    ));

    let whitespace_cases = [
        r#"show <img src="assets/bg2.jpg" /> as 红叶 at left"#,
        r#"show<img src="assets/bg2.jpg" />as 红叶 at left"#,
        r#"show   <img src="assets/bg2.jpg" />   as 红叶 at left"#,
    ];
    for input in whitespace_cases {
        let node = parse_single_node(input);
        assert!(
            matches!(
                node,
                ScriptNode::ShowCharacter { alias, position: Position::Left, .. } if alias == "红叶"
            ),
            "input={input}"
        );
    }

    let node = parse_single_node(r#"show <img src="assets/bg2.jpg" /> as 红叶 at left"#);
    assert!(matches!(
        node,
        ScriptNode::ShowCharacter { alias, path: Some(path), position: Position::Left, transition: None }
        if alias == "红叶" && path.as_str() == "assets/bg2.jpg"
    ));

    let node = parse_single_node(
        r#"show <img src="assets/bg2.jpg" /> as 红叶 at left with `Dissolve(2.0, 0.5)`"#,
    );
    if let ScriptNode::ShowCharacter {
        transition: Some(t),
        ..
    } = node
    {
        assert!(t.name.contains("Dissolve") || t.name == "`Dissolve(2.0, 0.5)`");
    } else {
        panic!("Expected ShowCharacter with transition");
    }

    let node =
        parse_single_node(r#"show <img src="../characters/北风.png" /> as beifeng at center"#);
    assert!(matches!(
        node,
        ScriptNode::ShowCharacter { path: Some(path), alias, .. }
        if path.as_str() == "../characters/北风.png" && alias == "beifeng"
    ));
}

/// 测试 hide 指令：
/// - 带过渡效果（with fade）
/// - 不带过渡效果
#[test]
fn test_parse_hide_character() {
    let node = parse_single_node("hide royu with fade");
    assert!(matches!(
        node,
        ScriptNode::HideCharacter { alias, transition: Some(t) }
        if alias == "royu" && t.name == "fade"
    ));

    let node = parse_single_node("hide 红叶");
    assert!(matches!(
        node,
        ScriptNode::HideCharacter { alias, transition: None } if alias == "红叶"
    ));
}

/// 测试 changeBG 指令：
/// - 标准解析（with dissolve）
/// - 不带过渡效果
/// - 空格容错（无空格/多空格）
/// - 行内代码 effect
/// - 相对路径
#[test]
fn test_parse_change_bg() {
    let node = parse_single_node(r#"changeBG <img src="assets/bg.png" /> with dissolve"#);
    assert!(matches!(
        node,
        ScriptNode::ChangeBG { path, transition: Some(t) }
        if path == "assets/bg.png" && t.name == "dissolve"
    ));

    let node = parse_single_node(r#"changeBG <img src="assets/bg2.jpg" />"#);
    assert!(matches!(
        node,
        ScriptNode::ChangeBG { path, transition: None } if path == "assets/bg2.jpg"
    ));

    let whitespace_cases = [
        r#"changeBG<img src="assets/bg2.jpg" />with dissolve"#,
        r#"changeBG   <img src="assets/bg2.jpg" />   with dissolve"#,
    ];
    for input in whitespace_cases {
        let node = parse_single_node(input);
        assert!(
            matches!(
                node,
                ScriptNode::ChangeBG { path, transition: Some(t) }
                if path == "assets/bg2.jpg" && t.name == "dissolve"
            ),
            "input={input}"
        );
    }

    let node =
        parse_single_node(r#"changeBG <img src="assets/bg2.jpg" /> with `Dissolve(2.0, 0.5)`"#);
    if let ScriptNode::ChangeBG {
        transition: Some(t),
        ..
    } = node
    {
        assert!(!t.name.is_empty());
    } else {
        panic!("Expected ChangeBG with transition");
    }

    let node = parse_single_node(r#"changeBG <img src="../backgrounds/bg.jpg" /> with `dissolve`"#);
    assert!(matches!(
        node,
        ScriptNode::ChangeBG { path, .. } if path == "../backgrounds/bg.jpg"
    ));
}

#[test]
fn test_parse_choice_table() {
    let mut parser = Parser::new();
    let text = r#"
| 选择 |        |
| ---- | ------ |
| 选项A | label_a |
| 选项B | label_b |
"#;
    let script = parser.parse("test", text).unwrap();

    assert!(matches!(
        &script.nodes[0],
        ScriptNode::Choice { style: Some(s), options }
        if s == "选择" && options.len() == 2
    ));
}

#[test]
fn test_parse_transition_with_args() {
    let mut parser = Parser::new();
    let script = parser
        .parse(
            "test",
            r#"changeBG <img src="bg.png" /> with Dissolve(1.5)"#,
        )
        .unwrap();

    if let ScriptNode::ChangeBG {
        transition: Some(t),
        ..
    } = &script.nodes[0]
    {
        assert_eq!(t.name, "Dissolve");
        assert_eq!(t.args.len(), 1);
        assert!(matches!(&t.args[0], (None, TransitionArg::Number(n)) if (*n - 1.5).abs() < 0.001));
    } else {
        panic!("Expected ChangeBG node");
    }
}

#[test]
fn test_parse_transition_with_named_args() {
    let mut parser = Parser::new();
    let script = parser
        .parse(
            "test",
            r#"changeBG <img src="bg.png" /> with Dissolve(duration: 2.0)"#,
        )
        .unwrap();

    if let ScriptNode::ChangeBG {
        transition: Some(t),
        ..
    } = &script.nodes[0]
    {
        assert_eq!(t.name, "Dissolve");
        assert_eq!(t.args.len(), 1);
        assert!(
            matches!(&t.args[0], (Some(key), TransitionArg::Number(n)) if key == "duration" && (*n - 2.0).abs() < 0.001)
        );
        assert_eq!(t.get_duration(), Some(2.0));
    } else {
        panic!("Expected ChangeBG node");
    }
}

#[test]
fn test_parse_full_script() {
    let mut parser = Parser::new();
    let text = r#"
# Chapter 1

changeBG <img src="assets/bg.png" /> with dissolve

羽艾："你好"

："这是旁白"

show <img src="assets/char.png" /> as protagonist at center with dissolve

**choice_point**

| 选择 |        |
| ---- | ------ |
| 继续 | cont |
| 结束 | end |

**cont**

羽艾："继续"

**end**

hide protagonist with fade
"#;

    let script = parser.parse("test", text).unwrap();

    // 验证节点数量
    assert!(script.len() >= 8);

    // 验证标签索引
    assert!(script.find_label("choice_point").is_some());
    assert!(script.find_label("cont").is_some());
    assert!(script.find_label("end").is_some());
}

// -------------------------------------------------------------------------
// 从 C# Parser.cs 移植的测试用例
// -------------------------------------------------------------------------

/// 测试对话解析：冒号前后有空格
#[test]
fn test_parse_dialogue_with_spaces() {
    // 英文冒号前后有空格
    let result = parse_dialogue(r#"红叶 : "台词""#);
    assert!(result.is_some());
    let (speaker, content) = result.unwrap();
    assert_eq!(speaker, Some("红叶".to_string()));
    assert_eq!(content, "台词");
}

/// 测试对话解析：内容包含特殊字符
#[test]
fn test_parse_dialogue_special_chars() {
    let result = parse_dialogue(r#"红叶: "台词 abab;:.""#);
    assert!(result.is_some());
    let (speaker, content) = result.unwrap();
    assert_eq!(speaker, Some("红叶".to_string()));
    assert_eq!(content, "台词 abab;:.");
}

/// 测试选择分支：竖排样式
#[test]
fn test_parse_branch_vertical() {
    let mut parser = Parser::new();
    let text = r#"| 竖排  |        |
| ----- | ------ |
| 选项1 | label1 |
| 选项2 | label2 |
| 选项3 | label3 |"#;

    let script = parser.parse("test", text).unwrap();

    if let ScriptNode::Choice { style, options } = &script.nodes[0] {
        assert_eq!(style.as_deref(), Some("竖排"));
        assert_eq!(options.len(), 3);
        assert_eq!(options[0].text, "选项1");
        assert_eq!(options[0].target_label, "label1");
        assert_eq!(options[1].text, "选项2");
        assert_eq!(options[1].target_label, "label2");
        assert_eq!(options[2].text, "选项3");
        assert_eq!(options[2].target_label, "label3");
    } else {
        panic!("Expected Choice node");
    }
}

// 测试 img 标签解析：带 style 和 alt 属性
// 综合测试：模拟真实脚本（来自 C# 测试）
//=========================================================================
// goto 语法测试
//=========================================================================

/// 测试 goto 指令：
/// - `goto **start**` 基本语法
/// - `goto **选择支1**` 中文标签
/// - `goto  **end_scene**` 多空格兼容
#[test]
fn test_parse_goto() {
    let cases = [
        ("goto **start**", "start"),
        ("goto **选择支1**", "选择支1"),
        ("goto  **end_scene**", "end_scene"),
    ];

    for (input, expected) in cases {
        let node = parse_single_node(input);
        assert!(
            matches!(node, ScriptNode::Goto { target_label } if target_label == expected),
            "input={input}"
        );
    }
}

#[test]
fn test_parse_blockquote_comment_line() {
    let mut parser = Parser::new();
    let script = parser
        .parse("test", "> 说明：这里是注释，不应参与脚本解析")
        .unwrap();
    assert_eq!(script.nodes.len(), 0);
    assert!(parser.warnings().is_empty());
}

//=========================================================================
// audio 语法测试
//=========================================================================

/// 测试 audio/stopBGM 指令：
/// - audio SFX（无 loop）
/// - audio BGM（含 loop）
/// - audio 相对路径
/// - stopBGM
#[test]
fn test_parse_audio_commands() {
    let play_audio_cases = [
        (
            r#"<audio src="sfx/ding.mp3"></audio>"#,
            "sfx/ding.mp3",
            false,
        ),
        (
            r#"<audio src="bgm/Signal.mp3"></audio> loop"#,
            "bgm/Signal.mp3",
            true,
        ),
        (
            r#"<audio src="../bgm/music.mp3"></audio> loop"#,
            "../bgm/music.mp3",
            true,
        ),
    ];

    for (input, expected_path, expected_is_bgm) in play_audio_cases {
        let node = parse_single_node(input);
        assert!(
            matches!(
                node,
                ScriptNode::PlayAudio { path, is_bgm }
                    if path == expected_path && is_bgm == expected_is_bgm
            ),
            "input={input}"
        );
    }

    let stop_node = parse_single_node("stopBGM");
    assert!(matches!(stop_node, ScriptNode::StopBgm));
}

#[test]
fn test_parse_bgm_duck_unduck() {
    let duck = parse_single_node("bgmDuck");
    assert!(matches!(duck, ScriptNode::BgmDuck));

    let unduck = parse_single_node("bgmUnduck");
    assert!(matches!(unduck, ScriptNode::BgmUnduck));

    // case insensitive
    let duck_lower = parse_single_node("bgmduck");
    assert!(matches!(duck_lower, ScriptNode::BgmDuck));

    let unduck_upper = parse_single_node("BGMUNDUCK");
    assert!(matches!(unduck_upper, ScriptNode::BgmUnduck));
}

//=========================================================================
// 相对路径测试
//=========================================================================

//=========================================================================
// 综合测试：包含 goto 和 audio 的完整脚本
//=========================================================================

//=========================================================================
// changeScene / changeBG 职责分离测试
//=========================================================================

/// 测试 changeBG 过渡策略：
/// - 允许：`dissolve`、`Dissolve(1.5)`
/// - 禁止：`fade`、`fadewhite`、`rule`
///   测试 changeScene 标准过渡：
/// - `Dissolve(duration: 1)`
/// - `Fade(duration: 1.5)`
/// - `FadeWhite(duration: 2)`
///   测试 changeScene rule 过渡：
/// - 带参数：`mask + duration + reversed`
/// - 无参数：仅 `mask`
///   测试 changeScene 缺失 with 子句时报错：
/// - `bg.jpg` 路径
/// - `assets/bg.png` 路径
#[test]
fn test_parse_show_simplified_requires_at() {
    let mut parser = Parser::new();
    let err = parser.parse("test", "show alice").unwrap_err();
    assert!(matches!(
        err,
        crate::error::ParseError::MissingParameter { .. }
    ));
}

#[test]
fn test_parse_hide_missing_alias() {
    let mut parser = Parser::new();
    let err = parser.parse("test", "hide").unwrap_err();
    assert!(matches!(
        err,
        crate::error::ParseError::MissingParameter { .. }
    ));
}

#[test]
fn test_unknown_line_produces_warning() {
    let mut parser = Parser::new();
    let script = parser.parse("test", "??? what is this").unwrap();
    assert_eq!(script.len(), 0);
    assert_eq!(parser.warnings().len(), 1);
    assert!(parser.warnings()[0].contains("无法识别"));
}

//=========================================================================
// set 指令测试
//=========================================================================

// 测试 set 指令：
// - 正常分支：String/Bool(true)/Bool(false)/Int
// - 错误分支：缺少 `$`、缺少 `=`
//=========================================================================
// 条件分支测试
//=========================================================================

// 测试条件分支解析：
// - `if ... endif`（单分支）
// - `if/else`
// - `if/elseif/else`
// - 复合逻辑条件（And）
// - 缺失 `endif` 报错
//=========================================================================
// 表达式解析测试
//=========================================================================

// 测试表达式解析：
// - 比较：`==` / `!=`
// - 字面量：`true` / `false`
// - 逻辑：`not` / `and` / `or`
// - 括号表达式
// - 错误：空表达式、右括号缺失
// =========================================================================
// 阶段 24：TextBox / ClearCharacters 指令解析测试
// =========================================================================

///   测试 TextBox/ClearCharacters 指令：
/// - 单行命令：textBoxHide/textBoxShow/textBoxClear/clearCharacters
/// - 大小写不敏感：TEXTBOXHIDE/TextBoxShow/textboxclear/CLEARCHARACTERS
#[test]
fn test_parse_textbox_commands() {
    let single_line_cases = [
        ("textBoxHide", ScriptNode::TextBoxHide),
        ("textBoxShow", ScriptNode::TextBoxShow),
        ("textBoxClear", ScriptNode::TextBoxClear),
        ("clearCharacters", ScriptNode::ClearCharacters),
    ];

    for (input, expected) in single_line_cases {
        let node = parse_single_node(input);
        assert_eq!(node, expected, "input={input}");
    }

    let script = parse_ok("TEXTBOXHIDE\nTextBoxShow\ntextboxclear\nCLEARCHARACTERS");
    assert_eq!(script.nodes.len(), 4);
    let expected = [
        ScriptNode::TextBoxHide,
        ScriptNode::TextBoxShow,
        ScriptNode::TextBoxClear,
        ScriptNode::ClearCharacters,
    ];
    for (index, expected_node) in expected.into_iter().enumerate() {
        assert_eq!(script.nodes[index], expected_node);
    }
}

#[test]
fn test_parse_scene_effect_no_args() {
    let node = parse_single_node("sceneEffect shakeSmall");
    match node {
        ScriptNode::SceneEffect { effect } => {
            assert_eq!(effect.name, "shakeSmall");
            assert!(effect.args.is_empty());
        }
        other => panic!("Expected SceneEffect, got: {:?}", other),
    }
}

#[test]
fn test_parse_scene_effect_with_duration() {
    let node = parse_single_node("sceneEffect blurIn(duration: 0.75)");
    match node {
        ScriptNode::SceneEffect { effect } => {
            assert_eq!(effect.name, "blurIn");
            assert_eq!(effect.get_duration(), Some(0.75));
        }
        other => panic!("Expected SceneEffect, got: {:?}", other),
    }
}

#[test]
fn test_parse_scene_effect_with_level() {
    let node = parse_single_node("sceneEffect dimStep(level: 3)");
    match node {
        ScriptNode::SceneEffect { effect } => {
            assert_eq!(effect.name, "dimStep");
            assert!(matches!(
                effect.get_named("level"),
                Some(crate::command::TransitionArg::Number(n)) if (*n - 3.0).abs() < f64::EPSILON
            ));
        }
        other => panic!("Expected SceneEffect, got: {:?}", other),
    }
}

#[test]
fn test_parse_scene_effect_case_insensitive() {
    let node = parse_single_node("SceneEffect bounceSmall");
    match node {
        ScriptNode::SceneEffect { effect } => {
            assert_eq!(effect.name, "bounceSmall");
        }
        other => panic!("Expected SceneEffect, got: {:?}", other),
    }
}

#[test]
fn test_parse_scene_effect_missing_name() {
    let err = parse_err("sceneEffect");
    assert!(
        format!("{:?}", err).contains("MissingParameter"),
        "expected MissingParameter, got: {:?}",
        err
    );
}

#[test]
fn test_parse_title_card() {
    let node = parse_single_node(r#"titleCard "Hello World" (duration: 1.5)"#);
    match node {
        ScriptNode::TitleCard { text, duration } => {
            assert_eq!(text, "Hello World");
            assert!((duration - 1.5).abs() < f64::EPSILON);
        }
        other => panic!("Expected TitleCard, got: {:?}", other),
    }
}

#[test]
fn test_parse_title_card_default_duration() {
    let node = parse_single_node(r#"titleCard "No Duration""#);
    match node {
        ScriptNode::TitleCard { text, duration } => {
            assert_eq!(text, "No Duration");
            assert!((duration - 1.0).abs() < f64::EPSILON);
        }
        other => panic!("Expected TitleCard, got: {:?}", other),
    }
}

#[test]
fn test_parse_title_card_missing_text() {
    let err = parse_err("titleCard");
    assert!(
        format!("{:?}", err).contains("MissingParameter"),
        "expected MissingParameter, got: {:?}",
        err
    );
}

// =========================================================================
// extend 指令测试
// =========================================================================

#[test]
fn test_parse_extend_basic() {
    let node = parse_single_node(r#"extend "继续说话""#);
    assert!(matches!(
        node,
        ScriptNode::Extend { content, inline_effects, no_wait: false }
            if content == "继续说话" && inline_effects.is_empty()
    ));
}

#[test]
fn test_parse_extend_with_no_wait() {
    let node = parse_single_node(r#"extend "继续说话" -->"#);
    assert!(matches!(
        node,
        ScriptNode::Extend { content, no_wait: true, .. }
            if content == "继续说话"
    ));
}

#[test]
fn test_parse_extend_with_inline_effects() {
    let node = parse_single_node(r#"extend "接着{wait}继续""#);
    if let ScriptNode::Extend {
        content,
        inline_effects,
        ..
    } = node
    {
        assert_eq!(content, "接着继续");
        assert!(!inline_effects.is_empty());
    } else {
        panic!("Expected Extend node");
    }
}

#[test]
fn test_parse_extend_missing_text() {
    let err = parse_err("extend");
    assert!(matches!(
        err,
        crate::error::ParseError::MissingParameter { .. }
    ));
}

#[test]
fn test_parse_extend_missing_quotes() {
    let err = parse_err("extend hello");
    assert!(matches!(
        err,
        crate::error::ParseError::MissingParameter { .. }
    ));
}

// =========================================================================
// --> (no_wait) 修饰符测试
// =========================================================================

#[test]
fn test_parse_dialogue_with_no_wait_arrow() {
    let node = parse_single_node(r#"角色："台词" -->"#);
    assert!(matches!(node, ScriptNode::Dialogue { no_wait: true, .. }));
}

#[test]
fn test_parse_dialogue_without_no_wait() {
    let node = parse_single_node(r#"角色："没有箭头""#);
    assert!(matches!(node, ScriptNode::Dialogue { no_wait: false, .. }));
}

// =========================================================================
// set 指令边界测试
// =========================================================================

#[test]
fn test_parse_set_var_empty_name() {
    let err = parse_err("set $ = true");
    assert!(matches!(
        err,
        crate::error::ParseError::MissingParameter { .. }
    ));
}

#[test]
fn test_parse_set_var_invalid_name_chars() {
    let err = parse_err("set $na-me = true");
    assert!(matches!(err, crate::error::ParseError::InvalidLine { .. }));
}

// =========================================================================
// titleCard 边界测试
// =========================================================================

#[test]
fn test_parse_title_card_missing_closing_quote() {
    let err = parse_err(r#"titleCard "unclosed"#);
    assert!(format!("{:?}", err).contains("InvalidParameter"));
}

#[test]
fn test_parse_title_card_negative_duration() {
    let err = parse_err(r#"titleCard "text" (duration: -1)"#);
    assert!(format!("{:?}", err).contains("InvalidParameter"));
}

#[test]
fn test_parse_title_card_positional_duration() {
    let node = parse_single_node(r#"titleCard "text" (2.5)"#);
    match node {
        ScriptNode::TitleCard { text, duration } => {
            assert_eq!(text, "text");
            assert!((duration - 2.5).abs() < f64::EPSILON);
        }
        other => panic!("Expected TitleCard, got: {:?}", other),
    }
}

#[test]
fn test_parse_title_card_missing_closing_paren() {
    let err = parse_err(r#"titleCard "text" (duration: 1"#);
    assert!(format!("{:?}", err).contains("InvalidParameter"));
}

// =========================================================================
// callScript 边界测试
// =========================================================================

#[test]
fn test_parse_call_script_extra_params_after_link() {
    let err = parse_err("callScript [ch1](ch1.md) extra_stuff");
    assert!(matches!(err, crate::error::ParseError::InvalidLine { .. }));
}

#[test]
fn test_parse_call_script_empty_label() {
    let err = parse_err("callScript [ ](ch1.md)");
    assert!(matches!(err, crate::error::ParseError::InvalidLine { .. }));
}

#[test]
fn test_parse_call_script_empty_path() {
    let err = parse_err("callScript [ch1]( )");
    assert!(matches!(err, crate::error::ParseError::InvalidLine { .. }));
}

// =========================================================================
// audio unicode loop 标识测试
// =========================================================================

#[test]
fn test_parse_audio_unicode_loop() {
    let node = parse_single_node("<audio src=\"bgm.mp3\"></audio> \u{267E}");
    assert!(matches!(
        node,
        ScriptNode::PlayAudio { path, is_bgm: true } if path == "bgm.mp3"
    ));
}

// =========================================================================
// show 指令边界测试
// =========================================================================

#[test]
fn test_parse_show_invalid_position() {
    let err = parse_err(r#"show <img src="char.png" /> as alice at invalid_pos"#);
    assert!(matches!(
        err,
        crate::error::ParseError::InvalidParameter { .. }
    ));
}

#[test]
fn test_parse_show_missing_as() {
    let err = parse_err(r#"show <img src="char.png" /> at center"#);
    assert!(matches!(
        err,
        crate::error::ParseError::MissingParameter { .. }
    ));
}

// =========================================================================
// changeBG 边界测试
// =========================================================================

#[test]
fn test_parse_change_bg_missing_img() {
    let err = parse_err("changeBG with dissolve");
    assert!(matches!(
        err,
        crate::error::ParseError::MissingParameter { .. }
    ));
}

// =========================================================================
// label 边界测试
// =========================================================================

#[test]
fn test_parse_label_with_stars_inside() {
    let script = parse_ok("***bold***");
    assert_eq!(
        script.len(),
        0,
        "label with * inside should not parse as label"
    );
}

// =========================================================================
// chapter 边界测试
// =========================================================================

#[test]
fn test_parse_chapter_empty_title() {
    let script = parse_ok("## ");
    assert_eq!(script.len(), 0);
}

#[test]
fn test_parse_chapter_levels() {
    for level in 1..=6u8 {
        let input = format!("{} Title", "#".repeat(level as usize));
        let node = parse_single_node(&input);
        assert!(
            matches!(node, ScriptNode::Chapter { level: l, .. } if l == level),
            "level={level}"
        );
    }
}

// =========================================================================
// wait 边界测试 - zero duration
// =========================================================================

#[test]
fn test_parse_wait_zero() {
    let err = parse_err("wait 0");
    assert!(format!("{:?}", err).contains("InvalidParameter"));
}

// =========================================================================
// 条件块边界测试
// =========================================================================

#[test]
fn test_parse_conditional_with_multiple_body_commands() {
    let input = r#"
if $flag == true
  textBoxHide
  clearCharacters
  ："条件内多条命令"
endif
"#;
    let node = parse_single_node(input);
    if let ScriptNode::Conditional { branches } = node {
        assert_eq!(branches.len(), 1);
        assert_eq!(branches[0].body.len(), 3);
    } else {
        panic!("Expected Conditional node");
    }
}

#[test]
fn test_parse_conditional_empty_body() {
    let input = r#"
if $flag == true
endif
"#;
    let node = parse_single_node(input);
    if let ScriptNode::Conditional { branches } = node {
        assert_eq!(branches.len(), 1);
        assert!(branches[0].body.is_empty());
    } else {
        panic!("Expected Conditional node");
    }
}

// =========================================================================
// extract_transition_from_line 边界测试
// =========================================================================

#[test]
fn test_extract_transition_no_closing_backtick() {
    let parser = phase2::Phase2Parser::new();
    let t =
        parser.extract_transition_from_line(r#"changeBG <img src="bg.png" /> with `Dissolve(1.0)"#);
    assert!(t.is_some());
    let t = t.unwrap();
    assert_eq!(t.name, "Dissolve");
}

#[test]
fn test_extract_transition_inline_backtick_with_internal_backtick() {
    let parser = phase2::Phase2Parser::new();
    let t = parser.extract_transition_from_line(r#"changeBG <img src="bg.png" /> with `Dissolve`"#);
    assert!(t.is_some());
    let t = t.unwrap();
    assert_eq!(t.name, "Dissolve");
}

// =========================================================================
// goto 边界测试 - 不带星号的 label
// =========================================================================

#[test]
fn test_parse_goto_without_stars() {
    let node = parse_single_node("goto some_label");
    assert!(matches!(
        node,
        ScriptNode::Goto { target_label } if target_label == "some_label"
    ));
}

#[test]
fn test_parse_goto_missing_label() {
    let err = parse_err("goto");
    assert!(matches!(
        err,
        crate::error::ParseError::MissingParameter { .. }
    ));
}

#[test]
fn test_parse_goto_malformed_emphasis_is_not_stripped() {
    let node = parse_single_node("goto **end");
    assert!(matches!(
        node,
        ScriptNode::Goto { target_label } if target_label == "**end"
    ));

    let node = parse_single_node("goto end**");
    assert!(matches!(
        node,
        ScriptNode::Goto { target_label } if target_label == "end**"
    ));
}

// =========================================================================
// phase1: nested conditional blocks
// =========================================================================

#[test]
fn test_parse_nested_conditionals() {
    let input = r#"
if $outer == true
  if $inner == true
    ："内层"
  endif
  ："外层"
endif
"#;
    let node = parse_single_node(input);
    if let ScriptNode::Conditional { branches } = node {
        assert_eq!(branches.len(), 1);
        assert!(!branches[0].body.is_empty());
    } else {
        panic!("Expected Conditional node");
    }
}

// =========================================================================
// fullRestart 大小写测试已有，添加混合大小写
// =========================================================================

// =========================================================================
// changeScene 边界测试 - 缺少图片路径
// =========================================================================

#[test]
fn test_parse_change_scene_missing_img() {
    let err = parse_err("changeScene with Dissolve(1.0)");
    assert!(matches!(
        err,
        crate::error::ParseError::MissingParameter { .. }
    ));
}

// =========================================================================
// 表达式解析器边界测试
// =========================================================================

#[test]
fn test_parse_expression_single_quoted_string() {
    let expr = parse_expression("$name == 'Alice'", 1).unwrap();
    assert!(matches!(expr, crate::script::Expr::Eq(_, _)));
}

#[test]
fn test_parse_expression_unclosed_string() {
    let err = parse_expression(r#"$name == "unclosed"#, 1).unwrap_err();
    assert!(format!("{:?}", err).contains("未闭合"));
}

#[test]
fn test_parse_expression_negative_number() {
    let expr = parse_expression("$count == -5", 1).unwrap();
    assert!(matches!(expr, crate::script::Expr::Eq(_, _)));
}

#[test]
fn test_parse_expression_integer_literal() {
    let expr = parse_expression("42", 1).unwrap();
    assert!(matches!(
        expr,
        crate::script::Expr::Literal(crate::state::VarValue::Int(42))
    ));
}

#[test]
fn test_parse_expression_unexpected_char() {
    let err = parse_expression("@invalid", 1).unwrap_err();
    assert!(matches!(err, crate::error::ParseError::InvalidLine { .. }));
}

#[test]
fn test_parse_expression_trailing_content() {
    let err = parse_expression("true garbage", 1).unwrap_err();
    assert!(format!("{:?}", err).contains("无法解析"));
}

#[test]
fn test_parse_expression_unexpected_end_after_eq() {
    let err = parse_expression("$a ==", 1).unwrap_err();
    assert!(matches!(err, crate::error::ParseError::InvalidLine { .. }));
}

#[test]
fn test_parse_expression_empty_identifier() {
    let err = parse_expression("$ == true", 1).unwrap_err();
    assert!(matches!(err, crate::error::ParseError::InvalidLine { .. }));
}

#[test]
fn test_parse_expression_double_not() {
    let expr = parse_expression("not not $flag", 1).unwrap();
    assert!(matches!(expr, crate::script::Expr::Not(_)));
}

#[test]
fn test_parse_expression_complex_nested_parens() {
    let expr = parse_expression("($a == true) and (($b == false) or ($c == true))", 1).unwrap();
    assert!(matches!(expr, crate::script::Expr::And(_, _)));
}

#[test]
fn test_parse_expression_number_only_minus_sign() {
    let err = parse_expression("$x == -", 1).unwrap_err();
    assert!(matches!(err, crate::error::ParseError::InvalidLine { .. }));
}

// =========================================================================
// cutscene 测试
// =========================================================================

#[test]
fn test_parse_cutscene() {
    let node = parse_single_node(r#"cutscene "audio/ending_HVC_bgm.webm""#);
    match node {
        ScriptNode::Cutscene { path } => {
            assert_eq!(path, "audio/ending_HVC_bgm.webm");
        }
        other => panic!("Expected Cutscene, got: {:?}", other),
    }
}

#[test]
fn test_parse_cutscene_case_insensitive() {
    let node = parse_single_node(r#"CUTSCENE "video.webm""#);
    assert!(matches!(node, ScriptNode::Cutscene { .. }));

    let node2 = parse_single_node(r#"Cutscene "video.webm""#);
    assert!(matches!(node2, ScriptNode::Cutscene { .. }));
}

#[test]
fn test_parse_cutscene_missing_path() {
    let err = parse_err("cutscene");
    assert!(
        format!("{:?}", err).contains("MissingParameter"),
        "expected MissingParameter, got: {:?}",
        err
    );
}

#[test]
fn test_parse_cutscene_missing_quotes() {
    let err = parse_err("cutscene audio/video.webm");
    assert!(
        format!("{:?}", err).contains("InvalidParameter"),
        "expected InvalidParameter, got: {:?}",
        err
    );
}

#[test]
fn test_parse_cutscene_missing_closing_quote() {
    let err = parse_err(r#"cutscene "unclosed"#);
    assert!(format!("{:?}", err).contains("InvalidParameter"));
}

#[test]
fn test_parse_cutscene_empty_path() {
    let err = parse_err(r#"cutscene """#);
    assert!(format!("{:?}", err).contains("InvalidParameter"));
}
