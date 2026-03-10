# host/renderer/effects 摘要

## Purpose

`renderer/effects` 统一定义和解析视觉效果，把 Runtime 过渡描述转换为 Host 可执行的标准化效果结构。

## PublicSurface

- 模块入口：`host/src/renderer/effects/mod.rs`
- 核心类型：`EffectKind`、`ResolvedEffect`、`EffectRequest`、`EffectTarget`、`EffectParamValue`
- `EffectTarget` 含 `SceneEffect { effect_name }`、`TitleCard { text }` 等 variants
- `EffectKind` 含 `SceneEffect { name }` variant
- 关键接口：`resolve(Transition) -> ResolvedEffect`

## KeyFlow

1. 输入来自 `vn-runtime::command::Transition`。
2. `resolver` 解析效果名与参数，得到 `ResolvedEffect`。
3. `command_executor` 产出 `EffectRequest`，自动携带 `capability_id + params`。
4. `app/command_handlers/effect_applier` 通过扩展注册表按 capability 分发执行。
5. 已注册能力：`effect.scene.shake`、`effect.scene.blur`、`effect.scene.dim`、`effect.scene.title_card`。

## Dependencies

- 上游依赖：`vn-runtime` 命令模型
- 下游依赖：`command_executor`、`app/command_handlers`、`renderer` 动画/过渡系统

## Invariants

- 效果映射与默认参数来源唯一，避免多处定义分叉。
- 解析器只负责语义归一化，不绑定具体目标对象。
- `EffectRequest.capability_id` 必须可稳定映射到注册表能力，便于扩展化迁移与诊断。

## FailureModes

- 过渡参数非法或缺失，导致效果降级或未执行。
- 映射表不完整，导致新效果无法被识别。
- capability 缺失或执行失败时，需要回退到 legacy 路径并输出 capability 诊断。

## WhenToReadSource

- 需要新增效果类型或扩展参数协议时。
- 需要排查 Runtime 过渡到 Host 执行路径不一致时。

## RelatedDocs

- [renderer 摘要](renderer.md)
- [command_executor 摘要](command_executor.md)
- [app_command_handlers 摘要](app_command_handlers.md)

## LastVerified

2026-03-10

## Owner

Ring-rs 维护者
