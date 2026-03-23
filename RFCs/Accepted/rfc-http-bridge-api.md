# RFC: HTTP Bridge API

## 元信息

- 编号：RFC-023
- 状态：Implemented
- 作者：claude-4.6-opus
- 日期：2026-03-23
- 相关范围：`host`（game_mode / host_app）、`assets/games/`
- 前置：RFC-021（WebView 小游戏集成）

---

## 背景

RFC-021 通过 wry IPC（`window.ipc.postMessage`）实现了 WebView 小游戏与引擎的通信。当前实现存在以下问题：

### 向后兼容困难

IPC 协议由 `BridgeRequest` enum（`bridge.rs`）定义，任何字段变更都直接破坏 JSON 解析。没有版本协商机制——旧游戏发送的 JSON 格式若与新引擎的 `BridgeRequest` 不匹配，`serde_json::from_str` 直接报错。

### 单向通信

`window.ipc.postMessage` 是单向的——JS 发出消息后无法同步获取响应。当前仅 `onComplete` 和 `log` 两种消息（均为 fire-and-forget），但 `getState`、`getAssetUrl` 等需要返回值的 API 无法正常工作。

### 调试困难

IPC 消息在 Rust 侧以 closure 处理，无法用标准工具（curl、Postman、浏览器 Network 面板）调试。问题定位必须加 `tracing` 日志逐步排查。

### 已定义但无法接线的 API

`BridgeRequest` 中定义了 `PlaySound`、`PlayBgm`、`GetState`、`SetState`、`GetAssetUrl` 等变体，但因 IPC 单向限制和主线程状态访问限制，大部分尚未实际接线。

### custom protocol 维护负担

资源加载通过 wry `with_custom_protocol("game", ...)` 注册 `game://` 协议，需维护 `make_asset_handler` 闭包、`mime_from_ext` 函数，以及处理 Windows WebView2 将 `game://` 翻译为 `http://game.localhost/` 的平台差异。这套机制与通信管线完全独立，增加了理解和维护成本。

---

## 目标与非目标

### 目标

- 引入本地 HTTP 服务器，**统一承担静态资源服务与 Bridge API 两项职责**，取代 `game://` 自定义协议和 wry IPC
- 提供版本化 REST-like API（`/v1/...`），支持向后兼容演进
- 所有 Bridge 调用均为标准 HTTP 请求-响应模式，JS 侧可 `await fetch()` 同步获取结果
- 接线当前已定义但未工作的 Bridge 能力（音频、状态读写等）
- 提供 JS SDK（init_script 注入），封装 HTTP 调用为 `window.engine.*` API
- HTTP 服务仅在小游戏运行期间启动，游戏结束后关闭
- 网络通信仅限 `127.0.0.1`，不暴露到外部网络，避免杀毒软件误报

### 非目标

- 非 WebView 客户端的正式支持（HTTP 天然支持，但不作为设计目标）

---

## 方案设计

### 架构概览

```
游戏启动                          运行时                        游戏结束
    │                               │                             │
    ▼                               ▼                             ▼
启动 tiny_http::Server          每帧: server.try_recv()        关闭 Server
  on 127.0.0.1:0                  ├─ /v1/* → Bridge API 处理
  → 获得随机端口 PORT              ├─ 其他路径 → 静态资源服务
    │                              └─ 无请求 → 跳过
    ▼
创建 WebView
  URL: http://127.0.0.1:{PORT}/index.html
  init_script: window.engine.* SDK
    │
    ▼
Game JS:
  await engine.playSound("hit.mp3")
    → fetch(`http://127.0.0.1:${PORT}/v1/audio/play-sound`, {...})
    → HTTP 200 { "success": true }
