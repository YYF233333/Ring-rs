# host/ui_modes 摘要

## Purpose

`ui_modes` 提供 UI 模式插件系统，通过 `UiModeHandler` trait 和 `UiModeRegistry` 实现自定义 UI 模式的注册与运行时调度。新增 UI 模式只需实现 trait 并注册，无需修改核心代码。

## PublicSurface

- 模块入口：`host/src/ui_modes/mod.rs`
- 核心类型：
  - `UiModeHandler` trait：UI 模式处理器接口（mode_id / activate / render / deactivate）
  - `UiModeRegistry`：注册表与运行时调度（register / activate / take_active / restore_active / complete_active / cancel_current）
  - `UiModeStatus`：渲染帧返回状态（Active / Completed / Cancelled）
  - `UiModeError`：模式错误（UnknownMode / AlreadyActive / ResourceLoadFailed / InvalidParams）
  - `ActiveUiMode`：从 registry 中取出的活跃模式（临时持有用于渲染）
- 内置 handler：
  - `map_handler::MapModeHandler`：地图 UI 模式（mode_id="show_map"），支持背景图 + 颜色掩码命中检测

## KeyFlow

1. `script.rs` 收到 `Command::RequestUI` 时，非 `call_game` 的模式通过 `registry.activate()` 路由
2. 帧循环中 `take_active()` 取出活跃 handler，调用 `handler.render()` 渲染
3. 渲染返回 `Completed(value)` 或 `Cancelled` 时，通过 `complete_active()` 归还并注入 `RuntimeInput::UIResult`
4. 仍在活跃时通过 `restore_active()` 归还

## Dependencies

- `egui`（UI 渲染）
- `image`（背景图/掩码图解码）
- `vn_runtime::state::VarValue`（参数/结果值）
- `resources::ResourceManager`（资源加载）

## Invariants

- 同一时刻只有一个 UI 模式活跃
- 活跃 handler 在渲染期间通过 take/restore 模式临时移出 registry，避免借用冲突
- `call_game`（WebView 小游戏）不走 registry，保持独立处理路径
- handler 在 activate 失败时自动归还到 handlers map

## WhenToReadSource

- 需要新增 UI 模式时（实现 UiModeHandler trait）
- 需要修改 UI 模式生命周期或渲染集成时

## RelatedDocs

- [host 总览](../host.md)
- [app 摘要](app.md)
- [app_update 摘要](app-update.md)
- [RFC: UI Mode Plugin System](../../../../../RFCs/rfc-ui-mode-plugin-system.md)

## LastVerified

2026-03-24

## Owner

claude-4.6-opus
