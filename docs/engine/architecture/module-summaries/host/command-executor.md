# host/command_executor 摘要

## Purpose

`command_executor` 是 Host 侧的命令执行器：把 `vn-runtime::Command` 翻译为 `RenderState` 变更，并把音频/效果等外部副作用记录到 `last_output`。

## PublicSurface

- 入口：`host/src/command_executor/mod.rs`
- 核心类型：`CommandExecutor`、`CommandOutput`、`ExecuteResult`
- 关键接口：`execute`、`execute_batch`
- 主要子模块：`audio`、`background`、`character`、`effects`、`ui`、`types`

## KeyFlow

1. `execute()` 每次先重置 `last_output`，再按命令类型分发到子模块。
2. UI/背景/角色命令直接改 `RenderState`；音频和效果命令只记录副作用输出。
3. `ShowText` / `ExtendText` / `SetTextMode` 等文本相关命令集中在 `ui.rs`。
4. `execute_batch()` 顺序执行多条命令，返回最后一个等待结果，遇到错误立即中止。
5. `FullRestart`、`Cutscene`、`RequestUI` 在这里不做上层编排，只返回 `Ok` 交给 `app/update/script.rs` 处理。

## Invariants

- 执行器负责状态翻译，不做渲染、不直接驱动外部系统。
- `last_output` 的生命周期以“单条命令执行”为单位，避免跨命令泄漏。
- `Command` 到副作用的边界应稳定：状态改写留在这里，副作用消费留给 `app/command_handlers`。

## WhenToReadSource

- 需要新增 `Command` 的 Host 落地逻辑时。
- 需要排查某个等待态是在哪条命令上产生时。
- 需要确认文本模式、续接台词或场景效果如何改写 `RenderState` 时。

## RelatedDocs

- [host 总览](../host.md)
- [app_update 摘要](app-update.md)
- [app_command_handlers 摘要](app-command-handlers.md)
- [renderer_render_state 摘要](renderer-render-state.md)
- [vn-runtime command 摘要](../vn-runtime/command.md)

## LastVerified

2026-03-24

## Owner

GPT-5.4
