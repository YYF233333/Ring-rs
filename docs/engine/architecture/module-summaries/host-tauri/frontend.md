# host-tauri/frontend

> LastVerified: 2026-03-30
> Owner: GPT-5.4

## 职责

Vue 3 前端渲染层——接收 Rust 后端通过 IPC 推送的 `RenderState` JSON，渲染视觉小说画面和系统 UI。

## 关键结构

### 入口与路由

| 文件 | 说明 |
|------|------|
| `main.ts` | 应用入口，全局错误捕获（Vue errorHandler + unhandledrejection） |
| `App.vue` | 根组件，页面状态投影（本地 `Screen` / `showInGameMenu` 与后端 `host_screen` 双向同步）、键盘事件处理；监听 `VNScene` 的 **`ui-result`** → 调用 **`useEngine.submitUiResult`** |

### composables（状态管理层）

| composable | 说明 |
|------------|------|
| `useBackend` | 统一后端调用入口：Tauri 模式走 `invoke()`，浏览器模式走 HTTP fetch（debug server） |
| `useEngine` | **模块级单例**（共享 `renderState` / 游戏循环 / `clientToken`）：`frontendConnected` 先领取 owner token；随后 `startGame`、`startGameAtLabel`、`tick`、`setHostScreen`、`setPlaybackMode` 等推进类调用都会自动携带 token；`playbackMode` 完全以后端 `RenderState` 为真源 |
| `useConfirmDialog` | 模块级确认框：`ask(title, message)` 返回 `Promise<boolean>`，与 `ConfirmDialog.vue` 配合 |
| `useAssets` | 资源 URL 管理：`assetUrl(logicalPath)` 通过 `ring-asset` 自定义协议（Tauri）或 debug HTTP server（浏览器）生成可访问 URL |
| `useSettings` | 用户设置管理（单例）：load/save/update 与后端同步 |
| `useNavigation` | 页面导航状态机（单例）：Screen 枚举 + 栈式导航 |
| `useLogger` | 模块级日志：同时输出到 browser console 和 Rust tracing（通过 IPC 转发） |

### VN 渲染组件 (`vn/`)

| 组件 | 说明 |
|------|------|
| `VNScene` | VN 场景容器：组合背景、角色、对话、选择等子组件；挂载 `MapOverlay`、`MiniGameOverlay`；**emit** `ui-result`（key + value），由根组件调用 `submitUiResult` |
| `BackgroundLayer` | 背景图渲染，处理 dissolve 过渡 |
| `CharacterLayer` | 角色立绘层；使用后端下发的 `pos_x`、`pos_y`、`anchor_x`、`anchor_y`、`render_scale`（manifest 解析结果）做 CSS 定位与缩放，不再使用前端硬编码的 `positionMap` |
| `DialogueBox` | 对话框，显示打字机效果文本（ADV/NVL 模式） |
| `ChoicePanel` | 分支选择面板 |
| `TransitionOverlay` | 场景过渡遮罩（fade/fadewhite/rule） |
| `TitleCard` | 字卡全屏文字显示 |
| `ChapterMark` | 章节标记淡入淡出 |
| `VideoOverlay` | 视频过场播放；**纯 emit**（如完成/跳过）由父组件接 `useEngine.finishCutscene` |
| `MapOverlay` | 地图 UI 模式（`active_ui_mode` 为地图类 mode 时展示） |
| `MiniGameOverlay` | iframe 小游戏容器（`active_ui_mode` 为小游戏 mode 时展示；先读取游戏 HTML、注入 `base href` 后用 `srcdoc` 加载，保证 `ring-asset` 下的相对资源路径稳定） |

### 系统 UI 组件 (`screens/`)

| 组件 | 说明 |
|------|------|
| `TitleScreen` | 标题画面（新游戏/继续/读取/设置/退出）；**纯 emit**，由 `App.vue` 调用 `useEngine` |
| `SaveLoadScreen` | 存档/读取界面（复用，通过 mode prop 区分） |
| `SettingsScreen` | 设置界面（音量/文字速度/Auto 延迟等） |
| `HistoryScreen` | 对话历史回看；**纯 emit** 请求数据，由父组件 `useEngine.getHistory` 拉取并传入 |
| `InGameMenu` | 游戏内菜单（右键/ESC 呼出） |

### 通用 UI 组件 (`components/`)

| 组件 | 说明 |
|------|------|
| `Toast` | 全局消息提示 |
| `ConfirmDialog` | 确认对话框（Promise 风格） |
| `SkipAutoIndicator` | Skip/Auto 模式指示器 |

### 静态资源 (`public/`)

