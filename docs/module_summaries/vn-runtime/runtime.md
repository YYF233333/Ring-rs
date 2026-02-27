# vn-runtime/runtime 摘要

## Purpose

驱动脚本执行循环：接收输入、管理等待状态、执行节点并输出 `Command`。

## PublicSurface

- 模块入口：`vn-runtime/src/runtime/mod.rs`
- 核心类型：`VNRuntime`（engine）
- 执行器：`Executor`（`runtime/executor`）

## KeyFlow

1. `VNRuntime::tick(Option<RuntimeInput>)` 先处理输入解除等待。
2. 若仍在等待，直接返回当前等待原因。
3. 否则循环取 `ScriptNode`，交给 `Executor::execute`。
4. 汇总 `commands`，处理 `jump_to` 与 `waiting`。
5. 记录历史事件（对话、章节、背景、BGM、跳转/选项）。

## Dependencies

- 读取：`Script`（节点与 label 索引）
- 写入：`RuntimeState`（位置、变量、背景、角色、等待）
- 输出：`Command` 给 Host
- 使用：`History` 记录可回放事件

## Invariants

- `tick` 是唯一推进入口。
- 等待状态由 `WaitingReason` 显式建模。
- 输入与等待状态不匹配时返回 `RuntimeError::StateMismatch`。

## FailureModes

- `ChoiceSelected` 索引越界。
- `Goto`/Choice 目标 label 不存在。
- 条件表达式求值失败。

## WhenToReadSource

- 需要确认等待解除逻辑（Click/Choice/Signal/Time）时。
- 需要验证历史记录与命令映射是否一致时。

## RelatedDocs

- [模块总览](../vn-runtime.md)
- [state 模块接口](../../../vn-runtime/src/state.rs)
- [存档格式](../../save_format.md)

## LastVerified

2026-02-27

## Owner

Ring-rs 维护者
