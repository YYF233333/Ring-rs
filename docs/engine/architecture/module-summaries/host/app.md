# host/app 摘要

## Purpose

`app` 是宿主应用层的组装点：负责构建 `AppState`，并把初始化、更新、绘制、存档、脚本加载与副作用处理串成一条稳定主链路。

## PublicSurface

- 入口：`host/src/app/mod.rs`
- 核心类型：`AppInit`、`AppState`、`CoreSystems`、`UiSystems`、`GameSession`
- 关键导出：`update`、绘制/脚本加载/存档辅助函数、`export_recording`
- 子模块：`snapshot`（快照栈与 Backspace 回退编排）

## KeyFlow

1. `AppState::new(config, init_params)` 创建资源、渲染状态、音频、UI 配置、存档、事件流与内建扩展。
2. `AppState` 以 `core` / `ui` / `session` 三组状态承载主循环需要的可变状态。
3. `update`、`draw`、`save`、`script_loader`、`command_handlers` 分别承担每帧推进、绘制、存档、脚本切换与副作用消费。
4. `ui_mode_registry` 在初始化时注册内建 `show_map` handler；小游戏请求与事件流保留在 `AppState` 顶层。
5. `export_recording` 导出输入录制缓冲区；panic unwind 时 `Drop` 会尝试自动导出。
6. `snapshot_stack` 作为 `AppState` 顶层字段维护快照栈，供每帧更新链路在 Backspace 时回退。

## Invariants

- `AppState` 是 Host 主循环的聚合根，跨帧状态应挂在这里或其三组子状态中。
- `app` 负责编排，不实现脚本语义本身。
- `extension_registry`、`event_stream`、`ui_mode_registry` 属于跨子系统协调能力，保留在 `AppState` 顶层。
- 快照栈不写入存档；加载存档或新开游戏后须清空。

## WhenToReadSource

- 需要新增主循环共享状态或新子系统挂载点时。
- 需要排查初始化顺序、headless/windowed 组装差异时。
- 需要修改录制导出、小游戏启动请求或 UI mode 注册时。

## RelatedDocs

- [host 总览](../host.md)
- [app_update 摘要](app-update.md)
- [app_command_handlers 摘要](app-command-handlers.md)
- [host_app 摘要](host-app.md)
- [仓库导航地图](../../navigation-map.md)

## LastVerified

2026-03-24

## Owner

claude-4.6-opus
