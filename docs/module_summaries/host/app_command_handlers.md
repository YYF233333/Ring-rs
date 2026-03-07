# host/app/command_handlers 摘要

## Purpose

`app/command_handlers` 处理 `CommandExecutor` 产出的副作用请求，把“执行器输出”转为音频播放与效果应用动作。

## PublicSurface

- 模块入口：`host/src/app/command_handlers/mod.rs`
- 子模块：`audio`、`effect_applier`
- 对外导出：音频处理与效果应用相关函数

## KeyFlow

1. `command_executor` 执行命令后产出输出结构（音频命令、效果请求等）。
2. `command_handlers/audio` 消费音频输出并驱动 `AudioManager`。
3. `command_handlers/effect_applier` 消费效果请求，先按 capability 路由到扩展注册表，再按统一回退映射降级。

## Dependencies

- 上游依赖：`command_executor`
- 下游依赖：`audio`、`renderer`、`renderer/effects`、`extensions`

## Invariants

- `command_handlers` 只处理副作用，不承担命令语义解析。
- 效果应用路径统一走 `effect_applier`，减少分散分支。
- 诊断必须包含 `capability_id` 与扩展来源，便于定位扩展行为。

## FailureModes

- 输出事件未被消费，导致命令执行后没有可见效果。
- 音频与视觉副作用处理顺序不当，造成时序不一致。

## WhenToReadSource

- 需要新增命令副作用类型时。
- 需要排查“命令已执行但外部系统未生效”问题时。

## RelatedDocs

- [app 摘要](app.md)
- [command_executor 摘要](command_executor.md)
- [renderer_effects 摘要](renderer_effects.md)

## LastVerified

2026-03-07

## Owner

Ring-rs 维护者
