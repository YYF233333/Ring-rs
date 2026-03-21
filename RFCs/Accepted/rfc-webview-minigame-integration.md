# RFC: WebView 小游戏集成

## 元信息

- 编号：RFC-021
- 状态：Accepted
- 作者：claude-4.6-opus
- 日期：2026-03-21
- 相关范围：`host`（game_mode / app / command_executor）、`vn-runtime`（command / script）
- 前置：RFC-020（双向 UI-Script 通信协议）

---

## 背景

AVG 游戏在市场竞争中越来越需要加入非 AVG 元素提升可玩性，从简单的小游戏（卡牌、乒乓球）到复杂的动作游戏（类 Vampire Survivors）。当前引擎的 egui 技术栈不适合实现复杂的实时游戏逻辑和渲染。

通过概念验证 spike 确认：
- wry 0.49 可正常编译并嵌入 winit 窗口（Windows WebView2）

Vampire Survivors 原版使用 Phaser (HTML5/Canvas) 构建，证明 WebView 可以处理足够复杂度的游戏。

---

## 目标与非目标

### 目标

- 通过 wry 嵌入 WebView，支持在 VN 脚本中启动 HTML5 小游戏
- 定义 JS Bridge 协议，让小游戏可访问引擎服务（音频、状态、资源）
- 小游戏结束后结果回传脚本（复用 RFC-020 的 UIResult 机制）
- 优雅降级：WebView 不可用时（如 headless 模式）给出明确提示，不阻塞 VN 主线

### 非目标

- 具体小游戏的实现（由游戏开发者用 HTML5 技术栈开发）
- 移动平台（Android/iOS）适配（wry 支持但需额外工程）
- 小游戏状态存档集成（Phase 4）
- JS SDK / 开发模板（Phase 4）

---

## 方案设计

### 架构概览

```
VN 脚本                    Host                         WebView (wry)
    |                        |                              |
    |--Cmd::RequestUI------->|                              |
    |  mode="call_game"      |--build_as_child(window)----->|
    |  params={game_id,...}  |  (加载 assets/games/xxx/)    |
    |                        |                              |
    |  WaitForUIResult       |                              |
    |                        |<--ipc: BridgeRequest---------|
    |                        |  (playSound, getState, ...)  |
    |                        |                              |
    |                        |---evaluate_script: response->|
    |                        |                              |
    |<-RuntimeInput::UIResult|<--ipc: onComplete(data)------|
    |  {result}              |--destroy WebView------------>|
```

### 模块结构

```
host/src/game_mode/
├── mod.rs          -- 模块入口，feature gate
├── lifecycle.rs    -- GameMode 状态机（Idle/Running）
└── bridge.rs       -- JS Bridge 协议（BridgeRequest/BridgeResponse）
```

### 脚本语法

```markdown
callGame "card_battle" as $result
callGame "pong" as $score (difficulty: 3, time_limit: 60)
```

`callGame` 为语法糖，解析为 `ScriptNode::RequestUI { mode: "call_game", ... }`。

### 生命周期

1. 脚本触发 `callGame "game_id" { params }`
2. Host 收到 `Command::RequestUI { mode: "call_game", ... }`
3. Host 暂停 wgpu 渲染循环，创建 wry WebView 子窗口
4. WebView 加载 `assets/games/{game_id}/index.html`
5. 游戏通过 `window.ipc.postMessage()` 发送 `BridgeRequest`
6. Host 处理请求，通过 `evaluate_script()` 返回 `BridgeResponse`
7. 游戏调用 `onComplete(result)` 通知结束
8. Host 销毁 WebView，恢复 wgpu 渲染
9. 结果通过 `RuntimeInput::UIResult` 回传 Runtime

### JS Bridge 协议

#### Engine → Game（注入 JS API）

```javascript
window.engine = {
    playSound(name) { ... },
    playBGM(name, opts) { ... },
    getState(key) { ... },
    setState(key, value) { ... },
    getAssetUrl(path) { ... },
    log(level, message) { ... },
    onComplete(resultData) { ... },
};
```

#### Game → Engine（IPC 消息）

使用 `BridgeRequest` 枚举（serde tagged union）：

```json
{"type": "playSound", "name": "hit.mp3"}
{"type": "getState", "key": "player_hp"}
{"type": "onComplete", "result": {"score": 100}}
```

#### Engine → Game（IPC 响应）

使用 `BridgeResponse` 结构：

```json
{"success": true, "data": 42}
{"success": false, "error": "variable not found"}
```

### 降级策略

**GUI 模式 WebView 创建失败**：立即回传空字符串 `UIResult`，脚本可通过结果变量判断。

**Headless 模式**：跳过 WebView 启动，由录制文件中的 `UIResult` 事件提供真实游戏结果，保证分支路径与录制时一致。录制系统通过 `InputEvent::UIResult` 变体自动捕获所有 UI 交互结果。

> **历史变更**：wry 依赖最初通过 `mini-games` feature gate 条件编译。经测试 feature 开关仅影响约 400KB 二进制体积，feature gate 已移除，wry 常开编译。

---

## 影响范围

| 模块 | 改动 | 风险 |
|------|------|------|
| `host/src/game_mode/` | 新增模块（已在 spike 中创建骨架） | 低：feature gate 隔离 |
| `host/Cargo.toml` | 新增 wry 可选依赖（已完成） | 低：不影响默认编译 |
| `vn-runtime/parser` | 新增 `callGame` 语法糖 | 低：复用 RequestUI 节点 |
| `host/src/app/update/script.rs` | 处理 RequestUI mode="call_game" | 中：需要与 wgpu 渲染循环协调 |

---

## 迁移计划

纯新增功能，无破坏性变更。

---

## 验收标准

- [ ] `callGame "game_id" as $result` 语法正确解析
- [ ] wry 正常编译（常开依赖）
- [ ] GameMode 状态机正确管理 Idle/Running 转换
- [ ] JS Bridge 协议类型定义完整（BridgeRequest / BridgeResponse）
- [ ] Headless 模式下 callGame 正确降级（立即返回空结果）
- [ ] `cargo check-all` 通过
