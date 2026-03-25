# host-tauri/frontend

> LastVerified: 2026-03-25
> Owner: Claude

## 职责

Vue 3 前端渲染层——接收 Rust 后端通过 IPC 推送的 `RenderState` JSON，渲染视觉小说画面和系统 UI。

## 关键结构

### 入口与路由

| 文件 | 说明 |
|------|------|
| `main.ts` | 应用入口，全局错误捕获（Vue errorHandler + unhandledrejection） |
| `App.vue` | 根组件，页面路由（基于 `useNavigation` 的 Screen 状态机），键盘事件处理 |

### composables（状态管理层）

| composable | 说明 |
|------------|------|
| `useBackend` | 统一后端调用入口：Tauri 模式走 `invoke()`，浏览器模式走 HTTP fetch（debug server） |
| `useEngine` | 游戏引擎接口：startGame / tick 循环 / click / choose / 存档操作 |
| `useAssets` | 资源 URL 管理：获取 assets_root → `assetUrl(logicalPath)` 拼接可访问 URL |
| `useSettings` | 用户设置管理（单例）：load/save/update 与后端同步 |
| `useNavigation` | 页面导航状态机（单例）：Screen 枚举 + 栈式导航 |
| `useLogger` | 模块级日志：同时输出到 browser console 和 Rust tracing（通过 IPC 转发） |

### VN 渲染组件 (`vn/`)

| 组件 | 说明 |
|------|------|
| `VNScene` | VN 场景容器：组合背景、角色、对话、选择等子组件 |
| `BackgroundLayer` | 背景图渲染，处理 dissolve 过渡 |
| `CharacterLayer` | 角色立绘层，处理位置/透明度/过渡动画 |
| `DialogueBox` | 对话框，显示打字机效果文本（ADV/NVL 模式） |
| `ChoicePanel` | 分支选择面板 |
| `TransitionOverlay` | 场景过渡遮罩（fade/fadewhite/rule） |
| `TitleCard` | 字卡全屏文字显示 |
| `ChapterMark` | 章节标记淡入淡出 |
| `VideoOverlay` | 视频过场播放 |

### 系统 UI 组件 (`screens/`)

| 组件 | 说明 |
|------|------|
| `TitleScreen` | 标题画面（新游戏/继续/读取/设置/退出） |
| `SaveLoadScreen` | 存档/读取界面（复用，通过 mode prop 区分） |
| `SettingsScreen` | 设置界面（音量/文字速度/Auto 延迟等） |
| `HistoryScreen` | 对话历史回看 |
| `InGameMenu` | 游戏内菜单（右键/ESC 呼出） |

### 通用 UI 组件 (`components/`)

| 组件 | 说明 |
|------|------|
| `Toast` | 全局消息提示 |
| `ConfirmDialog` | 确认对话框（Promise 风格） |
| `SkipAutoIndicator` | Skip/Auto 模式指示器 |

## 数据流

```
useEngine.startGame()
  └─ callBackend("init_game") → renderState.value = response
       │
       ▼
     gameLoop() [requestAnimationFrame 驱动]
       └─ callBackend("tick", {dt}) → renderState.value = response
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
            └─ VideoOverlay ← cutscene → @finished → finish_cutscene
```

### 资源 URL 解析

```
RenderState.current_background = "images/bg01.png" (逻辑路径)
  │
  ▼
useAssets.assetUrl("images/bg01.png")
  ├─ Tauri 模式 → convertFileSrc(assetsRoot + "/images/bg01.png")
  │   → "asset://localhost/F:/Code/Ring-rs/assets/images/bg01.png"
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

- 前端**不持有**独立游戏逻辑状态——RenderState 是唯一数据源
- 游戏循环由前端 `requestAnimationFrame` 驱动，每帧调用后端 `tick`
- `callBackend` 透明切换 Tauri IPC / Debug HTTP，前端逻辑无需关心运行环境
- `useAssets` 的 `assetUrl()` 确保逻辑路径到可访问 URL 的统一转换
- `useLogger` 同时输出到 console 和 Rust tracing，调试时两端日志统一

## 与其他模块的关系

| 模块 | 关系 |
|------|------|
| `commands.rs` | 调用：所有 callBackend 对应 Rust #[command] |
| `render_state.rs` | 消费：接收 JSON 并按 TS 类型渲染 |
| `types/render-state.ts` | 依赖：TypeScript 类型定义必须与 Rust 侧同步 |
| `debug_server.rs` | 可选后端：浏览器调试模式的 HTTP API |
