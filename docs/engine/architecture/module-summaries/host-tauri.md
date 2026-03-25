# host-tauri 模块总览

> LastVerified: 2026-03-25
> Owner: Claude

## 职责

Tauri 2 宿主应用——将 vn-runtime 的 Command 通过 IPC 序列化为 JSON RenderState 发送到 Vue 3 前端渲染。

## 架构概览

```
┌──────────────────────────────────────────────────────────────────┐
│  Vue 3 前端 (WebView)                                            │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────────────┐ │
│  │ App.vue  │  │ VNScene  │  │ Screens  │  │ composables/     │ │
│  │ (路由)   │→ │ (渲染)   │  │ (系统UI) │  │ useEngine 等     │ │
│  └────┬─────┘  └──────────┘  └──────────┘  └──────┬───────────┘ │
│       │                                           │              │
│       └─── callBackend() ─────────────────────────┘              │
│                    │ (Tauri IPC / Debug HTTP)                     │
├────────────────────┼─────────────────────────────────────────────┤
│  Rust 后端         │                                              │
│  ┌─────────────────▼──────────────────────┐                      │
│  │ commands.rs (#[command] × 22)          │                      │
│  └─────────────────┬──────────────────────┘                      │
│                    │ lock AppState                                │
│  ┌─────────────────▼──────────────────────┐                      │
│  │ state.rs (AppStateInner)               │                      │
│  │  ├─ VNRuntime (vn-runtime)             │                      │
│  │  ├─ CommandExecutor                    │                      │
│  │  ├─ RenderState                        │                      │
│  │  ├─ AudioManager                       │                      │
│  │  ├─ ResourceManager                    │                      │
│  │  ├─ SaveManager                        │                      │
│  │  └─ SnapshotStack                      │                      │
│  └────────────────────────────────────────┘                      │
└──────────────────────────────────────────────────────────────────┘
```

## 数据流

1. **初始化**：`lib.rs` setup → 查找项目根 → 加载配置 → 创建 ResourceManager / SaveManager / AudioManager → 注入 `AppStateInner`
2. **游戏循环**：前端 `requestAnimationFrame` → `callBackend("tick", {dt})` → `AppStateInner::process_tick()` → 推进打字机、过渡动画、音频 → 返回 `RenderState` JSON
3. **用户交互**：click/choose/backspace → 对应 IPC command → 修改 `AppStateInner` 状态 → 返回新 `RenderState`
4. **脚本执行**：`run_script_tick()` → `VNRuntime::tick()` → 产出 `Vec<Command>` → `CommandExecutor::execute_batch()` 翻译为 RenderState 变更 + AudioCommand 副作用

## 文件结构

| 文件 | 职责 | 摘要 |
|------|------|------|
| `lib.rs` | Tauri Builder 入口 | [无独立摘要] |
| `state.rs` | 核心状态管理 | [state.md](host-tauri/state.md) |
| `commands.rs` | IPC 命令层 | [commands.md](host-tauri/commands.md) |
| `command_executor.rs` | Command → RenderState 翻译 | [command-executor.md](host-tauri/command-executor.md) |
| `render_state.rs` | 可序列化渲染状态 | [render-state.md](host-tauri/render-state.md) |
| `audio.rs` | rodio 音频管理 | [audio.md](host-tauri/audio.md) |
| `resources.rs` | 资源路径管理 | [resources.md](host-tauri/resources.md) |
| `save_manager.rs` | 存档读写 | [state.md](host-tauri/state.md) 附录 |
| `config.rs` | 应用配置 | [state.md](host-tauri/state.md) 附录 |
| `manifest.rs` | 立绘元数据 | [resources.md](host-tauri/resources.md) 附录 |
| `debug_server.rs` | Debug HTTP 镜像 | 仅 debug build |
| `src/` (前端) | Vue 3 渲染层 | [frontend.md](host-tauri/frontend.md) |

## 与旧 host 的主要区别

| 方面 | 旧 host (macroquad) | host-tauri |
|------|---------------------|------------|
| 渲染 | macroquad GPU 直接绘制 | Vue 3 WebView DOM/CSS |
| 通信 | 同进程函数调用 | Tauri IPC (JSON 序列化) |
| 游戏循环 | `App::update()` 同步帧循环 | 前端 rAF + IPC `tick` |
| 音频 | kira | rodio |
| 状态管理 | `App` struct 直接持有 | `Arc<Mutex<AppStateInner>>` |
| UI 系统 | egui immediate-mode | Vue 3 组件 |
| 文件数 | ~60 .rs + 资源 | 11 .rs + ~25 .vue/.ts |
| 过渡动画 | Rust 侧 GPU shader | Rust 侧状态机 + CSS transition |

## 关键不变量

- `AppStateInner` 通过 `Arc<Mutex<>>` 共享，所有 IPC command 必须 lock 后操作
- `RenderState` 是前端唯一的数据源：前端不持有独立游戏状态，只读 + 渲染
- 音频字节通过 `cache_audio_bytes` 预加载到内存，不在播放时直接读文件
- Debug HTTP server 仅 `#[cfg(debug_assertions)]` 编译，镜像所有 IPC command

## 与其他模块的关系

- **依赖** `vn-runtime`：Parser、VNRuntime、Command、ScriptNode、SaveData、RuntimeState
- **被依赖**：无（顶层应用）
