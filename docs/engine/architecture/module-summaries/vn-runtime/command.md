# vn-runtime/command 摘要

## Purpose

定义 Runtime 到 Host 的唯一通信契约。`Command` 负责表达意图，不负责落地实现。

## PublicSurface

- 文件入口：`vn-runtime/src/command/mod.rs`
- 核心类型：`Command`、`Transition`、`TransitionArg`、`Choice`、`Position`、`TextMode`
- 节奏标签类型：`InlineEffect`、`InlineEffectKind`
- 等待恢复相关常量：`SIGNAL_*`

## KeyFlow

1. `runtime/executor` 按 `ScriptNode` 语义构造 `Command`。
2. `VNRuntime::tick` 按执行顺序返回 `Vec<Command>`。
3. Host 将命令翻译为渲染、音频、UI 或外部系统动作。
4. 需要 Host 回传恢复信号的命令，通过 `SIGNAL_*` 或 `RequestUI`/`UIResult` 链路与等待状态配对。

## Dependencies

- 被 `runtime/executor` 用作降级目标。
- 被状态层复用部分结构化类型，如 `Position`。
- 通过 `serde` 支持序列化，便于存档与测试。

## Invariants

- Runtime 与 Host 的边界收敛在 `Command` 层。
- `Transition` 只承载结构化参数，不解释具体效果语义。
- 对话命令保留 `inline_effects` / `no_wait`，文本模式切换通过 `SetTextMode` 显式表达。
- 需要宿主交互的通用入口是 `RequestUI`；完整会话重置通过 `FullRestart` 交给 Host 处理。

## FailureModes

- Runtime 扩展了命令字段或语义，但 Host 未同步处理。
- `Transition` 参数名或取值约定变化，导致效果解析偏差。
- `RequestUI` 的 `key` / `params` 协议变更后，Host 与 Runtime 不再对齐。

## WhenToReadSource

- 需要新增命令或调整字段结构时。
- 需要核对某个等待信号常量绑定哪条恢复链路时。
- 排查 Runtime 与 Host 行为不一致时。

## RelatedDocs

- [模块总览](../vn-runtime.md)
- [script 语义入口](script.md)

## LastVerified

2026-03-24

## Owner

GPT-5.4