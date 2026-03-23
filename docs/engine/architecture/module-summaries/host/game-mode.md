# host/game_mode 摘要

## Purpose

小游戏模式管理，通过 HTTP Bridge + WebView 嵌入 HTML5 小游戏。

## PublicSurface

- `host/src/game_mode/mod.rs`：模块入口
- `lifecycle.rs`：`GameMode`（状态机）、`GameModeState`、`PendingGameLaunch`、`GameCompletion`、`GameModeError`
- `http_bridge.rs`：`BridgeServer`（HTTP 服务器）、`BridgeServerError`、`js_sdk_init_script()`
- `bridge.rs`：`BridgeValue`（JSON 值类型）、`BridgeResponse`（API 响应格式）

## KeyFlow

1. `BridgeServer::start(game_dir)` 启动 HTTP 服务器（`127.0.0.1:0` 随机端口），验证游戏资源
2. `GameMode::start()` 创建 WebView，URL 指向 HTTP 服务器，注入 JS SDK
3. 每帧 `bridge.poll(app_state)` 轮询：静态资源请求 → 文件服务；`/v1/*` API 请求 → 处理音频/状态/日志/完成
4. `GameCompletion` 返回后 → 销毁 WebView → drop `BridgeServer` → `RuntimeInput::UIResult` 回传

## API 端点

- `/v1/info`
- `/v1/audio/play-sound`
- `/v1/audio/play-bgm`
- `/v1/audio/stop-bgm`
- `/v1/state/get`
- `/v1/state/set`
- `/v1/complete`
- `/v1/log`

## Dependencies

- `tiny_http`、`wry`
- `vn_runtime::state`
- `crate::app::AppState`（音频、Runtime）

## Invariants

- HTTP 服务器仅绑定 `127.0.0.1`，不暴露外部网络
- Headless 模式不启动 `BridgeServer`
- API 版本化前缀 `/v1/`，冻结后仅允许新增可选字段

## FailureModes

- HTTP 端口绑定失败
- 游戏资源目录不存在
- WebView 创建失败

## WhenToReadSource

需要新增 API 端点、修改 JS SDK、调整 WebView 生命周期时

## RelatedDocs

- [RFC-023：HTTP Bridge API](../../../../../RFCs/rfc-http-bridge-api.md)
- [host 总览](../host.md)
- [host_app 摘要](host-app.md)

## LastVerified

2026-03-23

## Owner

claude-4.6-opus
