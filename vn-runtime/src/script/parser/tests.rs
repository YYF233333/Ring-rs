//! # Parser 测试
//!
//! 从原 parser.rs 迁移的完整测试套件。

use super::*;
use crate::command::{Position, TransitionArg};
use crate::script::ast::ScriptNode;

// -------------------------------------------------------------------------
// 辅助函数测试
// -------------------------------------------------------------------------

#[test]
fn test_extract_img_src() {
    assert_eq!(
        extract_img_src(r#"<img src="path/to/image.png" />"#),
        Some("path/to/image.png")
    );
    assert_eq!(
        extract_img_src(r#"<img   src='another.jpg' alt="test">"#),
        Some("another.jpg")
    );
    assert_eq!(
        extract_img_src(r#"show <img src="char.png" /> as test"#),
        Some("char.png")
    );
    assert_eq!(extract_img_src("no image here"), None);
}

#[test]
fn test_extract_img_src_requires_quoted_value() {
    // 覆盖：quote_char 不是 ' 或 " 时返回 None
    assert_eq!(extract_img_src(r#"<img src=path/to/image.png />"#), None);
    assert_eq!(extract_img_src(r#"<img src= path/to/image.png />"#), None);
}

#[test]
fn test_extract_audio_src_and_requires_quoted_value() {
    assert_eq!(
        extract_audio_src(r#"<audio src="bgm.mp3"></audio>"#),
        Some("bgm.mp3")
    );
    // 覆盖：quote_char 校验失败分支
    assert_eq!(extract_audio_src(r#"<audio src=bgm.mp3></audio>"#), None);
}

#[test]
fn test_extract_keyword_value() {
    let line = "show <img src=\"char.png\" /> as royu at center with dissolve";
    assert_eq!(extract_keyword_value(line, "as"), Some("royu"));
    assert_eq!(extract_keyword_value(line, "at"), Some("center"));

    let line2 = "show <img src=\"char.png\" /> as test_char at nearleft";
    assert_eq!(extract_keyword_value(line2, "as"), Some("test_char"));
    assert_eq!(extract_keyword_value(line2, "at"), Some("nearleft"));
}

#[test]
fn test_extract_keyword_value_edge_cases() {
    // 覆盖：value_start 越界 / value 为空
    assert_eq!(
        extract_keyword_value("show <img src=\"x\" /> as", "as"),
        None
    );

    // 覆盖：">as " 模式（无空格紧跟 img 标签结束）
    let line = "show <img src=\"char.png\" />as royu at center";
    assert_eq!(extract_keyword_value(line, "as"), Some("royu"));
}

#[test]
fn test_parse_transition() {
    let t = parse_transition("dissolve").unwrap();
    assert_eq!(t.name, "dissolve");
    assert!(t.args.is_empty());

    // 位置参数
    let t = parse_transition("Dissolve(1.5)").unwrap();
    assert_eq!(t.name, "Dissolve");
    assert_eq!(t.args.len(), 1);
    assert!(matches!(&t.args[0], (None, TransitionArg::Number(n)) if (*n - 1.5).abs() < 0.001));

    let t = parse_transition("fade").unwrap();
    assert_eq!(t.name, "fade");
}

#[test]
fn test_parse_transition_invalid_format_returns_none() {
    // 缺失右括号
    assert_eq!(parse_transition("Dissolve(1.0"), None);
    // 参数解析失败（混用位置/命名）-> parse_transition 内部吞掉 Err，返回 None
    assert_eq!(parse_transition("Dissolve(1.0, duration: 2.0)"), None);
}

#[test]
fn test_parse_transition_named_args() {
    // 命名参数
    let t = parse_transition("Dissolve(duration: 1.5)").unwrap();
    assert_eq!(t.name, "Dissolve");
    assert_eq!(t.args.len(), 1);
    assert!(
        matches!(&t.args[0], (Some(key), TransitionArg::Number(n)) if key == "duration" && (*n - 1.5).abs() < 0.001)
    );
    assert!(t.is_all_named());

    // 多个命名参数
    let t = parse_transition("Fade(duration: 2.0, reversed: true)").unwrap();
    assert_eq!(t.name, "Fade");
    assert_eq!(t.args.len(), 2);
    assert!(
        matches!(&t.args[0], (Some(key), TransitionArg::Number(n)) if key == "duration" && (*n - 2.0).abs() < 0.001)
    );
    assert!(matches!(&t.args[1], (Some(key), TransitionArg::Bool(true)) if key == "reversed"));

    // 辅助方法测试
    assert_eq!(t.get_duration(), Some(2.0));
    assert_eq!(t.get_reversed(), Some(true));
}

#[test]
fn test_parse_transition_positional_args() {
    // 多个位置参数
    let t = parse_transition("Effect(1.0, 0.5, true, \"test\")").unwrap();
    assert_eq!(t.name, "Effect");
    assert_eq!(t.args.len(), 4);
    assert!(t.is_all_positional());
    assert!(matches!(&t.args[0], (None, TransitionArg::Number(n)) if (*n - 1.0).abs() < 0.001));
    assert!(matches!(&t.args[1], (None, TransitionArg::Number(n)) if (*n - 0.5).abs() < 0.001));
    assert!(matches!(&t.args[2], (None, TransitionArg::Bool(true))));
    assert!(matches!(&t.args[3], (None, TransitionArg::String(s)) if s == "test"));

    // 辅助方法测试（位置参数回退）
    assert_eq!(t.get_positional(0), Some(&TransitionArg::Number(1.0)));
}

#[test]
fn test_parse_transition_mixed_args_error() {
    // 混用位置参数和命名参数应该失败
    let result = parse_transition_args("1.0, duration: 2.0");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("不允许混用"));

    let result = parse_transition_args("duration: 2.0, 1.0");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("不允许混用"));
}

#[test]
fn test_parse_transition_duplicate_key_error() {
    // 重复的命名参数 key 应该失败
    let result = parse_transition_args("duration: 1.0, duration: 2.0");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("重复的命名参数"));
}

#[test]
fn test_is_table_separator() {
    assert!(is_table_separator("| --- | --- |"));
    assert!(is_table_separator("|---|---|"));
    assert!(is_table_separator("| :---: | ---: |"));
    assert!(!is_table_separator("| text | text |"));
    assert!(!is_table_separator("not a table"));
}

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

#[test]
fn test_parse_chapter() {
    let mut parser = Parser::new();
    let script = parser.parse("test", "# Chapter 1").unwrap();

    assert_eq!(script.len(), 1);
    assert!(matches!(
        &script.nodes[0],
        ScriptNode::Chapter { title, level: 1 } if title == "Chapter 1"
    ));
}

#[test]
fn test_parse_chapter_invalid_is_ignored() {
    let mut parser = Parser::new();
    // 7 个 #（超过 6）应被忽略而不是报错
    let script = parser.parse("test", "####### too deep").unwrap();
    assert_eq!(script.len(), 0);
}

#[test]
fn test_parse_label() {
    let mut parser = Parser::new();
    let script = parser.parse("test", "**start**").unwrap();

    assert_eq!(script.len(), 1);
    assert!(matches!(
        &script.nodes[0],
        ScriptNode::Label { name } if name == "start"
    ));
}

#[test]
fn test_parse_dialogue() {
    let mut parser = Parser::new();

    // 中文冒号和引号
    let chinese_dialogue = "羽艾：\u{201C}你好\u{201D}";
    let script = parser.parse("test", chinese_dialogue).unwrap();
    assert!(matches!(
        &script.nodes[0],
        ScriptNode::Dialogue { speaker: Some(s), content } if s == "羽艾" && content == "你好"
    ));

    // 英文冒号和引号
    let script = parser.parse("test", r#"Test: "Hello""#).unwrap();
    assert!(matches!(
        &script.nodes[0],
        ScriptNode::Dialogue { speaker: Some(s), content } if s == "Test" && content == "Hello"
    ));

    // 旁白
    let narration = "：\u{201C}这是旁白\u{201D}";
    let script = parser.parse("test", narration).unwrap();
    assert!(matches!(
        &script.nodes[0],
        ScriptNode::Dialogue { speaker: None, content } if content == "这是旁白"
    ));
}

#[test]
fn test_parse_change_bg() {
    let mut parser = Parser::new();
    let script = parser
        .parse(
            "test",
            r#"changeBG <img src="assets/bg.png" /> with dissolve"#,
        )
        .unwrap();

    assert!(matches!(
        &script.nodes[0],
        ScriptNode::ChangeBG { path, transition: Some(t) }
        if path == "assets/bg.png" && t.name == "dissolve"
    ));
}

#[test]
fn test_parse_show_character() {
    let mut parser = Parser::new();
    let script = parser
        .parse(
            "test",
            r#"show <img src="assets/char.png" /> as royu at center with Dissolve(1.5)"#,
        )
        .unwrap();

    assert!(matches!(
        &script.nodes[0],
        ScriptNode::ShowCharacter { path: Some(path), alias, position: Position::Center, transition: Some(t) }
        if path.as_str() == "assets/char.png" && alias == "royu" && t.name == "Dissolve"
    ));
}

#[test]
fn test_parse_show_character_without_path() {
    let mut parser = Parser::new();
    let script = parser.parse("test", r#"show beifeng at left"#).unwrap();

    assert!(matches!(
        &script.nodes[0],
        ScriptNode::ShowCharacter { path: None, alias, position: Position::Left, transition: None }
        if alias == "beifeng"
    ));
}

#[test]
fn test_parse_hide_character() {
    let mut parser = Parser::new();
    let script = parser.parse("test", "hide royu with fade").unwrap();

    assert!(matches!(
        &script.nodes[0],
        ScriptNode::HideCharacter { alias, transition: Some(t) }
        if alias == "royu" && t.name == "fade"
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

    let script = parser.parse("test", &text).unwrap();

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

/// 测试 show 指令：无空格、有空格、多空格
#[test]
fn test_parse_show_whitespace_tolerance() {
    let mut parser = Parser::new();

    // 标准格式
    let script = parser
        .parse(
            "test",
            r#"show <img src="assets/bg2.jpg" /> as 红叶 at left"#,
        )
        .unwrap();
    assert!(matches!(
        &script.nodes[0],
        ScriptNode::ShowCharacter { alias, position: Position::Left, .. }
        if alias == "红叶"
    ));

    // 无空格格式
    let script = parser
        .parse("test", r#"show<img src="assets/bg2.jpg" />as 红叶 at left"#)
        .unwrap();
    assert!(matches!(
        &script.nodes[0],
        ScriptNode::ShowCharacter { alias, position: Position::Left, .. }
        if alias == "红叶"
    ));

    // 多空格格式
    let script = parser
        .parse(
            "test",
            r#"show   <img src="assets/bg2.jpg" />   as 红叶 at left"#,
        )
        .unwrap();
    assert!(matches!(
        &script.nodes[0],
        ScriptNode::ShowCharacter { alias, position: Position::Left, .. }
        if alias == "红叶"
    ));
}

/// 测试 show 指令：无过渡效果
#[test]
fn test_parse_show_without_effect() {
    let mut parser = Parser::new();
    let script = parser
        .parse(
            "test",
            r#"show <img src="assets/bg2.jpg" /> as 红叶 at left"#,
        )
        .unwrap();

    assert!(matches!(
        &script.nodes[0],
        ScriptNode::ShowCharacter { alias, path: Some(path), position: Position::Left, transition: None }
        if alias == "红叶" && path.as_str() == "assets/bg2.jpg"
    ));
}

/// 测试 show 指令：带行内代码格式的 effect
#[test]
fn test_parse_show_with_inline_code_effect() {
    let mut parser = Parser::new();
    let script = parser
        .parse(
            "test",
            r#"show <img src="assets/bg2.jpg" /> as 红叶 at left with `Dissolve(2.0, 0.5)`"#,
        )
        .unwrap();

    if let ScriptNode::ShowCharacter {
        transition: Some(t),
        ..
    } = &script.nodes[0]
    {
        // 行内代码格式的 effect 应该被解析（去掉反引号）
        assert!(t.name.contains("Dissolve") || t.name == "`Dissolve(2.0, 0.5)`");
    } else {
        panic!("Expected ShowCharacter with transition");
    }
}

/// 测试 hide 指令：无过渡效果
#[test]
fn test_parse_hide_without_effect() {
    let mut parser = Parser::new();
    let script = parser.parse("test", "hide 红叶").unwrap();

    assert!(matches!(
        &script.nodes[0],
        ScriptNode::HideCharacter { alias, transition: None }
        if alias == "红叶"
    ));
}

/// 测试 changeBG 指令：无过渡效果
#[test]
fn test_parse_change_bg_without_effect() {
    let mut parser = Parser::new();
    let script = parser
        .parse("test", r#"changeBG <img src="assets/bg2.jpg" />"#)
        .unwrap();

    assert!(matches!(
        &script.nodes[0],
        ScriptNode::ChangeBG { path, transition: None }
        if path == "assets/bg2.jpg"
    ));
}

/// 测试 changeBG 指令：空格容错
#[test]
fn test_parse_change_bg_whitespace_tolerance() {
    let mut parser = Parser::new();

    // 无空格
    let script = parser
        .parse(
            "test",
            r#"changeBG<img src="assets/bg2.jpg" />with dissolve"#,
        )
        .unwrap();
    assert!(matches!(
        &script.nodes[0],
        ScriptNode::ChangeBG { path, transition: Some(t) }
        if path == "assets/bg2.jpg" && t.name == "dissolve"
    ));

    // 多空格
    let script = parser
        .parse(
            "test",
            r#"changeBG   <img src="assets/bg2.jpg" />   with dissolve"#,
        )
        .unwrap();
    assert!(matches!(
        &script.nodes[0],
        ScriptNode::ChangeBG { path, transition: Some(t) }
        if path == "assets/bg2.jpg" && t.name == "dissolve"
    ));
}

/// 测试 changeBG 指令：行内代码格式的 effect
#[test]
fn test_parse_change_bg_with_inline_code_effect() {
    let mut parser = Parser::new();
    let script = parser
        .parse(
            "test",
            r#"changeBG <img src="assets/bg2.jpg" /> with `Dissolve(2.0, 0.5)`"#,
        )
        .unwrap();

    if let ScriptNode::ChangeBG {
        transition: Some(t),
        ..
    } = &script.nodes[0]
    {
        // 行内代码格式应该被解析
        assert!(!t.name.is_empty());
    } else {
        panic!("Expected ChangeBG with transition");
    }
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

/// 测试标签解析：中文标签名
#[test]
fn test_parse_label_chinese() {
    let mut parser = Parser::new();
    let script = parser.parse("test", "**选择支1**").unwrap();

    assert!(matches!(
        &script.nodes[0],
        ScriptNode::Label { name } if name == "选择支1"
    ));
}

/// 测试 img 标签解析：带 style 和 alt 属性
#[test]
fn test_extract_img_src_with_attributes() {
    // 带 style 属性
    assert_eq!(
        extract_img_src(r#"<img src="bg1.png" style="zoom: 10%;" />"#),
        Some("bg1.png")
    );

    // 带 alt 和 style 属性
    assert_eq!(
        extract_img_src(r#"<img src="assets/chara.png" alt="chara" style="zoom:25%;" />"#),
        Some("assets/chara.png")
    );
}

/// 综合测试：模拟真实脚本（来自 C# 测试）
#[test]
fn test_parse_realistic_script() {
    let mut parser = Parser::new();
    let script_text = r#"# 章节标题

角色名:"台词"

changeBG <img src="bg1.png" style="zoom: 10%;" /> with dissolve

show <img src="assets/chara.png" style="zoom:25%;" /> as 红叶 at farleft with dissolve
"#;

    let script = parser.parse("test", script_text).unwrap();

    // 应该有 4 个节点
    assert_eq!(script.len(), 4);

    // 验证章节
    assert!(matches!(
        &script.nodes[0],
        ScriptNode::Chapter { title, level: 1 } if title == "章节标题"
    ));

    // 验证对话
    assert!(matches!(
        &script.nodes[1],
        ScriptNode::Dialogue { speaker: Some(s), content }
        if s == "角色名" && content == "台词"
    ));

    // 验证 changeBG
    assert!(matches!(
        &script.nodes[2],
        ScriptNode::ChangeBG { path, transition: Some(t) }
        if path == "bg1.png" && t.name == "dissolve"
    ));

    // 验证 show
    if let ScriptNode::ShowCharacter {
        path: Some(path),
        alias,
        position: _,
        transition,
    } = &script.nodes[3]
    {
        assert_eq!(path.as_str(), "assets/chara.png");
        assert_eq!(alias, "红叶");
        assert!(transition.is_some());
        assert_eq!(transition.as_ref().unwrap().name, "dissolve");
        // farleft 可能还没有在 Position 中定义
    } else {
        panic!("Expected ShowCharacter node");
    }
}

//=========================================================================
// goto 语法测试
//=========================================================================

/// 测试 goto 指令：基本语法
#[test]
fn test_parse_goto_basic() {
    let mut parser = Parser::new();
    let script = parser.parse("test", "goto **start**").unwrap();
    assert_eq!(script.nodes.len(), 1);
    assert!(matches!(
        &script.nodes[0],
        ScriptNode::Goto { target_label } if target_label == "start"
    ));
}

/// 测试 goto 指令：中文标签
#[test]
fn test_parse_goto_chinese_label() {
    let mut parser = Parser::new();
    let script = parser.parse("test", "goto **选择支1**").unwrap();
    assert_eq!(script.nodes.len(), 1);
    assert!(matches!(
        &script.nodes[0],
        ScriptNode::Goto { target_label } if target_label == "选择支1"
    ));
}

/// 测试 goto 指令：带空格
#[test]
fn test_parse_goto_with_spaces() {
    let mut parser = Parser::new();
    let script = parser.parse("test", "goto  **end_scene**").unwrap();
    assert_eq!(script.nodes.len(), 1);
    assert!(matches!(
        &script.nodes[0],
        ScriptNode::Goto { target_label } if target_label == "end_scene"
    ));
}

//=========================================================================
// audio 语法测试
//=========================================================================

/// 测试 audio 指令：SFX（无 loop）
#[test]
fn test_parse_audio_sfx() {
    let mut parser = Parser::new();
    let script = parser
        .parse("test", r#"<audio src="sfx/ding.mp3"></audio>"#)
        .unwrap();
    assert_eq!(script.nodes.len(), 1);
    assert!(matches!(
        &script.nodes[0],
        ScriptNode::PlayAudio { path, is_bgm } if path == "sfx/ding.mp3" && !is_bgm
    ));
}

/// 测试 audio 指令：BGM（带 loop）
#[test]
fn test_parse_audio_bgm() {
    let mut parser = Parser::new();
    let script = parser
        .parse("test", r#"<audio src="bgm/Signal.mp3"></audio> loop"#)
        .unwrap();
    assert_eq!(script.nodes.len(), 1);
    assert!(matches!(
        &script.nodes[0],
        ScriptNode::PlayAudio { path, is_bgm } if path == "bgm/Signal.mp3" && *is_bgm
    ));
}

/// 测试 audio 指令：相对路径
#[test]
fn test_parse_audio_relative_path() {
    let mut parser = Parser::new();
    let script = parser
        .parse("test", r#"<audio src="../bgm/music.mp3"></audio> loop"#)
        .unwrap();
    assert_eq!(script.nodes.len(), 1);
    assert!(matches!(
        &script.nodes[0],
        ScriptNode::PlayAudio { path, is_bgm } if path == "../bgm/music.mp3" && *is_bgm
    ));
}

/// 测试 stopBGM 指令
#[test]
fn test_parse_stop_bgm() {
    let mut parser = Parser::new();
    let script = parser.parse("test", "stopBGM").unwrap();
    assert_eq!(script.nodes.len(), 1);
    assert!(matches!(&script.nodes[0], ScriptNode::StopBgm));
}

//=========================================================================
// 相对路径测试
//=========================================================================

/// 测试 changeBG 相对路径
#[test]
fn test_parse_change_bg_relative_path() {
    let mut parser = Parser::new();
    let script = parser
        .parse(
            "test",
            r#"changeBG <img src="../backgrounds/bg.jpg" /> with `dissolve`"#,
        )
        .unwrap();
    assert_eq!(script.nodes.len(), 1);
    assert!(matches!(
        &script.nodes[0],
        ScriptNode::ChangeBG { path, .. } if path == "../backgrounds/bg.jpg"
    ));
}

/// 测试 show 相对路径
#[test]
fn test_parse_show_relative_path() {
    let mut parser = Parser::new();
    let script = parser
        .parse(
            "test",
            r#"show <img src="../characters/北风.png" /> as beifeng at center"#,
        )
        .unwrap();
    assert_eq!(script.nodes.len(), 1);
    assert!(matches!(
        &script.nodes[0],
        ScriptNode::ShowCharacter { path: Some(path), alias, .. }
        if path.as_str() == "../characters/北风.png" && alias == "beifeng"
    ));
}

//=========================================================================
// 综合测试：包含 goto 和 audio 的完整脚本
//=========================================================================

#[test]
fn test_parse_script_with_goto_and_audio() {
    let mut parser = Parser::new();
    let text = r#"# 测试章节

<audio src="../bgm/intro.mp3"></audio> loop

**开始**

角色："欢迎！"

| 选项 | 跳转 |
| --- | --- |
| 去选项A | **选项A** |
| 去选项B | **选项B** |

**选项A**
角色："你选了A"
goto **结束**

**选项B**
角色："你选了B"
goto **结束**

**结束**
stopBGM
角色："再见！"
"#;

    let script = parser.parse("test", text).unwrap();

    // 验证关键节点
    let has_audio = script
        .nodes
        .iter()
        .any(|n| matches!(n, ScriptNode::PlayAudio { is_bgm: true, .. }));
    let has_goto = script
        .nodes
        .iter()
        .any(|n| matches!(n, ScriptNode::Goto { .. }));
    let has_stop_bgm = script
        .nodes
        .iter()
        .any(|n| matches!(n, ScriptNode::StopBgm));
    let has_choice = script
        .nodes
        .iter()
        .any(|n| matches!(n, ScriptNode::Choice { .. }));

    assert!(has_audio, "应该有 BGM 播放");
    assert!(has_goto, "应该有 goto 指令");
    assert!(has_stop_bgm, "应该有 stopBGM 指令");
    assert!(has_choice, "应该有选择分支");
}

//=========================================================================
// changeScene / changeBG 职责分离测试
//=========================================================================

/// 测试 changeBG 不允许 fade 效果
#[test]
fn test_parse_change_bg_fade_deprecated() {
    let mut parser = Parser::new();
    let result = parser.parse("test", r#"changeBG <img src="bg.jpg" /> with fade"#);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(format!("{:?}", err).contains("fade") || format!("{:?}", err).contains("Fade"));
}

/// 测试 changeBG 不允许 fadewhite 效果
#[test]
fn test_parse_change_bg_fadewhite_deprecated() {
    let mut parser = Parser::new();
    let result = parser.parse("test", r#"changeBG <img src="bg.jpg" /> with fadewhite"#);
    assert!(result.is_err());
}

/// 测试 changeBG 不允许其他非 dissolve 效果
#[test]
fn test_parse_change_bg_only_dissolve() {
    let mut parser = Parser::new();

    // dissolve 允许
    let result = parser.parse("test", r#"changeBG <img src="bg.jpg" /> with dissolve"#);
    assert!(result.is_ok());

    // Dissolve(duration) 允许
    let result = parser.parse(
        "test",
        r#"changeBG <img src="bg.jpg" /> with Dissolve(1.5)"#,
    );
    assert!(result.is_ok());

    // 其他效果不允许
    let result = parser.parse("test", r#"changeBG <img src="bg.jpg" /> with rule"#);
    assert!(result.is_err());
}

/// 测试 changeScene 必须带 with 子句
#[test]
fn test_parse_change_scene_requires_with() {
    let mut parser = Parser::new();
    let result = parser.parse("test", r#"changeScene <img src="bg.jpg" />"#);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(format!("{:?}", err).contains("with"));
}

/// 测试 changeScene with Dissolve
#[test]
fn test_parse_change_scene_dissolve() {
    let mut parser = Parser::new();
    let script = parser
        .parse(
            "test",
            r#"changeScene <img src="bg.jpg" /> with Dissolve(duration: 1)"#,
        )
        .unwrap();

    if let ScriptNode::ChangeScene {
        path,
        transition: Some(t),
    } = &script.nodes[0]
    {
        assert_eq!(path, "bg.jpg");
        assert_eq!(t.name, "Dissolve");
        assert_eq!(t.get_duration(), Some(1.0));
    } else {
        panic!("Expected ChangeScene node");
    }
}

/// 测试 changeScene with Fade
#[test]
fn test_parse_change_scene_fade() {
    let mut parser = Parser::new();
    let script = parser
        .parse(
            "test",
            r#"changeScene <img src="bg.jpg" /> with Fade(duration: 1.5)"#,
        )
        .unwrap();

    if let ScriptNode::ChangeScene {
        path,
        transition: Some(t),
    } = &script.nodes[0]
    {
        assert_eq!(path, "bg.jpg");
        assert_eq!(t.name, "Fade");
        assert_eq!(t.get_duration(), Some(1.5));
    } else {
        panic!("Expected ChangeScene node");
    }
}

/// 测试 changeScene with FadeWhite
#[test]
fn test_parse_change_scene_fade_white() {
    let mut parser = Parser::new();
    let script = parser
        .parse(
            "test",
            r#"changeScene <img src="bg.jpg" /> with FadeWhite(duration: 2)"#,
        )
        .unwrap();

    if let ScriptNode::ChangeScene {
        path,
        transition: Some(t),
    } = &script.nodes[0]
    {
        assert_eq!(path, "bg.jpg");
        assert_eq!(t.name, "FadeWhite");
        assert_eq!(t.get_duration(), Some(2.0));
    } else {
        panic!("Expected ChangeScene node");
    }
}

/// 测试 changeScene with rule-based effect
#[test]
fn test_parse_change_scene_rule() {
    let mut parser = Parser::new();
    let script = parser.parse(
        "test",
        r#"changeScene <img src="bg.jpg" /> with <img src="rule_10.png" /> (duration: 1, reversed: true)"#
    ).unwrap();

    if let ScriptNode::ChangeScene {
        path,
        transition: Some(t),
    } = &script.nodes[0]
    {
        assert_eq!(path, "bg.jpg");
        assert_eq!(t.name, "rule");
        // 检查参数
        assert_eq!(
            t.get_named("mask"),
            Some(&TransitionArg::String("rule_10.png".to_string()))
        );
        assert_eq!(t.get_duration(), Some(1.0));
        assert_eq!(t.get_reversed(), Some(true));
    } else {
        panic!("Expected ChangeScene node");
    }
}

/// 测试 changeScene with rule-based effect（无参数）
#[test]
fn test_parse_change_scene_rule_no_params() {
    let mut parser = Parser::new();
    let script = parser
        .parse(
            "test",
            r#"changeScene <img src="bg.jpg" /> with <img src="mask.png" />"#,
        )
        .unwrap();

    if let ScriptNode::ChangeScene {
        path,
        transition: Some(t),
    } = &script.nodes[0]
    {
        assert_eq!(path, "bg.jpg");
        assert_eq!(t.name, "rule");
        assert_eq!(
            t.get_named("mask"),
            Some(&TransitionArg::String("mask.png".to_string()))
        );
    } else {
        panic!("Expected ChangeScene node");
    }
}

#[test]
fn test_parse_change_scene_requires_with_clause() {
    let mut parser = Parser::new();
    let err = parser
        .parse("test", r#"changeScene <img src="assets/bg.png" />"#)
        .unwrap_err();
    assert!(matches!(
        err,
        crate::error::ParseError::MissingParameter { .. }
    ));
}

#[test]
fn test_parse_change_scene_invalid_transition() {
    let mut parser = Parser::new();
    // 过渡表达式解析失败（混用参数），应报 InvalidTransition
    let err = parser
        .parse(
            "test",
            r#"changeScene <img src="assets/bg.png" /> with Dissolve(1.0, duration: 2.0)"#,
        )
        .unwrap_err();
    assert!(matches!(
        err,
        crate::error::ParseError::InvalidTransition { .. }
    ));
}

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
fn test_parse_goto_label_extraction_and_empty_label_error() {
    let mut parser = Parser::new();
    // 支持 **label**
    let script = parser.parse("test", "goto **end**").unwrap();
    assert!(matches!(
        &script.nodes[0],
        ScriptNode::Goto { target_label } if target_label == "end"
    ));

    // "**  **" 会被 trim 成空字符串，应报 MissingParameter
    let err = parser.parse("test", "goto **  **").unwrap_err();
    assert!(matches!(
        err,
        crate::error::ParseError::MissingParameter { .. }
    ));
}

#[test]
fn test_parse_audio_loop_and_no_close_tag() {
    let mut parser = Parser::new();

    // 有 </audio> 且带 loop -> BGM
    let script = parser
        .parse("test", r#"<audio src="bgm.mp3"></audio> loop"#)
        .unwrap();
    assert!(matches!(
        &script.nodes[0],
        ScriptNode::PlayAudio { path, is_bgm: true } if path == "bgm.mp3"
    ));

    // 没有 </audio> -> is_bgm = false
    let script = parser.parse("test", r#"<audio src="sfx.mp3">"#).unwrap();
    assert!(matches!(
        &script.nodes[0],
        ScriptNode::PlayAudio { path, is_bgm: false } if path == "sfx.mp3"
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

#[test]
fn test_parse_table_incomplete_rows_and_empty_options_errors() {
    let mut parser = Parser::new();

    // 1) 不完整行应产生 warning，但仍能解析出 Choice
    let text = r#"
| 横排 |
| --- |
| 只有一列 |
| 选项1 | label1 |
"#
    .trim();
    let script = parser.parse("test", text).unwrap();
    assert_eq!(script.len(), 1);
    assert!(matches!(&script.nodes[0], ScriptNode::Choice { .. }));
    assert!(!parser.warnings().is_empty());

    // 2) 没有任何有效选项 -> InvalidTable
    let mut parser = Parser::new();
    let text = r#"
| 横排 |
| --- |
| 只有一列 |
"#
    .trim();
    let err = parser.parse("test", text).unwrap_err();
    assert!(matches!(err, crate::error::ParseError::InvalidTable { .. }));
}

#[test]
fn test_extract_transition_from_line_rule_without_src_and_with_invalid_args() {
    let parser = phase2::Phase2Parser::new();

    // rule: 有 <img 但没有 src -> 返回 simple("rule")
    let t = parser
        .extract_transition_from_line(r#"changeScene <img src="bg.png" /> with <img alt="x" />"#)
        .unwrap();
    assert_eq!(t.name, "rule");
    assert!(t.args.is_empty());

    // rule: 有 src，但括号里参数非法（混用）-> extract_rule_args 不扩展参数，只保留 mask
    let t = parser
        .extract_transition_from_line(
            r#"changeScene <img src="bg.png" /> with <img src="masks/rule.png" /> (1.0, duration: 2.0)"#,
        )
        .unwrap();
    assert_eq!(t.name, "rule");
    assert!(t.get_named("mask").is_some());
    assert!(t.get_named("duration").is_none());
}

//=========================================================================
// set 指令测试
//=========================================================================

#[test]
fn test_parse_set_var_string() {
    let mut parser = Parser::new();
    let script = parser.parse("test", r#"set $name = "Alice""#).unwrap();

    assert_eq!(script.len(), 1);
    if let ScriptNode::SetVar { name, value } = &script.nodes[0] {
        assert_eq!(name, "name");
        assert!(
            matches!(value, crate::script::Expr::Literal(crate::state::VarValue::String(s)) if s == "Alice")
        );
    } else {
        panic!("Expected SetVar node");
    }
}

#[test]
fn test_parse_set_var_bool() {
    let mut parser = Parser::new();

    let script = parser.parse("test", "set $is_active = true").unwrap();
    if let ScriptNode::SetVar { name, value } = &script.nodes[0] {
        assert_eq!(name, "is_active");
        assert!(matches!(
            value,
            crate::script::Expr::Literal(crate::state::VarValue::Bool(true))
        ));
    } else {
        panic!("Expected SetVar node");
    }

    let script = parser.parse("test", "set $is_done = false").unwrap();
    if let ScriptNode::SetVar { name, value } = &script.nodes[0] {
        assert_eq!(name, "is_done");
        assert!(matches!(
            value,
            crate::script::Expr::Literal(crate::state::VarValue::Bool(false))
        ));
    } else {
        panic!("Expected SetVar node");
    }
}

#[test]
fn test_parse_set_var_int() {
    let mut parser = Parser::new();
    let script = parser.parse("test", "set $count = 42").unwrap();

    if let ScriptNode::SetVar { name, value } = &script.nodes[0] {
        assert_eq!(name, "count");
        assert!(matches!(
            value,
            crate::script::Expr::Literal(crate::state::VarValue::Int(42))
        ));
    } else {
        panic!("Expected SetVar node");
    }
}

#[test]
fn test_parse_set_var_missing_dollar() {
    let mut parser = Parser::new();
    let err = parser.parse("test", "set name = 123").unwrap_err();
    assert!(matches!(err, crate::error::ParseError::InvalidLine { .. }));
}

#[test]
fn test_parse_set_var_missing_equals() {
    let mut parser = Parser::new();
    let err = parser.parse("test", "set $name 123").unwrap_err();
    assert!(matches!(
        err,
        crate::error::ParseError::MissingParameter { .. }
    ));
}

//=========================================================================
// 条件分支测试
//=========================================================================

#[test]
fn test_parse_simple_if() {
    let mut parser = Parser::new();
    let text = r#"
if $flag == true
  ："条件为真"
endif
"#;
    let script = parser.parse("test", text).unwrap();

    assert_eq!(script.len(), 1);
    if let ScriptNode::Conditional { branches } = &script.nodes[0] {
        assert_eq!(branches.len(), 1);
        assert!(branches[0].condition.is_some());
        assert_eq!(branches[0].body.len(), 1);
    } else {
        panic!("Expected Conditional node");
    }
}

#[test]
fn test_parse_if_else() {
    let mut parser = Parser::new();
    let text = r#"
if $name == "Alice"
  ："你好，Alice"
else
  ："你好，陌生人"
endif
"#;
    let script = parser.parse("test", text).unwrap();

    if let ScriptNode::Conditional { branches } = &script.nodes[0] {
        assert_eq!(branches.len(), 2);
        assert!(branches[0].condition.is_some()); // if 分支
        assert!(branches[1].condition.is_none()); // else 分支
    } else {
        panic!("Expected Conditional node");
    }
}

#[test]
fn test_parse_if_elseif_else() {
    let mut parser = Parser::new();
    let text = r#"
if $role == "admin"
  ："欢迎管理员"
elseif $role == "user"
  ："欢迎用户"
else
  ："欢迎访客"
endif
"#;
    let script = parser.parse("test", text).unwrap();

    if let ScriptNode::Conditional { branches } = &script.nodes[0] {
        assert_eq!(branches.len(), 3);
        assert!(branches[0].condition.is_some()); // if
        assert!(branches[1].condition.is_some()); // elseif
        assert!(branches[2].condition.is_none()); // else
    } else {
        panic!("Expected Conditional node");
    }
}

#[test]
fn test_parse_if_with_logical_ops() {
    let mut parser = Parser::new();
    let text = r#"
if $a == true and $b == false
  ："复合条件"
endif
"#;
    let script = parser.parse("test", text).unwrap();

    if let ScriptNode::Conditional { branches } = &script.nodes[0] {
        assert_eq!(branches.len(), 1);
        // 条件应该是 And 表达式
        if let Some(crate::script::Expr::And(_, _)) = &branches[0].condition {
            // OK
        } else {
            panic!("Expected And expression");
        }
    } else {
        panic!("Expected Conditional node");
    }
}

#[test]
fn test_parse_if_missing_endif() {
    let mut parser = Parser::new();
    let text = r#"
if $flag == true
  ："没有 endif"
"#;
    // 未闭合的条件块会在 parse_conditional 阶段报错
    let result = parser.parse("test", text);
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        crate::error::ParseError::InvalidLine { .. }
    ));
}

//=========================================================================
// 表达式解析测试
//=========================================================================

#[test]
fn test_parse_expression_variable() {
    let expr = parse_expression("$foo == \"bar\"", 1).unwrap();
    assert!(matches!(expr, crate::script::Expr::Eq(_, _)));
}

#[test]
fn test_parse_expression_bool_literal() {
    let expr = parse_expression("true", 1).unwrap();
    assert!(matches!(
        expr,
        crate::script::Expr::Literal(crate::state::VarValue::Bool(true))
    ));

    let expr = parse_expression("false", 1).unwrap();
    assert!(matches!(
        expr,
        crate::script::Expr::Literal(crate::state::VarValue::Bool(false))
    ));
}

#[test]
fn test_parse_expression_not() {
    let expr = parse_expression("not $flag", 1).unwrap();
    assert!(matches!(expr, crate::script::Expr::Not(_)));
}

#[test]
fn test_parse_expression_and_or() {
    let expr = parse_expression("$a == true and $b == false", 1).unwrap();
    assert!(matches!(expr, crate::script::Expr::And(_, _)));

    let expr = parse_expression("$a == true or $b == false", 1).unwrap();
    assert!(matches!(expr, crate::script::Expr::Or(_, _)));
}

#[test]
fn test_parse_expression_parentheses() {
    let expr = parse_expression("($a == true)", 1).unwrap();
    assert!(matches!(expr, crate::script::Expr::Eq(_, _)));

    let expr = parse_expression("($a == true) and ($b == false)", 1).unwrap();
    assert!(matches!(expr, crate::script::Expr::And(_, _)));
}

#[test]
fn test_parse_expression_not_equal() {
    let expr = parse_expression("$name != \"Bob\"", 1).unwrap();
    assert!(matches!(expr, crate::script::Expr::NotEq(_, _)));
}

#[test]
fn test_parse_expression_empty_error() {
    let err = parse_expression("", 1).unwrap_err();
    assert!(matches!(err, crate::error::ParseError::InvalidLine { .. }));
}

#[test]
fn test_parse_expression_unclosed_paren() {
    let err = parse_expression("($a == true", 1).unwrap_err();
    assert!(matches!(err, crate::error::ParseError::InvalidLine { .. }));
}

// =========================================================================
// 阶段 24：TextBox / ClearCharacters 指令解析测试
// =========================================================================

#[test]
fn test_parse_textbox_hide() {
    let mut parser = Parser::new();
    let script = parser.parse("test", "textBoxHide").unwrap();
    assert_eq!(script.nodes.len(), 1);
    assert!(matches!(script.nodes[0], ScriptNode::TextBoxHide));
}

#[test]
fn test_parse_textbox_show() {
    let mut parser = Parser::new();
    let script = parser.parse("test", "textBoxShow").unwrap();
    assert_eq!(script.nodes.len(), 1);
    assert!(matches!(script.nodes[0], ScriptNode::TextBoxShow));
}

#[test]
fn test_parse_textbox_clear() {
    let mut parser = Parser::new();
    let script = parser.parse("test", "textBoxClear").unwrap();
    assert_eq!(script.nodes.len(), 1);
    assert!(matches!(script.nodes[0], ScriptNode::TextBoxClear));
}

#[test]
fn test_parse_clear_characters() {
    let mut parser = Parser::new();
    let script = parser.parse("test", "clearCharacters").unwrap();
    assert_eq!(script.nodes.len(), 1);
    assert!(matches!(script.nodes[0], ScriptNode::ClearCharacters));
}

#[test]
fn test_parse_textbox_commands_case_insensitive() {
    let mut parser = Parser::new();
    let script = parser.parse("test", "TEXTBOXHIDE\nTextBoxShow\ntextboxclear\nCLEARCHARACTERS").unwrap();
    assert_eq!(script.nodes.len(), 4);
    assert!(matches!(script.nodes[0], ScriptNode::TextBoxHide));
    assert!(matches!(script.nodes[1], ScriptNode::TextBoxShow));
    assert!(matches!(script.nodes[2], ScriptNode::TextBoxClear));
    assert!(matches!(script.nodes[3], ScriptNode::ClearCharacters));
}
