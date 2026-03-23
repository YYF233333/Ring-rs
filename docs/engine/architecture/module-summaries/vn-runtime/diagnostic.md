# vn-runtime/diagnostic 摘要

## Purpose

提供纯静态、无 IO 的脚本诊断能力：校验跳转目标，并提取资源引用给外部工具继续检查。

## PublicSurface

- 文件入口：`vn-runtime/src/diagnostic/mod.rs`
- 关键类型：`Diagnostic`、`DiagnosticResult`、`DiagnosticLevel`
- 关键函数：`analyze_script`、`extract_resource_references`、`get_defined_labels`、`get_jump_targets`

## KeyFlow

1. 输入已解析的 `Script`。
2. 收集 label 定义、跳转目标与对应 source map 行号。
3. 当前实现只落地“未定义跳转目标”错误；`Warn` / `Info` 等级已建模但暂无规则。
4. 递归遍历节点，提取背景、场景、角色、音频、视频等资源引用。

## Dependencies

- 复用 `script` 的 AST 与路径解析逻辑。
- 被 `xtask script-check` 等外部检查链路消费。

## Invariants

- 诊断 API 保持纯函数风格，不依赖运行时状态或外部环境。
- 行号来自 `Script` 的 source map，准确性取决于 parser 产物。

## FailureModes

- `goto` 或 `choice` 指向未定义 label。
- source map 漂移会让诊断行号不准确。

## WhenToReadSource

- 需要新增诊断规则或调整分级时。
- 需要确认资源提取是否覆盖某个新节点时。
- 需要排查行号来源或输出格式时。

## RelatedDocs

- [模块总览](../vn-runtime.md)
- [parser 专题](parser.md)

## LastVerified

2026-03-24

## Owner

GPT-5.4