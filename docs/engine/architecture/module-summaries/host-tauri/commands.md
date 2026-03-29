# host-tauri/commands

> LastVerified: 2026-03-30
> Owner: GPT-5.4

## 职责

IPC 命令层——Tauri IPC / Debug HTTP 的薄桥接层。所有会推进或修改会话状态的命令都要求前端先通过 `frontend_connected` 领取 `client_token`，再携带 token 访问后端 authoritative 的 `AppStateInner`。

## 关键类型/结构

| 类型 | 说明 |
|------|------|
| `FrontendSession` | `frontend_connected` 返回的连接令牌与当前 `RenderState` |
| `HarnessTraceBundle` | `debug_run_until` 返回的 deterministic trace bundle |

所有函数接收 `State<AppState>`，lock Mutex 后委托给 `AppStateInner` 方法。需读写 `AudioManager` / `ResourceManager` / `SaveManager` / `AppConfig` 时统一走 `inner.services()` 或 `inner.services_mut()`。

## 命令清单

### 游戏循环

| 命令 | 参数 | 返回 | 说明 |
|------|------|------|------|
| `init_game` | `client_token, script_path` | `RenderState` | 新开游戏并清理旧 continue |
| `init_game_at_label` | `client_token, script_path, label` | `RenderState` | 从指定 label 启动 |
| `tick` | `client_token, dt` | `RenderState` | 每帧推进（受 `host_screen` authority 控制） |
| `click` | `client_token` | `RenderState` | 处理用户点击 |
| `choose` | `client_token, index` | `RenderState` | 处理分支选择 |
| `get_render_state` | 无 | `RenderState` | 获取当前渲染快照 |

### 存档

| 命令 | 参数 | 返回 | 说明 |
|------|------|------|------|
| `save_game` | `client_token, slot` | `()` | 保存到指定槽位 |
| `save_game_with_thumbnail` | `client_token, slot, thumbnail_base64` | `()` | 保存并附带缩略图 |
| `load_game` | `client_token, slot` | `RenderState` | 加载存档并恢复 |
| `list_saves` | 无 | `Vec<SaveInfo>` | 列出所有存档信息 |
| `delete_save` | `slot: u32` | `()` | 删除存档 |
| `continue_game` | `client_token` | `RenderState` | 加载 continue 存档 |

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
| `frontend_connected` | `client_label?` | `FrontendSession` | 抢占 owner 并返回当前 `RenderState` |
| `set_host_screen` | `client_token, screen` | `RenderState` | 同步前端 UI 投影到后端宿主模式 |
| `return_to_title` | `client_token, save_continue?` | `RenderState` | 返回标题，可选写入 continue |
| `quit_game` | 无 | `()` | 退出应用 |
| `finish_cutscene` | `client_token` | `RenderState` | 视频播放完毕通知 |
| `backspace` | `client_token` | `RenderState` | 回退到上一快照 |

### 播放模式

| 命令 | 参数 | 返回 | 说明 |
|------|------|------|------|
| `set_playback_mode` | `client_token, mode` | `RenderState` | 设置 Normal/Auto/Skip |
| `get_playback_mode` | 无 | `String` | 获取当前播放模式 |

### 辅助

| 命令 | 参数 | 返回 | 说明 |
|------|------|------|------|
| `get_history` | 无 | `Vec<HistoryEntry>` | 获取对话历史 |
| `log_frontend` | `level, module, message, data` | 无 | 前端日志转发到 Rust tracing |
| `debug_snapshot` | 无 | `serde_json::Value` | 完整内部状态快照（调试用） |
| `debug_run_until` | `client_token, dt, max_steps, stop_on_wait?, stop_on_script_finished?` | `HarnessTraceBundle` | fixed-step 跑到稳定点并返回机读 trace bundle |

## 数据流

```
前端（典型经 useEngine）frontend_connected() → 领取 `client_token`
  │
  ▼
前端（典型经 useEngine）callBackend("command", { clientToken, ...args })
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
- 会推进或改变会话状态的命令必须先经过 `inner.assert_owner(client_token)`，防止多客户端同时驱动同一会话
- 返回类型统一为 `Result<T, String>`（Tauri IPC 要求错误类型可序列化）
- `save_game` / `save_game_with_thumbnail` 统一经 `AppStateInner::build_save_data()` 收集 audio / render / history / chapter 信息
- `update_settings` 会立即同步音量到 AudioManager（bgm_volume / sfx_volume 除以 100 转为 0.0–1.0）
- `debug_server.rs` 完整镜像所有命令，用于浏览器调试模式
- `return_to_title` 命令层不再单独调用 `stop_bgm`；BGM/会话音频清理已内聚到 `AppStateInner::reset_session()`（由 `return_to_title` 与初始化路径共用）
- `frontend_connected` 不再强制重置会话，而是改为“领取 owner + 返回当前投影状态”

## 与其他模块的关系

| 模块 | 关系 |
|------|------|
| `state.rs` | 依赖：所有命令委托给 AppStateInner |
| `render_state.rs` | 返回类型：大多数命令返回 RenderState |
| `save_manager.rs` | 使用：存档相关命令 |
| `config.rs` | 使用：get_config |
| `debug_server.rs` | 镜像：同一组命令的 HTTP 版本 |
| 前端 `useBackend.ts` | 被调用：通过 Tauri invoke 或 HTTP fetch |
