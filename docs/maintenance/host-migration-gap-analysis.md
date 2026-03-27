# 旧 Host → host-tauri 迁移差距分析

> 生成时间：2026-03-26
>
> 旧 Host 技术栈：winit + wgpu + egui + rodio + ffmpeg-sidecar + wry
>
> 新 Host 技术栈：Tauri + Vue.js + Web Audio API

## 一、总体架构差异

| 维度 | 旧 Host (`host/`) | 新 Host (`host-tauri/`) |
|------|----|----|
| 渲染 | wgpu GPU 渲染（SpriteRenderer + DissolveRenderer） | 前端 CSS/DOM 渲染 |
| UI 框架 | egui（即时模式 GUI） | Vue.js（声明式 Web UI） |
| 音频 | rodio（原生音频设备） | Web Audio API（前端播放，后端仅状态追踪） |
| 视频 | FFmpeg 子进程解码 RGBA 帧 → GPU 纹理 | HTML5 `<video>` 元素 |
| 输入 | winit 事件 + InputManager | 浏览器事件（前端处理） |
| 进程模型 | 单进程，渲染循环驱动 | Rust 后端 + WebView 前端，IPC 通信 |

---

## 二、已完成迁移

以下能力在 host-tauri 中已有对等实现（实现路径可能与旧 Host 不同，但功能已覆盖）。

### 2.1 功能对照表

| 特性 | 旧 Host 实现 | host-tauri 实现 |
|------|-------------|----------------|
| 命令执行（23 种 Command） | `command_executor/` 多文件 | `command_executor.rs` 单文件 |
| 背景显示/切换 | `ShowBackground` + 过渡 | 同上 + `BackgroundLayer.vue` |
| 场景切换（Fade/FadeWhite/Rule/Dissolve） | `ChangeScene` + `SceneTransitionManager` | `ChangeScene` + `TransitionOverlay.vue` |
| 角色立绘管理 | `ShowCharacter`（4 种策略） | 同逻辑 + `CharacterLayer.vue` |
| 对话/打字机 | egui 文本渲染 + `TypewriterTimer` | `DialogueBox.vue` + 后端打字机引擎 |
| 选择分支 | egui 按钮 | `ChoiceOverlay.vue` |
| 章节标记 | egui 渲染 + 淡入淡出动画 | `ChapterMark.vue` |
| 标题卡片 | `TitleCard` + `EffectRequest` | `TitleCard.vue` + 后端状态 |
| BGM/SFX 播放 | rodio 播放 | Web Audio API（`useAudio.ts`） |
| BGM Duck/Unduck | rodio 音量渐变 | 后端状态 + 前端 GainNode |
| BGM 交叉淡入淡出 | 路径切换时过渡 | 后端 `pending_transition` + `AudioRenderState.bgm_transition`；前端双 `GainNode` + `linearRampToValueAtTime` |
| BGM 淡入/淡出 | rodio 包络 | `stop_bgm(fade_out)` / `play_bgm` 静音启动 fade_in；前端 `GainNode` 包络 |
| 存档管理 | `SaveManager`（99 槽位 + Continue） | `save_manager.rs`（同） |
| 持久化变量 | `PersistentStore` | `PersistentStore`（`state.rs`） |
| 快照/回退 | `SnapshotStack` | `SnapshotStack`（`state.rs`） |
| 对话历史 | `HistoryEntry` | `HistoryEntry` + `get_history` |
| 播放模式 | `PlaybackMode`（Normal/Auto/Skip） | 同 |
| NVL 模式 | ADV/NVL 切换 | 同 + `nvl_entries` |
| Inline Effects | `{wait}` `{cps}` 等 | 后端打字机处理 |
| 配置加载 | `AppConfig` + 校验 | `config.rs` |
| Manifest（立绘组/站位预设） | `manifest/` | `manifest.rs` |
| ZIP 资源来源 | `ResourceManager` + 多来源 | `ResourceSource` + `FsSource` / `ZipSource`；`AppConfig.asset_source` 切换；`ZipSource` 为 `Mutex<ZipArchive>` + 路径索引 |
| 数据驱动 UI | `screens.json` + `layout.json` | IPC 命令返回给前端 |
| FullRestart | 重置会话 | `reset_session()` |
| Cutscene（视频过场） | FFmpeg 解码 | `VideoOverlay.vue`（HTML5 video） |
| Debug 调试 | 事件流（JSONL） | Debug HTTP Server（`debug_server.rs`），方式不同 |
| 子脚本预加载 | 启动时递归扫描 | `preload_called_scripts()` DFS |

### 2.2 场景效果（与旧 Host 对齐部分）

| 子项 | 说明 |
|------|------|
| Shake | 后端 `update_shake()`：30Hz + 衰减正弦，与旧 Host `AnimationSystem` 行为对齐 |
| Blur / Dim | 后端 `blur_amount` / `dim_level`，前端 CSS filter，功能等价 |

### 2.3 ZIP 模式说明

ZIP 资源来源已完整支持。后端注册 `ring-asset` 自定义协议 handler（`lib.rs`），前端通过 `convertFileSrc(logicalPath, "ring-asset")` 生成协议 URL，WebView 的 `<img>` / `<audio>` / `<video>` 等元素直接通过该协议加载资源，协议 handler 内部委托 `ResourceManager` 读取——FS 与 ZIP 来源对前端完全透明，不再需要文件系统路径。

