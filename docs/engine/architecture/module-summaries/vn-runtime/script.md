# vn-runtime/script 摘要

## Purpose

定义脚本语义模型，并向上层导出解析入口与表达式求值能力。

## PublicSurface

- 模块入口：`vn-runtime/src/script/mod.rs`
- 子模块：`ast`、`expr`、`parser`
- 重导出：`Script`、`ScriptNode`、`ChoiceOption`、`Parser` 以及表达式求值接口

## KeyFlow

1. `parser` 读取文本并产出 `Script`。
2. `ast` 用 `ScriptNode` 描述章节、对话、控制流、媒体/UI 请求等脚本语义。
3. `expr` 为条件分支、变量赋值和 UI 参数提供求值能力。
4. `runtime/executor` 读取这些节点并生成 `Command` 或等待状态。
5. 对话节点保留 `inline_effects` / `no_wait`，续接台词用 `Extend`，跨脚本流程用 `CallScript` / `ReturnFromScript`。

## Dependencies

- 复用 `command` 中的结构化类型表达位置、过渡和文本模式等语义。
- 被 `runtime`、`diagnostic` 与脚本检查链路直接消费。

## Invariants

- AST 只描述“脚本想表达什么”，不携带宿主实现细节。
- 表达式求值通过 `EvalContext` 访问变量，不读取隐藏全局状态。

## FailureModes

- 语法错误会在解析阶段产出 `ParseError`。
- 变量缺失或类型不匹配会在求值阶段失败。

## WhenToReadSource

- 需要确认某条语句具体落到哪个 `ScriptNode` 时。
- 需要新增脚本语法、字段或控制流节点时。
- 需要核对 UI 请求、跨脚本调用或对话扩展字段的精确语义时。

## RelatedDocs

- [parser 专题](parser.md)
- [脚本语法规范](../../../../authoring/script-syntax.md)
- [模块总览](../vn-runtime.md)

## LastVerified

2026-03-24

## Owner

GPT-5.4