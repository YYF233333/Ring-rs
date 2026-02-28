# host/command_executor 摘要

## Purpose

`command_executor` 是 Host 侧命令执行核心：把 `vn-runtime::Command` 翻译为 `RenderState` 变更，并产出外部副作用输出。

## PublicSurface

- 模块入口：`host/src/command_executor/mod.rs`
- 核心类型：`CommandExecutor`
- 关键接口：`execute`、`execute_batch`
- 子模块：`audio`、`background`、`character`、`ui`、`types`

## KeyFlow

1. `execute` 接收单个 `Command`，按类型分发到对应执行函数。
2. 执行函数更新 `RenderState`，并将副作用写入 `last_output`。
3. `execute_batch` 顺序执行命令，汇总等待态或提前返回错误。
4. 上层 `app`/`command_handlers` 根据输出继续驱动音频与效果系统。

## Dependencies

- 输入依赖：`vn_runtime::command::Command`
- 状态依赖：`renderer::RenderState`、`resources::ResourceManager`
- 下游协作：`app/command_handlers`

## Invariants

- 执行器只负责状态转换，不直接做渲染。
- 每次 `execute` 前重置 `last_output`，避免上次输出泄漏。

## FailureModes

- 命令参数与当前状态不兼容，返回 `ExecuteResult::Error`。
- 批处理执行中前置命令失败，导致后续命令不再执行。

## WhenToReadSource

- 需要新增 `Command` 类型的 Host 落地逻辑时。
- 需要排查等待态（点击/选择/时间）来源时。

## RelatedDocs

- [host 总览](../host.md)
- [app_command_handlers 摘要](app_command_handlers.md)
- [renderer_render_state 摘要](renderer_render_state.md)
- [vn-runtime command 摘要](../vn-runtime/command.md)

## LastVerified

2026-02-28

## Owner

Ring-rs 维护者
