# host-tauri/commands

> LastVerified: 2026-03-25
> Owner: Claude

## 职责

IPC 命令层——24 个 `#[tauri::command]` 函数（`use tauri::command`），作为前端与 Rust 后端之间的薄桥接层。锁内通过 `AppStateInner::services()` / `services_mut()` 访问子系统，不再对四个独立 `Option` 字段分别解包。

## 关键类型/结构

无独立类型。所有函数接收 `State<AppState>`，lock Mutex 后委托给 `AppStateInner` 方法，返回 `Result<T, String>`。需读写 `AudioManager` / `ResourceManager` / `SaveManager` / `AppConfig` 时统一走 `inner.services()` 或 `inner.services_mut()`。

## 命令清单

### 游戏循环

| 命令 | 参数 | 返回 | 说明 |
|------|------|------|------|
| `init_game` | `script_path: String` | `RenderState` | 解析脚本并初始化运行时 |
| `tick` | `dt: f32` | `RenderState` | 每帧推进（打字机、动画、音频） |
| `click` | 无 | `RenderState` | 处理用户点击 |
| `choose` | `index: usize` | `RenderState` | 处理分支选择 |
| `get_render_state` | 无 | `RenderState` | 获取当前渲染快照 |

### 存档

| 命令 | 参数 | 返回 | 说明 |
|------|------|------|------|
| `save_game` | `slot: u32` | `()` | 保存到指定槽位 |
| `load_game` | `slot: u32` | `RenderState` | 加载存档并恢复 |
| `list_saves` | 无 | `Vec<SaveInfo>` | 列出所有存档信息 |
| `delete_save` | `slot: u32` | `()` | 删除存档 |
| `continue_game` | 无 | `RenderState` | 加载 continue 存档 |

### 配置与设置

| 命令 | 参数 | 返回 | 说明 |
|------|------|------|------|
| `get_assets_root` | 无 | `String` | 资源根目录绝对路径 |
| `get_config` | 无 | `AppConfig` | 当前配置 |
| `get_user_settings` | 无 | `UserSettings` | 用户设置 |
| `update_settings` | `settings: UserSettings` | `()` | 更新设置（含音量同步） |

### 流程控制

| 命令 | 参数 | 返回 | 说明 |
|------|------|------|------|
| `frontend_connected` | 无 | `()` | 前端（重新）连接通知，重置后端状态 |
| `return_to_title` | 无 | `()` | 返回标题画面 |
| `quit_game` | 无 | `()` | 退出应用 |
| `finish_cutscene` | 无 | `RenderState` | 视频播放完毕通知 |
| `backspace` | 无 | `RenderState` | 回退到上一快照 |

### 播放模式

| 命令 | 参数 | 返回 | 说明 |
|------|------|------|------|
| `set_playback_mode` | `mode: String` | `()` | 设置 Normal/Auto/Skip |
| `get_playback_mode` | 无 | `String` | 获取当前播放模式 |

### 辅助

| 命令 | 参数 | 返回 | 说明 |
|------|------|------|------|
| `get_history` | 无 | `Vec<HistoryEntry>` | 获取对话历史 |
| `log_frontend` | `level, module, message, data` | 无 | 前端日志转发到 Rust tracing |
| `debug_snapshot` | 无 | `serde_json::Value` | 完整内部状态快照（调试用） |

## 数据流

```
前端（典型经 useEngine）callBackend("command", args)
  │
  ├─ Tauri 模式 → invoke() → tauri::generate_handler![] → #[command] fn
  └─ Debug 模式 → fetch POST /api/{command} → debug_server::dispatch()
       │
       ▼
  state.inner.lock() → AppStateInner 方法 → Result<T, String>
       │
       ▼
  JSON 序列化 → 返回前端
```

## 关键不变量

- 所有命令的第一步是 `state.inner.lock().map_err()`——lock 失败返回错误而非 panic
- 返回类型统一为 `Result<T, String>`（Tauri IPC 要求错误类型可序列化）
- `save_game` 构造 `SaveData` 时收集 audio state 和 chapter_mark，确保存档完整
- `update_settings` 会立即同步音量到 AudioManager（bgm_volume / sfx_volume 除以 100 转为 0.0–1.0）
- `debug_server.rs` 完整镜像所有命令，用于浏览器调试模式
- `return_to_title` 命令层不再单独调用 `stop_bgm`；BGM/会话音频清理已内聚到 `AppStateInner::reset_session()`（由 `return_to_title` 与初始化路径共用）

## 与其他模块的关系

| 模块 | 关系 |
|------|------|
| `state.rs` | 依赖：所有命令委托给 AppStateInner |
| `render_state.rs` | 返回类型：大多数命令返回 RenderState |
| `save_manager.rs` | 使用：存档相关命令 |
| `config.rs` | 使用：get_config |
| `debug_server.rs` | 镜像：同一组命令的 HTTP 版本 |
| 前端 `useBackend.ts` | 被调用：通过 Tauri invoke 或 HTTP fetch |
