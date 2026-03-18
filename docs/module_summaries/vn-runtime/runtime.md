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
5. 处理脚本控制流（`callScript` / `returnFromScript`）并维护调用栈。
6. 记录历史事件（对话、章节、背景、BGM、跳转/选项）。

## Dependencies

- 读取：`Script`（节点与 label 索引）
- 写入：`RuntimeState`（位置、变量、背景、角色、等待）
- 输出：`Command` 给 Host
- 使用：`History` 记录可回放事件

## Invariants

- `tick` 是唯一推进入口。
- 等待状态由 `WaitingReason` 显式建模。
- 输入与等待状态不匹配时返回 `RuntimeError::StateMismatch`。
- `WaitForTime` 可被 `Click` 打断（用于 `wait` 指令的交互打断）。
- 跨文件调用通过 `RuntimeState.call_stack` 显式建模，可序列化恢复。
- `RuntimeState` 包含两个变量域：
  - `variables`：会话变量（随存档序列化，`fullRestart` 后清空）
  - `persistent_variables`：持久变量（bare key，由 host 在启动时注入、`fullRestart` 时持久化）
- `EvalContext::get_var` 严格按命名空间路由：`persistent.key` 查 `persistent_variables`，其余查 `variables`，无跨域回退。

## FailureModes

- `ChoiceSelected` 索引越界。
- `Goto`/Choice 目标 label 不存在。
- 条件表达式求值失败。
- `callScript` 目标脚本未注册。

## WhenToReadSource

- 需要确认等待解除逻辑（Click/Choice/Signal/Time）时。
- 需要验证历史记录与命令映射是否一致时。

## RelatedDocs

- [模块总览](../vn-runtime.md)
- [存档格式](../../save_format.md)
- 状态与变量域定义见源码 `vn-runtime/src/state.rs`

## LastVerified

2026-03-18

## Owner

Ring-rs 维护者