---

## 三、未完成与差距

### 3.1 表现与 UI

| 项 | 旧 Host | host-tauri 现状 | 备注 |
|----|---------|-----------------|------|
| **Rule 过渡 `reversed`** | DissolveRenderer shader 支持 reversed | 类型有 `reversed` 字段；`TransitionOverlay.vue` TODO | CSS mask + 整层不透明度难以表达反转遮罩语义；**延后**至 Canvas/WebGL 升级时一并做 |
| **存档缩略图** | GPU readback → PNG，`save_thumbnail` | 无截图管线；`save_thumbnail` 未调用；`SaveLoadScreen.vue` 仅槽位号 | 可用 Tauri 窗口截图或前端采集后写入并在列表展示 |
| **缓动体系统一** | `AnimationSystem` + `EasingFunction` 枚举 | 多为 CSS `ease` / `ease-in-out` 与局部线性插值 | Shake/Blur/Dim 已覆盖；若要对齐「全场景同一套 easing」，需前端库或后端统一曲线 |

### 3.2 测试、调试与自动化

| 项 | 旧 Host 要点 | host-tauri 现状 | 迁移方向（概要） |
|----|--------------|-----------------|------------------|
| **输入录制与回放** | `RecordingBuffer`、`RecordingExporter`（JSONL）、`InputReplayer`、F8 / Panic 导出；`host/src/input/recording.rs` | 未实现 | 前端录浏览器事件 → JSONL，或 Playwright 等替代 |
| **Headless** | CPU-only egui、固定帧循环、与回放联动；`--headless`、`--replay-input`、`--exit-on`、`--max-frames`、`--timeout-sec` 等 | 仅 `RING_HEADLESS` 藏窗，仍依赖 WebView | 独立 CLI 复用 vn-runtime + CommandExecutor，可无 Tauri |
| **事件流（EventStream）** | `EngineEvent` JSONL；`host/src/event_stream/mod.rs` | 无对等；Debug HTTP 的 `debug_snapshot` 不等价 | tracing 关键路径，或 Tauri 事件通道 |
| **CLI 参数** | 与 headless/回放/事件流联动 | 无 CLI，仅环境变量 | 与 headless/工具链一并设计 |

### 3.3 扩展与嵌入

| 项 | 旧 Host 要点 | host-tauri 现状 | 备注 |
|----|--------------|-----------------|------|
| **扩展/能力系统** | `EffectExtension`、`ExtensionRegistry`、`CapabilityId`；`host/src/extensions/` | 效果在 `command_executor.rs` / `state.rs` 硬编码 | **低优先级**；内置效果已够用，仅在有第三方效果插件需求时再抽象 |
| **UI 模式插件** | `UiModeHandler`、`UiModeRegistry`；`host/src/ui_modes/` | `RequestUI` 为降级处理 | 前端动态组件 + IPC 传参 |
| **小游戏 / WebView 模式** | `MiniGameRuntime`、`BridgeServer`（tiny_http）；`host/src/game_mode/` | 未实现 | iframe 或动态组件 + postMessage / IPC |

### 3.4 明确不迁移（由平台承担）

| 项 | 说明 |
|----|------|
| **纹理缓存与显存预算** | 旧 Host `TextureCache`（FIFO、256MB 等）；Web 端由浏览器管理图片内存，**无需**对等实现 |

---

## 四、技术栈替代（无需显式迁移）

| 旧 Host 特性 | Web 平台替代 |
|---|---|
| wgpu GPU 渲染（SpriteRenderer/DissolveRenderer） | CSS transitions/animations + DOM |
| egui 即时模式 UI（12+ 界面） | Vue 组件 |
| winit 窗口管理 | Tauri 窗口 |
| InputManager | 浏览器事件 |
| ChoiceNavigator（键盘选项导航） | `ChoiceOverlay.vue`（可补键盘导航） |
| nine-patch / image slider | CSS `border-image`、`<input type="range">` 等 |
| GpuTexture / TextureFactory | `<img>` / CSS `background-image` |
| TextureCache（256MB 预算） | 浏览器内存管理 |
| egui CJK 字体 | CSS `font-family` + `@font-face` |
| 正交投影 / quad | CSS `transform` / `position` |

---

## 五、优先级建议（仅待办）

### P0（用户体验）

1. **Rule 过渡 `reversed`** — 与 Canvas/WebGL 升级绑定，当前方案刻意延后。

### P1（功能完整性）

2. **存档缩略图** — 采集、写入、存读档 UI 展示。

### P2（开发体验 / 测试）

3. **Headless 与 CLI** — CI、脚本验证、基准（可与独立 CLI 同规划）。
4. **输入录制与回放** — 自动化测试、Bug 复现。
5. **事件流** — 调试与性能分析。

### P3（可扩展性，按需）

6. **UI 模式插件** — `RequestUI` 前端完整实现。
7. **小游戏模式** — iframe / 组件嵌入与通信。
8. **扩展/能力系统** — 仅在有第三方效果插件需求时。
