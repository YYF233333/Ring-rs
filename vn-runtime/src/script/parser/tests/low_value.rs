use super::*;

#[test]
fn test_parse_full_restart() {
    let node = parse_single_node("fullRestart");
    assert_eq!(node, ScriptNode::FullRestart);

    // 大小写不敏感
    let node2 = parse_single_node("FULLRESTART");
    assert_eq!(node2, ScriptNode::FullRestart);
}

#[test]
fn test_parse_pause() {
    let node = parse_single_node("pause");
    assert_eq!(node, ScriptNode::Pause);

    let node2 = parse_single_node("Pause");
    assert_eq!(node2, ScriptNode::Pause);
}

// -------------------------------------------------------------------------
// 从 mod.rs 迁出的其余测试
// -------------------------------------------------------------------------

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

// =========================================================================
// requestUI 测试
// =========================================================================

#[test]
fn test_parse_request_ui_basic() {
    let node = parse_single_node(r#"requestUI "show_map" as $destination"#);
    assert!(matches!(
        node,
        ScriptNode::RequestUI { mode, result_var, params }
            if mode == "show_map" && result_var == "destination" && params.is_empty()
    ));
}

#[test]
fn test_parse_request_ui_with_params() {
    let script = parse_ok(r#"requestUI "show_map" as $choice (map_id: "world", zoom: 2)"#);
    assert_eq!(script.nodes.len(), 1);
    match &script.nodes[0] {
        ScriptNode::RequestUI {
            mode,
            result_var,
            params,
        } => {
            assert_eq!(mode, "show_map");
            assert_eq!(result_var, "choice");
            assert_eq!(params.len(), 2);
            assert_eq!(params[0].0, "map_id");
            assert_eq!(params[1].0, "zoom");
        }
        other => panic!("expected RequestUI, got {:?}", other),
    }
}

#[test]
fn test_parse_request_ui_case_insensitive() {
    let node = parse_single_node(r#"requestui "test" as $result"#);
    assert!(matches!(
        node,
        ScriptNode::RequestUI { mode, .. } if mode == "test"
    ));
}

#[test]
fn test_parse_request_ui_missing_mode() {
    let _err = parse_err("requestUI");
}

#[test]
fn test_parse_request_ui_missing_as_clause() {
    let _err = parse_err(r#"requestUI "mode""#);
}

#[test]
fn test_parse_request_ui_missing_dollar_sign() {
    let _err = parse_err(r#"requestUI "mode" as result"#);
}

// =========================================================================
// textMode 测试
// =========================================================================

#[test]
fn test_parse_text_mode_nvl() {
    let script = parse_ok("textMode nvl");
    assert_eq!(script.len(), 1);
    assert!(matches!(
        script.nodes[0],
        ScriptNode::SetTextMode(TextMode::NVL)
    ));
}

#[test]
fn test_parse_text_mode_adv() {
    let script = parse_ok("textMode adv");
    assert_eq!(script.len(), 1);
    assert!(matches!(
        script.nodes[0],
        ScriptNode::SetTextMode(TextMode::ADV)
    ));
}

#[test]
fn test_parse_text_mode_case_insensitive() {
    let script = parse_ok("textmode NVL");
    assert_eq!(script.len(), 1);
    assert!(matches!(
        script.nodes[0],
        ScriptNode::SetTextMode(TextMode::NVL)
    ));
}

#[test]
fn test_parse_text_mode_missing_mode() {
    parse_err("textMode");
}

#[test]
fn test_parse_text_mode_invalid_mode() {
    parse_err("textMode foo");
}

#[test]
fn test_parse_show_map_basic() {
    let script = parse_ok("showMap \"world\" as $destination");
    assert_eq!(script.nodes.len(), 1);
    match &script.nodes[0] {
        ScriptNode::RequestUI {
            mode,
            result_var,
            params,
        } => {
            assert_eq!(mode, "show_map");
            assert_eq!(result_var, "destination");
            assert_eq!(params.len(), 1);
            assert_eq!(params[0].0, "map_id");
        }
        other => panic!("expected RequestUI, got {:?}", other),
    }
}

#[test]
fn test_parse_show_map_case_insensitive() {
    let script = parse_ok("showmap \"city\" as $choice");
    assert_eq!(script.nodes.len(), 1);
    assert!(matches!(&script.nodes[0], ScriptNode::RequestUI { mode, .. } if mode == "show_map"));
}

#[test]
fn test_parse_show_map_missing_map_id() {
    parse_err("showMap");
}

#[test]
fn test_parse_show_map_missing_as_clause() {
    parse_err("showMap \"world\"");
}

#[test]
fn test_parse_call_game_basic() {
    let script = parse_ok("callGame \"pong\" as $score");
    assert_eq!(script.nodes.len(), 1);
    match &script.nodes[0] {
        ScriptNode::RequestUI {
            mode,
            result_var,
            params,
        } => {
            assert_eq!(mode, "call_game");
            assert_eq!(result_var, "score");
            assert!(params.iter().any(|(k, _)| k == "game_id"));
        }
        other => panic!("expected RequestUI, got {:?}", other),
    }
}

#[test]
fn test_parse_call_game_with_params() {
    let script = parse_ok("callGame \"cards\" as $result (difficulty: 3)");
    assert_eq!(script.nodes.len(), 1);
    match &script.nodes[0] {
        ScriptNode::RequestUI {
            mode,
            result_var,
            params,
        } => {
            assert_eq!(mode, "call_game");
            assert_eq!(result_var, "result");
            assert!(params.len() >= 2);
        }
        other => panic!("expected RequestUI, got {:?}", other),
    }
}

#[test]
fn test_parse_call_game_missing_id() {
    parse_err("callGame");
}

#[test]
fn test_parse_call_game_missing_as() {
    parse_err("callGame \"pong\"");
}
