use super::*;

#[test]
fn test_diagnostic_display() {
    let diag = Diagnostic::error("test.md", "未定义的跳转目标")
        .with_line(10)
        .with_detail("goto **missing_label**");

    let display = format!("{}", diag);
    assert!(display.contains("[ERROR]"));
    assert!(display.contains("test.md:10"));
    assert!(display.contains("未定义的跳转目标"));
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

#[test]
fn test_diagnostic_result_warn_count_and_non_empty() {
    let mut result = DiagnosticResult::new();
    result.push(Diagnostic::warn("test.md", "warn1"));
    result.push(Diagnostic::warn("test.md", "warn2"));
    result.push(Diagnostic::info("test.md", "info1"));

    assert_eq!(result.warn_count(), 2);
    assert!(!result.is_empty());
    assert!(!result.has_errors());
}

#[test]
fn test_diagnostic_display_without_line_and_detail() {
    let diag = Diagnostic::warn("test.md", "some warning");
    let display = format!("{}", diag);
    assert!(display.contains("[WARN]"));
    assert!(display.contains("test.md"));
    assert!(display.contains("some warning"));
    assert!(diag.line.is_none());
    assert!(diag.detail.is_none());
}

#[test]
fn test_diagnostic_display_info_level() {
    let diag = Diagnostic::info("test.md", "info message");
    let display = format!("{}", diag);
    assert!(display.contains("[INFO]"));
}

#[test]
fn test_resource_type_display() {
    assert_eq!(format!("{}", ResourceType::Background), "背景");
    assert_eq!(format!("{}", ResourceType::Scene), "场景");
    assert_eq!(format!("{}", ResourceType::Character), "立绘");
    assert_eq!(format!("{}", ResourceType::Audio), "音频");
}
