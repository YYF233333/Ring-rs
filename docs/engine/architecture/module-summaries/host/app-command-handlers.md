# host/app/command_handlers 摘要

## Purpose

`app/command_handlers` 消费 `CommandExecutor` 的副作用输出，把“状态变更之外的事”交给音频系统和效果 capability 系统处理。

## PublicSurface

- 入口：`host/src/app/command_handlers/mod.rs`
- 关键接口：`handle_audio_command`、`apply_effect_requests`
- 子模块：`audio`、`effect_applier`

## KeyFlow

1. `handle_audio_command` 读取 `last_output.audio_command`，先通过 `ResourceManager` 取字节并缓存，再驱动 `AudioManager`。
2. `apply_effect_requests` 取走 `last_output.effect_requests`，为每个请求构造 `EngineContext` 并交给 `ExtensionRegistry` 分发。
3. capability 缺失或执行失败时，`effect_applier` 按 target/effect 做统一回退，并在末尾输出扩展诊断。

## Invariants

- 这里只处理副作用，不解释脚本语义或等待态。
- 音频加载统一走 `ResourceManager`，不在调用方区分 FS/ZIP。
- 效果请求统一从 `effect_applier` 进入扩展层，避免散落多处分支。

## WhenToReadSource

- 需要新增一种 `CommandOutput` 副作用时。
- 需要排查“命令执行成功但音频/效果没生效”时。
- 需要修改 capability 回退表或诊断输出时。

## RelatedDocs

- [app 摘要](app.md)
- [command_executor 摘要](command-executor.md)
- [extensions 摘要](extensions.md)
- [renderer_effects 摘要](renderer-effects.md)

## LastVerified

2026-03-24

## Owner

GPT-5.4
