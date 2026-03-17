use super::*;

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
fn test_extract_audio_src_supports_single_quoted_value() {
    assert_eq!(
        extract_audio_src(r#"<audio src='voice.ogg'></audio>"#),
        Some("voice.ogg")
    );
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
fn test_parse_arg_value_unterminated_quotes_are_plain_strings() {
    assert_eq!(
        parse_arg_value("\"unterminated"),
        TransitionArg::String("\"unterminated".to_string())
    );
    assert_eq!(parse_arg_value("'"), TransitionArg::String("'".to_string()));
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
fn test_is_table_separator_requires_both_edge_pipes() {
    assert!(!is_table_separator("| --- | --- "));
    assert!(!is_table_separator(" --- | --- |"));
}

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

#[test]
fn test_parse_simple_commands_case_insensitive() {
    let cases = [
        ("STOPBGM", ScriptNode::StopBgm),
        ("StopBGM", ScriptNode::StopBgm),
        ("RETURNFROMSCRIPT", ScriptNode::ReturnFromScript),
        ("ReturnFromScript", ScriptNode::ReturnFromScript),
        ("PAUSE", ScriptNode::Pause),
    ];
    for (input, expected) in cases {
        let node = parse_single_node(input);
        assert_eq!(node, expected, "input={input}");
    }
}
