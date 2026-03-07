# vn-runtime/script 摘要

## Purpose

负责脚本语义模型与解析入口：定义 AST、表达式系统，并导出 `Parser` 供上层使用。

## PublicSurface

- 模块入口：`vn-runtime/src/script/mod.rs`
- 子模块：`ast`、`expr`、`parser`
- 重导出：`Script`、`ScriptNode`、`ChoiceOption`、`Parser`、表达式求值接口

## KeyFlow

1. 脚本文本进入 `parser`。
2. 生成 `Script`（节点序列 + base_path + source_map）。
3. `runtime/executor` 消费 `ScriptNode` 产生命令。
4. 条件分支与变量赋值依赖 `expr` 求值器。
5. 阶段 0 新增跨文件控制流节点：`CallScript`、`ReturnFromScript`。

## Dependencies

- 依赖 `command` 中的部分类型（如 transition、position）表达语义。
- 被 `runtime`、`diagnostic`、`xtask script-check` 直接消费。

## Invariants

- AST 只描述语义，不携带宿主层实现细节。
- 表达式求值通过 `EvalContext` 访问变量，不读全局状态。

## FailureModes

- 语法非法导致 `ParseError`。
- 表达式变量缺失/类型不匹配导致求值错误。

## WhenToReadSource

- 新增脚本语法或扩展节点类型时。
- 需要确认某个脚本语句映射到哪个 `ScriptNode` 时。

## RelatedDocs

- [parser 专题](parser.md)
- [脚本语法规范](../../script_syntax_spec.md)
- [模块总览](../vn-runtime.md)

## LastVerified

2026-03-07

## Owner

Ring-rs 维护者
