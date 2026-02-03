//! # 诊断模块
//!
//! 提供脚本静态检查和诊断 API，不依赖 IO 或引擎。
//!
//! ## 设计原则
//!
//! - 纯函数 API，可在无 IO 环境下运行
//! - 诊断分级：Error（必须修复）、Warn（建议修复）、Info（信息提示）
//! - 复用 parser/AST，不重复解析逻辑

use std::collections::HashSet;

use crate::script::{Script, ScriptNode};

/// 诊断级别
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DiagnosticLevel {
    /// 信息提示
    Info,
    /// 警告（建议修复）
    Warn,
    /// 错误（必须修复）
    Error,
}

impl std::fmt::Display for DiagnosticLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Info => write!(f, "INFO"),
            Self::Warn => write!(f, "WARN"),
            Self::Error => write!(f, "ERROR"),
        }
    }
}

/// 诊断条目
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    /// 诊断级别
    pub level: DiagnosticLevel,
    /// 脚本 ID / 文件路径
    pub script_id: String,
    /// 行号（如果可定位，从 1 开始）
    pub line: Option<usize>,
    /// 诊断消息
    pub message: String,
    /// 诊断详情（可选，如原始行内容）
    pub detail: Option<String>,
}

impl Diagnostic {
    /// 创建错误诊断
    pub fn error(script_id: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            level: DiagnosticLevel::Error,
            script_id: script_id.into(),
            line: None,
            message: message.into(),
            detail: None,
        }
    }

    /// 创建警告诊断
    pub fn warn(script_id: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            level: DiagnosticLevel::Warn,
            script_id: script_id.into(),
            line: None,
            message: message.into(),
            detail: None,
        }
    }

    /// 创建信息诊断
    pub fn info(script_id: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            level: DiagnosticLevel::Info,
            script_id: script_id.into(),
            line: None,
            message: message.into(),
            detail: None,
        }
    }

    /// 设置行号
    pub fn with_line(mut self, line: usize) -> Self {
        self.line = Some(line);
        self
    }

    /// 设置详情
    pub fn with_detail(mut self, detail: impl Into<String>) -> Self {
        self.detail = Some(detail.into());
        self
    }
}

impl std::fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {}", self.level, self.script_id)?;
        if let Some(line) = self.line {
            write!(f, ":{}", line)?;
        }
        write!(f, ": {}", self.message)?;
        if let Some(detail) = &self.detail {
            write!(f, "\n  | {}", detail)?;
        }
        Ok(())
    }
}

/// 诊断结果
#[derive(Debug, Clone, Default)]
pub struct DiagnosticResult {
    /// 诊断条目列表
    pub diagnostics: Vec<Diagnostic>,
}

impl DiagnosticResult {
    /// 创建空结果
    pub fn new() -> Self {
        Self::default()
    }

    /// 添加诊断
    pub fn push(&mut self, diagnostic: Diagnostic) {
        self.diagnostics.push(diagnostic);
    }

    /// 合并另一个结果
    pub fn merge(&mut self, other: DiagnosticResult) {
        self.diagnostics.extend(other.diagnostics);
    }

    /// 获取错误数量
    pub fn error_count(&self) -> usize {
        self.diagnostics
            .iter()
            .filter(|d| d.level == DiagnosticLevel::Error)
            .count()
    }

    /// 获取警告数量
    pub fn warn_count(&self) -> usize {
        self.diagnostics
            .iter()
            .filter(|d| d.level == DiagnosticLevel::Warn)
            .count()
    }

    /// 是否有错误
    pub fn has_errors(&self) -> bool {
        self.error_count() > 0
    }

    /// 是否为空
    pub fn is_empty(&self) -> bool {
        self.diagnostics.is_empty()
    }

    /// 按级别过滤
    pub fn filter_by_level(&self, min_level: DiagnosticLevel) -> Vec<&Diagnostic> {
        self.diagnostics
            .iter()
            .filter(|d| d.level >= min_level)
            .collect()
    }
}

/// 资源引用信息
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceReference {
    /// 资源类型
    pub resource_type: ResourceType,
    /// 资源路径（脚本中的原始路径）
    pub path: String,
    /// 解析后的逻辑路径（相对于 assets_root）
    pub resolved_path: String,
}

/// 资源类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ResourceType {
    /// 背景图片
    Background,
    /// 场景图片
    Scene,
    /// 角色立绘
    Character,
    /// 音频（BGM/SFX）
    Audio,
}

impl std::fmt::Display for ResourceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Background => write!(f, "背景"),
            Self::Scene => write!(f, "场景"),
            Self::Character => write!(f, "立绘"),
            Self::Audio => write!(f, "音频"),
        }
    }
}

//=============================================================================
// 脚本分析 API
//=============================================================================

/// 跳转目标信息（包含行号）
struct JumpTarget {
    label: String,
    line: Option<usize>,
}

/// 分析脚本，返回诊断结果
///
/// 执行以下检查：
/// - 未定义的跳转目标（goto/choice 目标 label 不存在）
///
/// # 参数
///
/// - `script`: 已解析的脚本
///
/// # 返回
///
/// 诊断结果
pub fn analyze_script(script: &Script) -> DiagnosticResult {
    let mut result = DiagnosticResult::new();

    // 收集所有已定义的 label
    let defined_labels: HashSet<&str> = script
        .nodes
        .iter()
        .filter_map(|node| node.as_label())
        .collect();

    // 收集所有跳转目标并检查（带行号）
    let jump_targets = collect_jump_targets_with_lines(script);
    for target in jump_targets {
        if !defined_labels.contains(target.label.as_str()) {
            let mut diag = Diagnostic::error(
                &script.id,
                format!("未定义的跳转目标: **{}**", target.label),
            )
            .with_detail(format!(
                "goto 或 choice 引用了不存在的 label '{}'",
                target.label
            ));
            if let Some(line) = target.line {
                diag = diag.with_line(line);
            }
            result.push(diag);
        }
    }

    result
}

