use super::*;

#[test]
fn test_analyze_script_undefined_label() {
    let mut parser = Parser::new();
    let text = r#"
**start**
角色："对话"
goto **nonexistent**
"#;

    let script = parser.parse("test", text).unwrap();
    let result = analyze_script(&script);

    assert!(result.has_errors());
    assert_eq!(result.error_count(), 1);
    assert!(result.diagnostics[0].message.contains("nonexistent"));
}

#[test]
fn test_analyze_script_valid_labels() {
    let mut parser = Parser::new();
    let text = r#"
**start**
角色："对话"
goto **end**

**end**
角色："结束"
"#;

    let script = parser.parse("test", text).unwrap();
    let result = analyze_script(&script);

    assert!(!result.has_errors());
    assert!(result.is_empty());
}

#[test]
fn test_analyze_script_choice_targets() {
    let mut parser = Parser::new();
    let text = r#"
**start**

| 选择 |        |
| ---- | ------ |
| 选项A | path_a |
| 选项B | path_b |

**path_a**
角色："A路线"
"#;

    let script = parser.parse("test", text).unwrap();
    let result = analyze_script(&script);

    assert!(result.has_errors());
    assert_eq!(result.error_count(), 1);
    assert!(result.diagnostics[0].message.contains("path_b"));
}

#[test]
fn test_extract_resource_references() {
    let mut parser = Parser::new();
    let text = r#"
changeBG <img src="backgrounds/room.png" />
show <img src="characters/alice.png" /> as alice at center
<audio src="bgm/theme.mp3"></audio> loop
"#;

    let script = parser
        .parse_with_base_path("test", text, "scripts")
        .unwrap();
    let refs = extract_resource_references(&script);

    assert_eq!(refs.len(), 3);
    assert_eq!(refs[0].resource_type, ResourceType::Background);
    assert_eq!(refs[0].path, "backgrounds/room.png");
    assert_eq!(refs[0].resolved_path, "scripts/backgrounds/room.png");
    assert_eq!(refs[1].resource_type, ResourceType::Character);
    assert_eq!(refs[2].resource_type, ResourceType::Audio);
}

#[test]
fn test_get_defined_labels() {
    let mut parser = Parser::new();
    let text = r#"
**start**
对话

**middle**
对话

**end**
"#;

    let script = parser.parse("test", text).unwrap();
    let labels = get_defined_labels(&script);

    assert_eq!(labels.len(), 3);
    assert!(labels.contains(&"start"));
    assert!(labels.contains(&"middle"));
    assert!(labels.contains(&"end"));
}

#[test]
fn test_analyze_conditional_jump_targets() {
    let mut parser = Parser::new();
    let text = r#"
**start**
set $flag = true

if $flag
goto **valid_target**
else
goto **invalid_target**
endif

**valid_target**
角色："到达"
"#;

    let script = parser.parse("test", text).unwrap();
    let result = analyze_script(&script);

    assert!(result.has_errors());
    assert_eq!(result.error_count(), 1);
    assert!(result.diagnostics[0].message.contains("invalid_target"));
}

#[test]
fn test_analyze_script_with_line_numbers() {
    let mut parser = Parser::new();
    let text = r#"
**start**
角色："对话"
goto **missing**
"#;

    let script = parser.parse("test", text).unwrap();
    assert!(script.has_source_map());

    let result = analyze_script(&script);
    assert!(result.has_errors());
    assert_eq!(result.error_count(), 1);

    let diag = &result.diagnostics[0];
    assert!(diag.line.is_some());
    assert_eq!(diag.line, Some(4));
}

#[test]
fn test_script_source_map() {
    let mut parser = Parser::new();
    let text = r#"
**start**
角色："第一句"
角色："第二句"
**end**
"#;

    let script = parser.parse("test", text).unwrap();

    assert!(script.has_source_map());
    assert_eq!(script.get_source_line(0), Some(2));
    assert_eq!(script.get_source_line(1), Some(3));
    assert_eq!(script.get_source_line(2), Some(4));
    assert_eq!(script.get_source_line(3), Some(5));
}

#[test]
fn test_extract_resource_references_change_scene() {
    let mut parser = Parser::new();
    let text = r#"changeScene <img src="bg/new.png" /> with Dissolve(duration: 1)"#;
    let script = parser
        .parse_with_base_path("test", text, "scripts")
        .unwrap();
    let refs = extract_resource_references(&script);

    assert_eq!(refs.len(), 1);
    assert_eq!(refs[0].resource_type, ResourceType::Scene);
    assert_eq!(refs[0].path, "bg/new.png");
    assert_eq!(refs[0].resolved_path, "scripts/bg/new.png");
}

#[test]
fn test_extract_resource_references_show_without_path() {
    let mut parser = Parser::new();
    let text = "show alice at center";
    let script = parser.parse("test", text).unwrap();
    let refs = extract_resource_references(&script);
    assert!(refs.is_empty());
}

#[test]
fn test_extract_resource_references_in_conditional() {
    let mut parser = Parser::new();
    let text = r#"
if $flag == true
  changeBG <img src="bg_a.png" />
else
  changeBG <img src="bg_b.png" />
endif
"#;
    let script = parser
        .parse_with_base_path("test", text, "scripts")
        .unwrap();
    let refs = extract_resource_references(&script);
    assert_eq!(refs.len(), 2);
    assert_eq!(refs[0].resource_type, ResourceType::Background);
    assert_eq!(refs[1].resource_type, ResourceType::Background);
}

#[test]
fn test_extract_resource_references_includes_cutscene() {
    let mut parser = Parser::new();
    let script = parser
        .parse_with_base_path("test", r#"cutscene "video/opening.mp4""#, "scripts")
        .unwrap();
    let refs = extract_resource_references(&script);

    assert_eq!(refs.len(), 1);
    assert_eq!(refs[0].resource_type, ResourceType::Video);
    assert_eq!(refs[0].path, "video/opening.mp4");
    assert_eq!(refs[0].resolved_path, "scripts/video/opening.mp4");
}

#[test]
fn test_get_jump_targets() {
    let mut parser = Parser::new();
    let text = r#"
**start**
goto **end**
goto **end**

| 选择 |        |
| --- | --- |
| A | target_a |

**end**
角色："结束"
"#;

    let script = parser.parse("test", text).unwrap();
    let targets = get_jump_targets(&script);

    assert!(targets.contains("end"));
    assert!(targets.contains("target_a"));
    assert_eq!(targets.len(), 2);
}

#[test]
fn test_diagnostic_result_filter() {
    let mut result = DiagnosticResult::new();
    result.push(Diagnostic::error("test", "错误1"));
    result.push(Diagnostic::warn("test", "警告1"));
    result.push(Diagnostic::info("test", "信息1"));

    let errors = result.filter_by_level(DiagnosticLevel::Error);
    assert_eq!(errors.len(), 1);

    let warns_and_errors = result.filter_by_level(DiagnosticLevel::Warn);
    assert_eq!(warns_and_errors.len(), 2);

    let all = result.filter_by_level(DiagnosticLevel::Info);
    assert_eq!(all.len(), 3);
}

#[test]
fn test_diagnostic_result_merge() {
    let mut result1 = DiagnosticResult::new();
    result1.push(Diagnostic::error("a.md", "err1"));

    let mut result2 = DiagnosticResult::new();
    result2.push(Diagnostic::warn("b.md", "warn1"));
    result2.push(Diagnostic::info("b.md", "info1"));

    result1.merge(result2);
    assert_eq!(result1.diagnostics.len(), 3);
    assert_eq!(result1.error_count(), 1);
    assert_eq!(result1.warn_count(), 1);
}