```

页面和 API 均由同一 HTTP server 提供，属于**同源请求**，无需处理 CORS。

### 依赖

新增 `tiny_http` crate（纯同步、无 async 依赖、约 30KB 编译增量）。

```toml
# host/Cargo.toml
tiny_http = "0.12"
```

`tiny_http` 提供 `Server::try_recv() -> Option<Request>` 非阻塞接收，与 winit 主循环集成零摩擦。

### 请求路由

HTTP server 按路径前缀分流：

- `/v1/*` → Bridge API 处理（JSON 请求-响应）
- 其他路径 → 静态资源服务（从 `game_dir` 读取文件）

```rust
fn handle_request(request: tiny_http::Request, game_dir: &Path, app_state: &mut AppState) {
    let url = request.url().to_string();

    if url.starts_with("/v1/") {
        handle_api_request(request, &url, app_state);
    } else {
        serve_static_file(request, &url, game_dir);
    }
}
```

### 静态资源服务

替代原有 `make_asset_handler` + `game://` custom protocol：

```rust
fn serve_static_file(request: tiny_http::Request, url_path: &str, game_dir: &Path) {
    let relative = url_path.strip_prefix('/').unwrap_or(url_path);
    let file_path = game_dir.join(relative);

    let (data, mime, status) = if file_path.is_file() {
        let data = std::fs::read(&file_path).unwrap_or_default();
        let mime = mime_from_ext(file_path.extension().and_then(|e| e.to_str()));
        (data, mime, 200)
    } else {
        (b"Not Found".to_vec(), "text/plain", 404)
    };

    let response = tiny_http::Response::from_data(data)
        .with_status_code(status)
        .with_header(
            tiny_http::Header::from_bytes("Content-Type", mime).unwrap()
        );
    let _ = request.respond(response);
}
```

### API 端点定义

所有端点前缀 `/v1/`。请求体和响应体均为 JSON。

#### 通用响应格式

```json
{
  "success": true,
  "data": <any>,        // 可选
  "error": "<message>"  // 仅 success=false 时
}
```

#### 端点列表

| 方法 | 路径 | 请求体 | 响应 data | 说明 |
|------|------|--------|-----------|------|
| POST | `/v1/audio/play-sound` | `{ "name": "hit.mp3" }` | null | 播放音效 |
| POST | `/v1/audio/play-bgm` | `{ "name": "bg.mp3", "loop": true }` | null | 播放 BGM |
| POST | `/v1/audio/stop-bgm` | `{}` | null | 停止 BGM |
| POST | `/v1/state/get` | `{ "key": "player_hp" }` | `{ "value": 100 }` | 读取脚本变量 |
| POST | `/v1/state/set` | `{ "key": "player_hp", "value": 100 }` | null | 写入脚本变量 |
| POST | `/v1/complete` | `{ "result": <any> }` | null | 游戏结束，回传结果 |
| POST | `/v1/log` | `{ "level": "info", "message": "..." }` | null | 日志输出 |
| GET | `/v1/info` | — | `{ "version": "0.1.0", "api_version": "1" }` | 引擎信息 |

#### 版本演进策略

- `/v1/` 端点一旦发布即冻结，仅允许新增可选字段
- 破坏性变更必须新增 `/v2/` 前缀
- 多版本共存：同一 server 同时处理 `/v1/` 和 `/v2/` 请求

### 主线程集成

```rust
// host/src/game_mode/http_bridge.rs

pub struct BridgeServer {
    server: tiny_http::Server,
    port: u16,
    game_dir: PathBuf,
}

impl BridgeServer {
    pub fn start(game_dir: PathBuf) -> Result<Self, BridgeServerError> {
        let server = tiny_http::Server::http("127.0.0.1:0")
            .map_err(|e| BridgeServerError::BindFailed(e.to_string()))?;
        let port = server.server_addr().to_ip().unwrap().port();
        Ok(Self { server, port, game_dir })
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub fn try_recv(&self) -> Option<tiny_http::Request> {
        self.server.try_recv().ok().flatten()
    }
}
```

每帧轮询（在 `host_app.rs` 的帧驱动中）：

```rust
fn poll_bridge_requests(app_state: &mut AppState, bridge: &BridgeServer) {
    while let Some(request) = bridge.try_recv() {
        let url = request.url().to_string();

        if url.starts_with("/v1/") {
            let response = handle_api_request(&url, &request, app_state);
            let http_response = tiny_http::Response::from_string(
                serde_json::to_string(&response).unwrap()
            ).with_header(
                tiny_http::Header::from_bytes("Content-Type", "application/json").unwrap()
            );
            let _ = request.respond(http_response);
        } else {
            serve_static_file(request, &url, &bridge.game_dir);
        }
    }
}
```

### JS SDK

引擎在 WebView 初始化时注入 SDK：

```javascript
(function() {
    const BASE = location.origin + '/v1';

    async function call(endpoint, body = {}) {
        const resp = await fetch(`${BASE}/${endpoint}`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(body),
        });
        const data = await resp.json();
        if (!data.success) throw new Error(data.error || 'Bridge call failed');
        return data.data;
    }

    window.engine = {
        async playSound(name) {
            await call('audio/play-sound', { name });
        },
        async playBGM(name, loop = true) {
            await call('audio/play-bgm', { name, loop });
        },
        async stopBGM() {
            await call('audio/stop-bgm');
        },
        async getState(key) {
            const result = await call('state/get', { key });
            return result?.value;
        },
        async setState(key, value) {
            await call('state/set', { key, value });
        },
        async complete(result) {
            await call('complete', { result });
        },
        log(level, message) {
            call('log', { level, message }).catch(() => {});
        },
    };
})();
```

SDK 使用 `location.origin` 获取基地址，无需注入端口号。

游戏开发者使用 API：

```javascript
const hp = await engine.getState("player_hp");
await engine.playSound("hit.mp3");
await engine.setState("player_hp", hp - 10);
if (hp <= 0) {
    await engine.complete("game_over");
}
```

### 生命周期

```
Command::RequestUI { mode: "call_game" }
  │
  ├─ 1. BridgeServer::start(game_dir) → 获得 port
  ├─ 2. 构造 init_script（注入 SDK）
  ├─ 3. 创建 WebView（URL: http://127.0.0.1:{PORT}/index.html，无 custom protocol）
  ├─ 4. 每帧 poll_bridge_requests()
  │     ├─ /v1/* → Bridge API 处理
  │     │     └─ /v1/complete → 设置 GameCompletion
  │     └─ 其他 → 静态资源服务
  ├─ 5. 收到 complete → 销毁 WebView
  ├─ 6. 关闭 BridgeServer（drop）
  └─ 7. RuntimeInput::UIResult 回传 Runtime
```

### Headless 模式

Headless 模式下不启动 HTTP server（无 WebView），由 replay 提供 UIResult（与 RFC-021 一致）。

---

## 影响范围

| 模块 | 改动 | 风险 |
|------|------|------|
| `host/Cargo.toml` | 新增 `tiny_http` 依赖 | 低：轻量依赖 |
| `host/src/game_mode/http_bridge.rs` (新增) | BridgeServer + 请求路由 + 静态资源服务 | 中：新增核心模块 |
| `host/src/game_mode/mod.rs` | 导出 BridgeServer | 低 |
| `host/src/game_mode/lifecycle.rs` | 删除 `with_custom_protocol` / `with_ipc_handler`，改为 HTTP URL 加载；`start()` 接收 `BridgeServer` 的 port | 中：WebView 创建方式变更 |
| `host/src/game_mode/bridge.rs` | 删除 `BridgeRequest` enum（HTTP 路由替代 enum dispatch） | 低：移除代码 |
| `host/src/host_app.rs` | 帧循环中插入 poll_bridge_requests；管理 BridgeServer 生命周期 | 中：主循环变更 |
| `assets/games/demo_stub/game.js` | 迁移到 `window.engine.*` SDK | 低：示例代码 |

### 删除的代码

| 位置 | 删除内容 | 原因 |
|------|---------|------|
| `lifecycle.rs` | `make_asset_handler` 函数 | HTTP server 统一服务静态资源 |
| `lifecycle.rs` | `mime_from_ext` 函数 | 移至 `http_bridge.rs`（复用） |
| `lifecycle.rs` | `with_custom_protocol("game", ...)` 调用 | 不再需要 custom protocol |
| `lifecycle.rs` | `with_ipc_handler(...)` 闭包 | 不再需要 IPC 通信 |
| `bridge.rs` | `BridgeRequest` enum | HTTP 路由替代 tagged union dispatch |
| `game.js` | `postToEngine` 兼容层 | 直接使用 `window.engine.*` |

---

## 迁移计划

一次性替换，不保留 IPC 降级路径。

1. 添加 `tiny_http` 依赖
2. 实现 `BridgeServer`（启动/端口/try_recv/静态资源服务）
3. 实现请求路由（URL 路径前缀分流：`/v1/*` → API，其他 → 静态文件）
4. 在 `lifecycle.rs` 中：删除 custom protocol 和 IPC handler，改为 HTTP URL 加载
5. 在 `host_app.rs` 帧循环中添加 poll + 管理 BridgeServer 生命周期
6. 实现 JS SDK（init_script 注入）
7. 接线 API 端点：audio/play-sound、state/get、state/set、complete、log
8. 迁移 demo_stub 到 `window.engine.*` SDK
9. 删除 `bridge.rs` 中的 `BridgeRequest` enum（如不再需要）
10. 更新文档

---

## 验收标准

- [ ] `tiny_http` 依赖添加且编译通过
- [ ] BridgeServer 在 127.0.0.1:0 正常启动，获得随机端口
- [ ] WebView 通过 `http://127.0.0.1:{PORT}/index.html` 正确加载游戏页面
- [ ] 静态资源（JS/CSS/图片/音频）通过 HTTP server 正确服务
- [ ] JS SDK 正确注入 WebView，`window.engine.*` API 可用
- [ ] `engine.playSound()` 正确触发引擎音效播放
- [ ] `engine.getState()` 正确读取脚本变量并返回
- [ ] `engine.setState()` 正确写入脚本变量
- [ ] `engine.complete()` 正确触发游戏结束和 UIResult 回传
- [ ] API 版本前缀 `/v1/` 工作正常
- [ ] `game://` custom protocol 和 IPC handler 已移除
- [ ] Headless 模式下不启动 HTTP server
- [ ] demo_stub 迁移到 HTTP SDK 后功能正常
- [ ] `cargo check-all` 通过
- [ ] 模块摘要文档更新
