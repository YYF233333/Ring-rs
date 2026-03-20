# host/extensions 摘要

## Purpose

`host/extensions` 提供效果扩展 API、capability 注册中心与调度入口，用于把“效果能力”从核心流程中解耦。

## PublicSurface

- 模块入口：`host/src/extensions/mod.rs`
- 核心类型：`ExtensionRegistry`、`EngineContext`、`ExtensionManifest`、`EffectExtension`、`EngineServices` trait（定义于 `services.rs`）、`CapabilityId` newtype；内建 capability 常量与 `build_builtin_registry` 在 `builtin_effects.rs`
- 关键能力：注册扩展、版本兼容校验、capability 调度、扩展诊断记录

## KeyFlow

1. `AppState::new` 构建内建扩展注册表（`builtin_effects.rs`）：`effect.dissolve`、`effect.fade`、`effect.rule_mask`、`effect.move`，以及场景效果 `effect.scene.shake`、`effect.scene.blur`、`effect.scene.dim`、`effect.scene.title_card`。
2. `command_executor` 产出带 `capability_id` 的 `EffectRequest`。
3. `EngineContext` 持有 `&mut dyn EngineServices`（而非 `&mut CoreSystems`），通过 trait 抽象访问核心系统，打破对 `app` 模块的反向依赖。
4. `effect_applier` 使用 `ExtensionRegistry` 按 capability 分发请求。
5. capability 缺失或执行失败时执行 capability 级回退并输出诊断。

## Dependencies

- 上游依赖：`renderer/effects`（请求模型）
- 下游依赖：`renderer` 动画与过渡系统

## Invariants

- 同一 `capability_id` 只能由一个扩展注册，避免行为冲突。
- 扩展 API 版本按主版本兼容；主版本不一致拒绝注册。
- 诊断必须包含 `capability_id` 与扩展来源。
- extensions 模块不 `use crate::app`，通过 `EngineServices` trait 抽象访问核心系统。

## FailureModes

- API 主版本不兼容，扩展注册失败。
- capability 重复注册，触发冲突错误。
- capability 执行失败，触发统一回退映射；若回退能力仍不可用，则放弃该效果请求。

## WhenToReadSource

- 新增效果能力或迁移内建效果到扩展时。
- 排查 capability 分发与回退行为不一致时。

## RelatedDocs

- [renderer_effects 摘要](renderer_effects.md)
- [app_command_handlers 摘要](app_command_handlers.md)
- [script_syntax_spec](../../script_syntax_spec.md)

## LastVerified

2026-03-18

## Owner

Composer