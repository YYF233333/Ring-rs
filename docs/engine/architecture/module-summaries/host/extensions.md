# host/extensions 摘要

## Purpose

`host/extensions` 提供效果 capability 扩展层，把具体效果实现从主循环和命令执行器中解耦出来。

## PublicSurface

- 入口：`host/src/extensions/mod.rs`
- 核心类型：`ExtensionRegistry`、`CapabilityId`、`EffectExtension`、`ExtensionManifest`
- 运行时上下文：`EngineContext`、`EngineServices`
- 关键构造：`build_builtin_registry()`

## KeyFlow

1. `AppState::new()` 调用 `build_builtin_registry()` 注册内建 capability，包括 dissolve、fade、rule-mask、move 与 scene effect 系列。
2. `command_executor` 产出带 `capability_id` 的 `EffectRequest`，`app/command_handlers/effect_applier` 再把请求送进 registry。
3. `EngineContext` 通过 `EngineServices` trait 暴露受控引擎能力，避免扩展层反向依赖 `app`。
4. registry 分发失败时由上层做 capability 级回退，并输出扩展诊断。

## Invariants

- 同一 `capability_id` 只能注册一次。
- 扩展兼容性按 `ENGINE_API_VERSION` 校验。
- extensions 访问核心系统必须经过 `EngineServices` 抽象边界。

## WhenToReadSource

- 需要新增 capability、迁移内建效果或调整版本兼容策略时。
- 需要排查 dispatch、回退或扩展诊断时。
- 需要确认某个效果应落在 `renderer/effects` 还是 `extensions` 层时。

## RelatedDocs

- [app_command_handlers 摘要](app-command-handlers.md)
- [renderer_effects 摘要](renderer-effects.md)
- [脚本语法规范](../../../../authoring/script-syntax.md)

## LastVerified

2026-03-24

## Owner

GPT-5.4
