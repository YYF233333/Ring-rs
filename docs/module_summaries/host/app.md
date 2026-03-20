# host/app 摘要

## Purpose

`app` 负责 Host 应用层编排：初始化子系统、维护 `AppState`、组织每帧更新与绘制、承接存档与脚本加载流程。

## PublicSurface

- 模块入口：`host/src/app/mod.rs`
- 初始化：`AppInit { headless, event_stream_path }`，`AppState::new(config, init_params: AppInit)`
- 关键类型：`AppState`、`CoreSystems`、`UiSystems`、`GameSession`、`ExtensionRegistry`（已从 `CoreSystems` 提升至 `AppState` 顶层）
- `AppState` 顶层字段含 `event_stream: EventStream`（结构化调试事件流）
- 状态/配置子模块：`app_mode`（AppMode/NavigationStack/UserSettings）、`state`（HostState）、`persistent`（PersistentStore）
- 关键子模块：`bootstrap`、`init`、`draw`、`save`、`script_loader`、`update`、`command_handlers`
- 导出：`export_recording(app_state)`（F8 导出录制缓冲区为 JSON Lines）
- 相关独立模块：`event_stream`（`host/src/event_stream/mod.rs`）、headless 入口（`host/src/headless.rs`）

## KeyFlow

1. `AppState::new(config, init_params)` 根据 `AppInit.headless` 创建 headless 或带设备音频，根据 `event_stream_path` 初始化 `EventStream`；非 headless 且 `debug.recording_buffer_size_mb > 0` 时启用输入录制。
2. `update` 路径推进输入、Runtime tick、命令执行和过渡/动画系统。
3. `draw` 路径将当前 `RenderState` 交给 `Renderer` 输出画面。
4. `export_recording(app_state)` 将输入管理器的录制快照导出到 `config.debug.recording_output_dir`。
5. `save` 与 `script_loader` 提供会话存档与脚本加载辅助能力（阶段 0 新增 callScript 可达脚本预注册）。

## Dependencies

- 依赖 `renderer`、`resources`、`audio`、`input`、`ui`（含 ScreenDefinitions）、`save_manager`
- 依赖 `vn-runtime` 提供脚本执行核心与等待模型

## Invariants

- `AppState` 是 Host 主循环的状态聚合根，子系统职责分层明确（core/ui/session）。`ExtensionRegistry` 位于 `AppState` 顶层而非 `CoreSystems` 内部。
- `CoreSystems` 实现 `EngineServices` trait（impl 位于 `engine_services.rs`），为 `extensions` 模块提供抽象访问入口。
- 脚本语义执行不在 `app` 内实现，只做编排与驱动。

## FailureModes

- 初始化阶段资源或配置加载失败，导致运行时降级或无法启动。
- 子系统状态未按阶段推进，导致 UI/渲染/输入状态不同步。

## WhenToReadSource

- 需要调整主循环阶段或插入新子系统时。
- 需要排查初始化顺序与运行时状态组装问题时。

## RelatedDocs

- [host 总览](../host.md)
- [app_update 摘要](app_update.md)
- [app_command_handlers 摘要](app_command_handlers.md)
- [仓库导航地图](../../navigation_map.md)

## LastVerified

2026-03-19

## Owner

Composer