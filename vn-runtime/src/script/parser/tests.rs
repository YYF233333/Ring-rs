//! # Parser 测试

use super::*;
use crate::command::{Position, TransitionArg};
use crate::script::ast::ScriptNode;

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
fn test_parse_goto_cross_file_is_banned() {
    let err = parse_err("goto **summer::start**");
    assert!(matches!(err, crate::error::ParseError::InvalidLine { .. }));
}

#[test]
fn test_parse_call_script_and_return_from_script() {
    let node = parse_single_node(r#"callScript [prologue](ring/summer/prologue.md)"#);
    assert!(matches!(
        node,
        ScriptNode::CallScript { path, display_label: Some(label) }
            if path == "ring/summer/prologue.md" && label == "prologue"
    ));

    let node = parse_single_node(r#"callScript [chapter1](ring/summer/1-1.md)"#);
    assert!(matches!(
        node,
        ScriptNode::CallScript { path, display_label: Some(label) }
            if path == "ring/summer/1-1.md" && label == "chapter1"
    ));

    let node = parse_single_node("returnFromScript");
    assert!(matches!(node, ScriptNode::ReturnFromScript));
}

#[test]
fn test_parse_call_script_requires_quoted_path() {
    let err = parse_err("callScript ring/summer/prologue.md");
    assert!(matches!(err, crate::error::ParseError::InvalidLine { .. }));

    let err = parse_err(r#"callScript "ring/summer/prologue.md""#);
    assert!(matches!(err, crate::error::ParseError::InvalidLine { .. }));
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

//=========================================================================
// 相对路径测试
//=========================================================================

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

/// 测试 changeBG 过渡策略：
/// - 允许：`dissolve`、`Dissolve(1.5)`
/// - 禁止：`fade`、`fadewhite`、`rule`
#[test]
fn test_parse_change_bg_transition_policy() {
    let ok_cases = [
        r#"changeBG <img src="bg.jpg" /> with dissolve"#,
        r#"changeBG <img src="bg.jpg" /> with Dissolve(1.5)"#,
    ];
    for input in ok_cases {
        let script = parse_ok(input);
        assert_eq!(script.nodes.len(), 1, "input={input}");
    }

    let err_cases = [
        (r#"changeBG <img src="bg.jpg" /> with fade"#, true),
        (r#"changeBG <img src="bg.jpg" /> with fadewhite"#, false),
        (r#"changeBG <img src="bg.jpg" /> with rule"#, false),
    ];
    for (input, check_fade_hint) in err_cases {
        let err = parse_err(input);
        if check_fade_hint {
            let err_text = format!("{err:?}");
            assert!(err_text.contains("fade") || err_text.contains("Fade"));
        }
    }
}

/// 测试 changeScene 标准过渡：
/// - `Dissolve(duration: 1)`
/// - `Fade(duration: 1.5)`
/// - `FadeWhite(duration: 2)`
#[test]
fn test_parse_change_scene_standard_transitions() {
    let ok_cases = [
        (
            r#"changeScene <img src="bg.jpg" /> with Dissolve(duration: 1)"#,
            "Dissolve",
            Some(1.0),
        ),
        (
            r#"changeScene <img src="bg.jpg" /> with Fade(duration: 1.5)"#,
            "Fade",
            Some(1.5),
        ),
        (
            r#"changeScene <img src="bg.jpg" /> with FadeWhite(duration: 2)"#,
            "FadeWhite",
            Some(2.0),
        ),
    ];

    for (input, expected_transition_name, expected_duration) in ok_cases {
        let node = parse_single_node(input);
        if let ScriptNode::ChangeScene {
            path,
            transition: Some(t),
        } = node
        {
            assert_eq!(path, "bg.jpg", "input={input}");
            assert_eq!(t.name, expected_transition_name, "input={input}");
            assert_eq!(t.get_duration(), expected_duration, "input={input}");
        } else {
            panic!("Expected ChangeScene node for input={input}");
        }
    }
}

/// 测试 changeScene rule 过渡：
/// - 带参数：`mask + duration + reversed`
/// - 无参数：仅 `mask`
#[test]
fn test_parse_change_scene_rule_transition() {
    let cases = [
        (
            r#"changeScene <img src="bg.jpg" /> with <img src="rule_10.png" /> (duration: 1, reversed: true)"#,
            "rule_10.png",
            Some(1.0),
            Some(true),
        ),
        (
            r#"changeScene <img src="bg.jpg" /> with <img src="mask.png" />"#,
            "mask.png",
            None,
            None,
        ),
    ];

    for (input, expected_mask, expected_duration, expected_reversed) in cases {
        let node = parse_single_node(input);
        if let ScriptNode::ChangeScene {
            path,
            transition: Some(t),
        } = node
        {
            assert_eq!(path, "bg.jpg", "input={input}");
            assert_eq!(t.name, "rule", "input={input}");
            assert_eq!(
                t.get_named("mask"),
                Some(&TransitionArg::String(expected_mask.to_string())),
                "input={input}"
            );
            assert_eq!(t.get_duration(), expected_duration, "input={input}");
            assert_eq!(t.get_reversed(), expected_reversed, "input={input}");
        } else {
            panic!("Expected ChangeScene node for input={input}");
        }
    }
}

/// 测试 changeScene 缺失 with 子句时报错：
/// - `bg.jpg` 路径
/// - `assets/bg.png` 路径
#[test]
fn test_parse_change_scene_requires_with_clause() {
    let cases = [
        r#"changeScene <img src="bg.jpg" />"#,
        r#"changeScene <img src="assets/bg.png" />"#,
    ];

    for input in cases {
        let err = parse_err(input);
        assert!(
            matches!(err, crate::error::ParseError::MissingParameter { .. }),
            "input={input}"
        );
        assert!(format!("{err:?}").contains("with"), "input={input}");
    }
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

/// 测试 set 指令：
/// - 正常分支：String/Bool(true)/Bool(false)/Int
/// - 错误分支：缺少 `$`、缺少 `=`
#[test]
fn test_parse_set_var() {
    enum ExpectedValue {
        String(&'static str),
        Bool(bool),
        Int(i64),
    }

    let ok_cases = [
        (
            r#"set $name = "Alice""#,
            "name",
            ExpectedValue::String("Alice"),
        ),
        (
            "set $is_active = true",
            "is_active",
            ExpectedValue::Bool(true),
        ),
        (
            "set $is_done = false",
            "is_done",
            ExpectedValue::Bool(false),
        ),
        ("set $count = 42", "count", ExpectedValue::Int(42)),
    ];

    for (input, expected_name, expected_value) in ok_cases {
        let node = parse_single_node(input);
        if let ScriptNode::SetVar { name, value } = node {
            assert_eq!(name, expected_name, "input={input}");
            match expected_value {
                ExpectedValue::String(s) => {
                    assert!(
                        matches!(value, crate::script::Expr::Literal(crate::state::VarValue::String(actual)) if actual == s),
                        "input={input}"
                    );
                }
                ExpectedValue::Bool(b) => {
                    assert!(
                        matches!(value, crate::script::Expr::Literal(crate::state::VarValue::Bool(actual)) if actual == b),
                        "input={input}"
                    );
                }
                ExpectedValue::Int(i) => {
                    assert!(
                        matches!(value, crate::script::Expr::Literal(crate::state::VarValue::Int(actual)) if actual == i),
                        "input={input}"
                    );
                }
            }
        } else {
            panic!("Expected SetVar node for input={input}");
        }
    }

    let err_cases = [
        ("set name = 123", "invalid-line"),
        ("set $name 123", "missing-parameter"),
    ];

    for (input, expected_kind) in err_cases {
        let err = parse_err(input);
        match expected_kind {
            "invalid-line" => {
                assert!(matches!(err, crate::error::ParseError::InvalidLine { .. }));
            }
            "missing-parameter" => {
                assert!(matches!(
                    err,
                    crate::error::ParseError::MissingParameter { .. }
                ));
            }
            _ => unreachable!(),
        }
    }
}

#[test]
fn test_parse_set_persistent_var() {
    // 正常用例：$persistent.key = value
    let node = parse_single_node("set $persistent.complete_summer = true");
    if let ScriptNode::SetVar { name, value } = node {
        assert_eq!(name, "persistent.complete_summer");
        assert!(
            matches!(
                value,
                crate::script::Expr::Literal(crate::state::VarValue::Bool(true))
            ),
            "expected Bool(true)"
        );
    } else {
        panic!("Expected SetVar node");
    }

    // 持久变量名中含有数字和下划线也应通过
    let node2 = parse_single_node(r#"set $persistent.my_var_2 = "hello""#);
    if let ScriptNode::SetVar { name, .. } = node2 {
        assert_eq!(name, "persistent.my_var_2");
    } else {
        panic!("Expected SetVar node");
    }

    // 格式错误：persistent. 后无 key 名
    let err1 = parse_err("set $persistent. = true");
    assert!(matches!(err1, crate::error::ParseError::InvalidLine { .. }));

    // 格式错误：persistent. 后含点号
    let err2 = parse_err("set $persistent.a.b = true");
    assert!(matches!(err2, crate::error::ParseError::InvalidLine { .. }));
}

#[test]
fn test_parse_full_restart() {
    let node = parse_single_node("fullRestart");
    assert_eq!(node, ScriptNode::FullRestart);

    // 大小写不敏感
    let node2 = parse_single_node("FULLRESTART");
    assert_eq!(node2, ScriptNode::FullRestart);
}

//=========================================================================
// 条件分支测试
//=========================================================================

/// 测试条件分支解析：
/// - `if ... endif`（单分支）
/// - `if/else`
/// - `if/elseif/else`
/// - 复合逻辑条件（And）
/// - 缺失 `endif` 报错
#[test]
fn test_parse_conditionals() {
    let ok_cases = [
        (
            r#"
if $flag == true
  ："条件为真"
endif
"#,
            1usize,
            false,
        ),
        (
            r#"
if $name == "Alice"
  ："你好，Alice"
else
  ："你好，陌生人"
endif
"#,
            2usize,
            false,
        ),
        (
            r#"
if $role == "admin"
  ："欢迎管理员"
elseif $role == "user"
  ："欢迎用户"
else
  ："欢迎访客"
endif
"#,
            3usize,
            false,
        ),
        (
            r#"
if $a == true and $b == false
  ："复合条件"
endif
"#,
            1usize,
            true,
        ),
    ];

    for (input, expected_branches, expect_and_condition) in ok_cases {
        let node = parse_single_node(input);
        if let ScriptNode::Conditional { branches } = node {
            assert_eq!(branches.len(), expected_branches, "input={input}");
            if expected_branches >= 2 {
                assert!(branches[0].condition.is_some(), "input={input}");
                assert!(
                    branches[expected_branches - 1].condition.is_none(),
                    "input={input}"
                );
            } else {
                assert!(branches[0].condition.is_some(), "input={input}");
                assert_eq!(branches[0].body.len(), 1, "input={input}");
            }

            if expect_and_condition {
                assert!(
                    matches!(branches[0].condition, Some(crate::script::Expr::And(_, _))),
                    "input={input}"
                );
            }
        } else {
            panic!("Expected Conditional node for input={input}");
        }
    }

    let err = parse_err(
        r#"
if $flag == true
  ："没有 endif"
"#,
    );
    assert!(matches!(err, crate::error::ParseError::InvalidLine { .. }));
}

//=========================================================================
// 表达式解析测试
//=========================================================================

/// 测试表达式解析：
/// - 比较：`==` / `!=`
/// - 字面量：`true` / `false`
/// - 逻辑：`not` / `and` / `or`
/// - 括号表达式
/// - 错误：空表达式、右括号缺失
#[test]
fn test_parse_expression() {
    let ok_cases = [
        ("$foo == \"bar\"", "eq"),
        ("true", "bool-true"),
        ("false", "bool-false"),
        ("not $flag", "not"),
        ("$a == true and $b == false", "and"),
        ("$a == true or $b == false", "or"),
        ("($a == true)", "eq"),
        ("($a == true) and ($b == false)", "and"),
        ("$name != \"Bob\"", "not-eq"),
        // 持久变量命名空间在表达式中正常解析
        ("$persistent.complete_summer != true", "not-eq"),
        ("$persistent.complete_summer == true", "eq"),
        ("not $persistent.complete_summer", "not"),
    ];
    for (input, expected_kind) in ok_cases {
        let expr = parse_expression(input, 1).unwrap();
        match expected_kind {
            "eq" => assert!(
                matches!(expr, crate::script::Expr::Eq(_, _)),
                "input={input}"
            ),
            "bool-true" => assert!(
                matches!(
                    expr,
                    crate::script::Expr::Literal(crate::state::VarValue::Bool(true))
                ),
                "input={input}"
            ),
            "bool-false" => assert!(
                matches!(
                    expr,
                    crate::script::Expr::Literal(crate::state::VarValue::Bool(false))
                ),
                "input={input}"
            ),
            "not" => assert!(matches!(expr, crate::script::Expr::Not(_)), "input={input}"),
            "and" => assert!(
                matches!(expr, crate::script::Expr::And(_, _)),
                "input={input}"
            ),
            "or" => assert!(
                matches!(expr, crate::script::Expr::Or(_, _)),
                "input={input}"
            ),
            "not-eq" => assert!(
                matches!(expr, crate::script::Expr::NotEq(_, _)),
                "input={input}"
            ),
            _ => unreachable!(),
        }
    }

    let err_cases = ["", "($a == true"];
    for input in err_cases {
        let err = parse_expression(input, 1).unwrap_err();
        assert!(matches!(err, crate::error::ParseError::InvalidLine { .. }));
    }
}

// =========================================================================
// 阶段 24：TextBox / ClearCharacters 指令解析测试
// =========================================================================

/// 测试 TextBox/ClearCharacters 指令：
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
fn test_parse_wait() {
    let node = parse_single_node("wait 1.0");
    assert_eq!(node, ScriptNode::Wait { duration: 1.0 });

    let node2 = parse_single_node("wait 0.5");
    assert_eq!(node2, ScriptNode::Wait { duration: 0.5 });

    let node3 = parse_single_node("Wait 2");
    assert_eq!(node3, ScriptNode::Wait { duration: 2.0 });

    // 缺少参数
    let err = parse_err("wait");
    assert!(
        format!("{:?}", err).contains("MissingParameter"),
        "expected MissingParameter, got: {:?}",
        err
    );

    // 非数字
    let err2 = parse_err("wait abc");
    assert!(
        format!("{:?}", err2).contains("InvalidParameter"),
        "expected InvalidParameter, got: {:?}",
        err2
    );

    // 负数
    let err3 = parse_err("wait -1");
    assert!(
        format!("{:?}", err3).contains("InvalidParameter"),
        "expected InvalidParameter for negative, got: {:?}",
        err3
    );
}

#[test]
fn test_parse_pause() {
    let node = parse_single_node("pause");
    assert_eq!(node, ScriptNode::Pause);

    let node2 = parse_single_node("Pause");
    assert_eq!(node2, ScriptNode::Pause);
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
