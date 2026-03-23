# host/ui_modes 摘要

## Purpose

`ui_modes` 提供 `requestUI` 的插件化 UI 模式机制，使脚本可在不改核心页面分发逻辑的前提下激活自定义交互界面。

## PublicSurface

- 入口：`host/src/ui_modes/mod.rs`
- 核心类型：`UiModeHandler`、`UiModeRegistry`、`UiModeStatus`、`UiModeError`
- 内建 handler：`map_handler::MapModeHandler`

## KeyFlow

1. `AppState::new()` 创建 `UiModeRegistry` 并注册内建 `show_map` handler。
2. `app/update/script.rs` 处理 `Command::RequestUI` 时，除 `call_game` 外都通过 `registry.activate()` 进入这里。
3. 窗口模式与 `headless` 都使用 `take_active()` -> `render()` -> `restore_active()/complete_active()` 的同一渲染协议。
4. handler 完成或取消后，调用方把结果注入 `RuntimeInput::UIResult` 回到 Runtime。

## Invariants

- 同一时刻只能有一个活跃 UI mode。
- 活跃 handler 在渲染期临时移出 registry，以避免借用冲突。
- `call_game` 走小游戏 WebView 路径，不走 `UiModeRegistry`。

## WhenToReadSource

- 需要新增新的 `requestUI mode` 时。
- 需要修改 UI mode 生命周期或完成结果回传时。
- 需要区分 `ui_modes` 与普通 egui 页面边界时。

## RelatedDocs

- [app_update 摘要](app-update.md)
- [host_app 摘要](host-app.md)
- [egui_screens 摘要](egui-screens.md)
- [RFC: UI Mode Plugin System](../../../../../RFCs/rfc-ui-mode-plugin-system.md)

## LastVerified

2026-03-24

## Owner

GPT-5.4
