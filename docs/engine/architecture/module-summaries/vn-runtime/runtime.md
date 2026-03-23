# vn-runtime/runtime 摘要

## Purpose

负责执行循环本身：消费输入、推进脚本、维护等待与调用栈，并返回本帧生成的 `Command`。

## PublicSurface

- 模块入口：`vn-runtime/src/runtime/mod.rs`
- 核心类型：`VNRuntime`
- 内部执行器：`runtime/executor::Executor`

## KeyFlow

1. `VNRuntime::tick(Option<RuntimeInput>)` 先尝试解除当前等待。
2. 若仍在等待，立即返回空命令和当前 `WaitingReason`。
3. 若可继续执行，则循环取出当前 `ScriptNode`，交给 `Executor` 解释。
4. 执行结果可能产生命令、跳转、等待，或触发 `callScript` / `returnFromScript` 控制流。
5. Runtime 同步维护历史记录、脚本位置、变量域与调用栈。

## Dependencies

- 读取 `Script` 的节点序列、label 索引与路径解析能力。
- 写入 `RuntimeState`，并向 Host 输出 `Command`。
- 使用 `History` 记录可回放事件。

## Invariants

- `tick` 是唯一的推进入口。
- 等待状态由 `WaitingReason` 显式建模，当前实现覆盖 `Click`、`Choice`、`Time`、`Signal`、`UIResult` 五类恢复路径。
- `WaitForChoice` 会校验索引；`WaitForSignal` 与 `WaitForUIResult` 收到不匹配输入时保持等待；`WaitForTime` 只允许 `Click` 打断。
- 跨脚本调用通过 `RuntimeState.call_stack` 显式保存，可序列化恢复。
- 变量严格分为 `variables` 与 `persistent_variables` 两个域，`persistent.*` 不回退到会话变量。

## FailureModes

- 选择索引越界。
- `goto` / `choice` 目标 label 不存在。
- 条件表达式求值失败。
- `callScript` 目标脚本未加载。

## WhenToReadSource

- 需要确认某种等待是“忽略输入”还是“报 `StateMismatch`”时。
- 需要核对跨脚本返回、历史记录或变量写回的精确行为时。
- 需要修改等待模型、恢复语义或状态序列化字段时。

## RelatedDocs

- [模块总览](../vn-runtime.md)
- [存档格式](../../../reference/save-format.md)

## LastVerified

2026-03-24

## Owner

GPT-5.4