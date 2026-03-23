# host/renderer/effects 摘要

## Purpose

`renderer/effects` 把 runtime 的 `Transition` 归一化为 Host 可执行的效果模型，并补齐 capability 路由所需的参数快照，供执行器与 effect applier 共用。

## PublicSurface

- 模块入口：`host/src/renderer/effects/mod.rs`
- 核心类型：`EffectKind`、`ResolvedEffect`、`EffectRequest`、`EffectTarget`、`EffectParamValue`
- 关键接口：`resolve(&Transition) -> ResolvedEffect`、`EffectRequest::new`

## KeyFlow

1. `resolve()` 把 `Transition` 名称/参数解析为 `ResolvedEffect`。
2. `CommandExecutor` 结合 target 上下文构造 `EffectRequest`。
3. `EffectRequest` 自动推导 `capability_id` 与规范化参数。
4. `app/command_handlers/effect_applier` 再按 capability 分发到场景效果、背景过渡、场景转场或标题字卡处理器。

## Dependencies

- 上游依赖：`vn-runtime` 命令模型
- 下游依赖：`command_executor`、`app/command_handlers`、`renderer` 动画/过渡系统

## Invariants

- 效果映射与默认参数来源唯一，避免多处定义分叉。
- 解析器只负责语义归一化，不绑定具体目标对象。
- `EffectRequest.capability_id` 由 target + effect 共同推导，需保持稳定，便于扩展化迁移与诊断。

## FailureModes

- 过渡参数非法或缺失，导致效果降级或未执行。
- 映射表不完整，导致新效果无法被识别。
- capability 路由与 target 语义不一致时，会造成效果分发错误或回退。

## WhenToReadSource

- 需要新增效果类型或扩展参数协议时。
- 需要排查 Runtime 过渡到 Host 执行路径不一致时。

## RelatedDocs

- [renderer 摘要](renderer.md)
- [command_executor 摘要](command-executor.md)
- [app_command_handlers 摘要](app-command-handlers.md)

## LastVerified

2026-03-24

## Owner

GPT-5.4