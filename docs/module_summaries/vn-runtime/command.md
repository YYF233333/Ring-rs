# vn-runtime/command 摘要

## Purpose

定义 Runtime 到 Host 的通信契约。`Command` 只声明“做什么”，不包含“怎么做”。

## PublicSurface

- 文件入口：`vn-runtime/src/command/mod.rs`
- 核心类型：`Command`、`Transition`、`TransitionArg`、`Choice`、`Position`、`InlineEffect`、`InlineEffectKind`
- `SignalId`：newtype（`pub struct SignalId(String)`，`serde(transparent)`），从 `input.rs` 重导出。`SIGNAL_*` 常量保持 `&str`，构造通过 `SignalId::new()`。

## KeyFlow

1. `Executor` 根据 `ScriptNode` 构造 `Command`。
2. `VNRuntime::tick` 按执行顺序返回 `Vec<Command>`。
3. Host 侧执行器将命令映射到渲染/音频/UI 系统。
4. `Command` 包含 `SceneEffect { name, args }`、`TitleCard { text, duration }` 等 variants。
5. 信号常量 `SIGNAL_SCENE_EFFECT`、`SIGNAL_TITLE_CARD` 用于场景效果与标题卡等待。
6. `InlineEffect` / `InlineEffectKind` 定义内联节奏标签数据模型（字符位置 + 效果类型：Wait/SetCpsAbsolute/SetCpsRelative/ResetCps）。
7. `Command::ShowText` 扩展 `inline_effects` 和 `no_wait` 字段；新增 `Command::ExtendText` variant。
8. `Command::BgmDuck` / `Command::BgmUnduck`：BGM 音量临时压低与恢复（即时指令，不产生等待态）。

## Dependencies

- 被 `runtime/executor` 大量使用。
- 被 `state` 引用（角色位置状态存储 `Position`）。
- 通过 serde 支持序列化/反序列化（便于存档与测试）。

## Invariants

- Runtime 与 Host 的语义边界在 `Command` 层收敛。
- `Transition` 参数仅做结构化，不解释具体效果语义。
- 任何新增脚本语义，若影响宿主行为，通常需要新增/扩展 `Command`。
- `Command::FullRestart`：Host 收到后负责持久化 `persistent_variables`、清空会话并返回标题；`CommandExecutor` 层是 no-op，由 `run_script_tick` 拦截处理。

## FailureModes

- 命令字段语义变更但 Host 未同步，可能出现行为偏差。
- Transition 参数命名约定不一致，导致宿主侧效果解析失败。

## WhenToReadSource

- 增加新命令或调整字段结构时。
- 排查 Runtime 与 Host 行为不一致时。

## RelatedDocs

- [模块总览](../vn-runtime.md)
- [仓库导航地图](../../navigation_map.md)
- [script 语义入口](script.md)

## LastVerified

2026-03-15

## Owner

Ring-rs 维护者