| 文件 | 说明 |
|------|------|
| `engine-sdk.js` | 小游戏 JS SDK：与宿主页面 `postMessage` 协议交互（仓库中保留 `public/engine-sdk.js` 源文件，并提供 `assets/engine-sdk.js` 可加载副本供 iframe 小游戏引用） |

## 数据流

```
frontendConnected()
  └─ callBackend("frontend_connected") → { client_token, render_state }
       │
       ▼
useEngine.startGame()
  └─（内部）callBackend("init_game", { clientToken, scriptPath }) → 模块级 renderState = response
       │
       ▼
     gameLoop() [requestAnimationFrame 驱动]
      └─ callBackend("tick", { clientToken, dt }) → 更新同一模块级 renderState
            │
            ▼
          App.vue 将 renderState 作为 prop 传入 VNScene
            │
            ▼
          VNScene 分发到子组件渲染各字段
            ├─ BackgroundLayer ← current_background, background_transition
            ├─ CharacterLayer ← visible_characters
            ├─ DialogueBox ← dialogue, text_mode, nvl_entries
            ├─ ChoicePanel ← choices → @choose → handleChoose(index)
            ├─ TransitionOverlay ← scene_transition
            ├─ TitleCard ← title_card
            ├─ ChapterMark ← chapter_mark
            ├─ VideoOverlay ← cutscene → @finished → 父组件 useEngine.finishCutscene
            └─ MapOverlay / MiniGameOverlay ← active_ui_mode → @ui-result → App.vue submitUiResult
```

### 资源 URL 解析

```
RenderState.current_background = "images/bg01.png" (逻辑路径)
  │
  ▼
useAssets.assetUrl("images/bg01.png")
  ├─ Tauri 模式 → convertFileSrc("images/bg01.png", "ring-asset")
  │   → "ring-asset://localhost/images/bg01.png"（macOS/Linux）
  │   → "http://ring-asset.localhost/images/bg01.png"（Windows）
  │   协议 handler 内部通过 ResourceManager 读取（FS/ZIP 透明）
  └─ Debug 模式 → "http://localhost:9528/assets/images/bg01.png"
```

### 页面导航状态机

```
Screen 类型: "title" | "ingame" | "save" | "load" | "settings" | "history"

导航方式: 栈式 (screenStack)
  navigateTo(screen) → push 当前 → 切换
  goBack() → pop → 恢复
  resetToTitle() → 清栈 → "title"
  resetToIngame() → 清栈 → "ingame"

注意：上述 `Screen` 只负责 UI 编排；真正决定后端能否推进的是 `RenderState.host_screen`。`App.vue` 通过 watcher 将二者保持同步。
```

### 键盘快捷键 (ingame 模式)

| 按键 | 行为 |
|------|------|
| 左键点击 | handleClick() |
| 右键 / Escape | 切换 InGameMenu |
| Ctrl (按住) | 进入 Skip 模式，松开恢复 Normal |
| A | 切换 Auto 模式 |
| Backspace | 回退快照 |

## 关键不变量

- 前端**不持有**独立游戏逻辑状态——`RenderState` 是唯一数据源（由 `useEngine` 模块单例持有 ref）
- 游戏循环由前端 `requestAnimationFrame` 驱动，但每帧必须携带 `clientToken`；owner 被抢占后旧客户端会收到显式错误并停止 loop
- `playbackMode` 完全以后端 `render_state.playback_mode` 为真源，本地不再维护独立 authoritative 副本
- `currentScreen` / `showInGameMenu` 只是后端 `host_screen` 的 UI 投影；非 `InGame` 屏幕下不会继续推进剧情
- **业务侧后端调用经 `useEngine` 聚合**；`App.vue` 与 VN/系统 UI 组件不直接 `callBackend`（底层仍由 `useEngine` / `useBackend` 统一走 IPC 或 Debug HTTP）
- `callBackend` 透明切换 Tauri IPC / Debug HTTP，`useBackend` 使用者无需关心运行环境
- `useAssets` 的 `assetUrl()` 确保逻辑路径到可访问 URL 的统一转换
- `useLogger` 同时输出到 console 和 Rust tracing，调试时两端日志统一

## 与其他模块的关系

| 模块 | 关系 |
|------|------|
| `commands.rs` | 调用：业务经 `useEngine` → `callBackend` 对应 Rust `#[command]` |
| `render_state.rs` | 消费：接收 JSON 并按 TS 类型渲染 |
| `types/render-state.ts` | 依赖：TypeScript 类型定义必须与 Rust 侧同步 |
| `debug_server.rs` | 可选后端：浏览器调试模式的 HTTP API |