/// 从脚本收集所有跳转目标（带行号信息）
fn collect_jump_targets_with_lines(script: &Script) -> Vec<JumpTarget> {
    let mut targets = Vec::new();

    for (index, node) in script.nodes.iter().enumerate() {
        let line = script.get_source_line(index);
        collect_targets_from_node(node, line, &mut targets);
    }

    targets
}

/// 从单个节点收集跳转目标
fn collect_targets_from_node(
    node: &ScriptNode,
    line: Option<usize>,
    targets: &mut Vec<JumpTarget>,
) {
    match node {
        ScriptNode::Goto { target_label } => {
            targets.push(JumpTarget {
                label: target_label.clone(),
                line,
            });
        }
        ScriptNode::Choice { options, .. } => {
            for opt in options {
                targets.push(JumpTarget {
                    label: opt.target_label.clone(),
                    line,
                });
            }
        }
        ScriptNode::Conditional { branches } => {
            // 递归收集条件分支中的跳转目标（条件分支内部节点没有独立行号）
            for branch in branches {
                for inner_node in &branch.body {
                    collect_targets_from_node(inner_node, line, targets);
                }
            }
        }
        _ => {}
    }
}

/// 提取脚本中的所有资源引用
///
/// 遍历脚本节点，提取背景、场景、角色立绘、音频等资源引用。
///
/// # 参数
///
/// - `script`: 已解析的脚本
///
/// # 返回
///
/// 资源引用列表
pub fn extract_resource_references(script: &Script) -> Vec<ResourceReference> {
    let mut refs = Vec::new();
    extract_from_nodes(&script.nodes, script, &mut refs);
    refs
}

/// 从节点列表提取资源引用
fn extract_from_nodes(nodes: &[ScriptNode], script: &Script, refs: &mut Vec<ResourceReference>) {
    for node in nodes {
        match node {
            ScriptNode::ChangeBG { path, .. } => {
                refs.push(ResourceReference {
                    resource_type: ResourceType::Background,
                    path: path.clone(),
                    resolved_path: script.resolve_path(path),
                });
            }
            ScriptNode::ChangeScene { path, .. } => {
                refs.push(ResourceReference {
                    resource_type: ResourceType::Scene,
                    path: path.clone(),
                    resolved_path: script.resolve_path(path),
                });
            }
            ScriptNode::ShowCharacter { path: Some(p), .. } => {
                refs.push(ResourceReference {
                    resource_type: ResourceType::Character,
                    path: p.clone(),
                    resolved_path: script.resolve_path(p),
                });
            }
            ScriptNode::PlayAudio { path, .. } => {
                refs.push(ResourceReference {
                    resource_type: ResourceType::Audio,
                    path: path.clone(),
                    resolved_path: script.resolve_path(path),
                });
            }
            ScriptNode::Conditional { branches } => {
                // 递归提取条件分支中的资源引用
                for branch in branches {
                    extract_from_nodes(&branch.body, script, refs);
                }
            }
            _ => {}
        }
    }
}

/// 获取脚本中所有已定义的 label 名称
pub fn get_defined_labels(script: &Script) -> Vec<&str> {
    script
        .nodes
        .iter()
        .filter_map(|node| node.as_label())
        .collect()
}

/// 获取脚本中所有跳转目标（去重）
pub fn get_jump_targets(script: &Script) -> HashSet<String> {
    collect_jump_targets_with_lines(script)
        .into_iter()
        .map(|t| t.label)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::script::Parser;

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

        // path_b 未定义
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

        // 验证背景
        assert_eq!(refs[0].resource_type, ResourceType::Background);
        assert_eq!(refs[0].path, "backgrounds/room.png");
        assert_eq!(refs[0].resolved_path, "scripts/backgrounds/room.png");

        // 验证立绘
        assert_eq!(refs[1].resource_type, ResourceType::Character);

        // 验证音频
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

        // invalid_target 未定义
        assert!(result.has_errors());
        assert_eq!(result.error_count(), 1);
        assert!(result.diagnostics[0].message.contains("invalid_target"));
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
    fn test_analyze_script_with_line_numbers() {
        let mut parser = Parser::new();
        // 脚本内容：
        // 第1行：空
        // 第2行：**start**
        // 第3行：角色："对话"
        // 第4行：goto **missing**
        let text = r#"
**start**
角色："对话"
goto **missing**
"#;

        let script = parser.parse("test", text).unwrap();

        // 验证 source_map 已填充
        assert!(script.has_source_map());

        let result = analyze_script(&script);
        assert!(result.has_errors());
        assert_eq!(result.error_count(), 1);

        // 验证诊断带有行号
        let diag = &result.diagnostics[0];
        assert!(diag.line.is_some());
        // goto 在第4行（从1开始计数）
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
        // 节点0: **start** -> 第2行
        assert_eq!(script.get_source_line(0), Some(2));
        // 节点1: 对话 -> 第3行
        assert_eq!(script.get_source_line(1), Some(3));
        // 节点2: 对话 -> 第4行
        assert_eq!(script.get_source_line(2), Some(4));
        // 节点3: **end** -> 第5行
        assert_eq!(script.get_source_line(3), Some(5));
    }
}
