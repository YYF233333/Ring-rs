# host/game_mode 摘要

## Purpose

`game_mode` 提供小游戏运行通道：通过本地 HTTP Bridge 服务静态资源与引擎 API，再用 WebView 把小游戏嵌入宿主窗口。

## PublicSurface

- 入口：`host/src/game_mode/mod.rs`
- 生命周期类型：`GameMode`、`GameModeState`、`PendingGameLaunch`、`GameCompletion`
- Bridge 类型：`BridgeServer`

## KeyFlow

1. 脚本侧 `call_game` 先在 `AppState.pending_game_launch` 中登记待启动请求。
2. `host_app` 在下一帧消费该请求，启动 `BridgeServer` 并创建 WebView。
3. `BridgeServer::poll()` 同时处理静态资源请求和 `/v1/*` API，请求内容主要分为 info、audio、state、complete、log 五类。
4. 小游戏完成后，`host_app` 关闭 WebView/Bridge，并把结果作为 `RuntimeInput::UIResult` 回传给 Runtime。

## Invariants

- Bridge 只绑定 `127.0.0.1`，作用域仅限本机。
- `call_game` 走 WebView + HTTP Bridge 路径，不属于 `ui_modes`。
- `BridgeServer` 只在小游戏活跃期间存活，结束后即销毁。

## WhenToReadSource

- 需要新增 Bridge API、修改 JS SDK 或调整 WebView 生命周期时。
- 需要排查小游戏结果回传、资源服务或音频桥接时。
- 需要确认某个 `requestUI` 该走 `call_game` 还是 `ui_modes` 时。

## RelatedDocs

- [host_app 摘要](host-app.md)
- [app_update 摘要](app-update.md)
- [ui_modes 摘要](ui-modes.md)
- [RFC-023：HTTP Bridge API](../../../../../RFCs/Accepted/rfc-http-bridge-api.md)

## LastVerified

2026-03-24

## Owner

GPT-5.4
