use super::*;

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
fn test_parse_transition_args_preserve_quoted_commas_and_colons() {
    let args = parse_transition_args(r#"mask: "rule,a.png", easing: 'ease:in'"#).unwrap();

    assert_eq!(
        args,
        vec![
            (
                Some("mask".to_string()),
                TransitionArg::String("rule,a.png".to_string()),
            ),
            (
                Some("easing".to_string()),
                TransitionArg::String("ease:in".to_string()),
            ),
        ]
    );
}

#[test]
fn test_parse_transition_args_invalid_identifier_causes_error() {
    let err = parse_transition_args("duration: 1.0, 1invalid: 2.0").unwrap_err();
    assert!(err.contains("不允许混用"));

    let err = parse_transition_args("duration: 1.0, bad-key: 2.0").unwrap_err();
    assert!(err.contains("不允许混用"));
}

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
        ScriptNode::Dialogue { speaker: Some(s), content, .. }
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
