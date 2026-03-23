# vn-runtime/parser 摘要

## Purpose

把脚本文本解析成 `Script` AST，并保留 source map、基础路径和可恢复的告警信息。

## PublicSurface

- 入口：`vn-runtime/src/script/parser/mod.rs`
- 核心接口：`Parser::parse`、`Parser::parse_with_base_path`、`Parser::warnings`
- 主要内部域：`phase1`、`phase2`、`helpers`、`expr_parser`、`inline_tags`

## KeyFlow

1. `phase1` 先把原始文本切成块结构。
2. `phase2` 再按显示、控制流、对话、杂项四类规则降为 `ScriptNode`。
3. 解析过程同步构建 source map，并通过 `Script::with_source_map` 产出最终脚本。
4. 对话相关规则会调用 `inline_tags` 剥离 `{wait}` / `{speed}` 等标签，并设置 `no_wait` / `Extend` 等字段。
5. 控制流与杂项规则还覆盖 `callScript`、`returnFromScript`、`sceneEffect`、`titleCard`、`cutscene`、`requestUI` 及其语法糖、`textMode` 等语句。

## Dependencies

- 依赖 `script/ast` 承载解析结果。
- 依赖 `command` 中的结构化类型表达过渡、文本模式等参数。
- 解析结果被 `runtime` 与 `diagnostic` 复用。

## Invariants

- 解析器主要依赖手写字符串处理，而非 regex 驱动。
- `parse_with_base_path` 决定相对资源路径的解析上下文。
- `warnings` 与硬错误分离，允许在保留有效节点的同时报告可恢复问题。

## FailureModes

- 语法结构或参数格式错误导致 `ParseError`。
- `base_path` 错误会让资源逻辑路径解析偏移。
- phase2 规则调整可能影响 source map 或告警位置。

## WhenToReadSource

- 需要新增语法关键字、块类型或对话标签时。
- 需要确认某条脚本语句到底落在哪个 phase2 分支时。
- 需要排查报错行号、告警行为或路径解析问题时。

## RelatedDocs

- [script 子模块摘要](script.md)
- [脚本语法规范](../../../../authoring/script-syntax.md)
- [模块总览](../vn-runtime.md)

## LastVerified

2026-03-24

## Owner

GPT-5.4