# vn-runtime/diagnostic 摘要

## Purpose

提供无 IO 的脚本静态检查能力：检查跳转目标有效性，并提取资源引用供外部校验。

## PublicSurface

- 文件入口：`vn-runtime/src/diagnostic/mod.rs`
- 关键类型：`Diagnostic`、`DiagnosticResult`、`DiagnosticLevel`
- 关键函数：`analyze_script`、`extract_resource_references`、`get_defined_labels`、`get_jump_targets`

## KeyFlow

1. 输入已解析的 `Script`。
2. 收集已定义 label 与跳转目标（含 source_map 行号）。
3. 生成错误/警告级别诊断。
4. 递归遍历节点提取背景/场景/立绘/音频引用。

## Dependencies

- 复用 `script` 的 AST 与路径解析能力，不重复解析逻辑。
- 被 `xtask script-check` 使用，构成本地脚本门禁链路。

## Invariants

- 诊断 API 保持纯函数风格，无环境依赖。
- 行号来自 `Script` 的 source map，依赖 parser 阶段产物质量。

## FailureModes

- 跳转目标缺失：`goto` 或 `choice` 指向不存在 label。
- source map 失真时，诊断行号可能偏移。

## WhenToReadSource

- 需要新增诊断规则（如重复 label、未使用 label）时。
- 需要调整输出格式或诊断分级时。

## RelatedDocs

- [模块总览](../vn-runtime.md)
- [parser 专题](parser.md)
- [coverage 指南](../../../../testing/coverage.md)

## LastVerified

2026-03-18

## Owner

Composer