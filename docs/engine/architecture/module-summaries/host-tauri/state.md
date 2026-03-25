# host-tauri/state

> LastVerified: 2026-03-25
> Owner: Claude

## 职责

核心应用状态管理——持有 VNRuntime、RenderState、各子系统 manager 以及游戏循环（tick/click/choose）逻辑。

## 关键类型/结构

| 类型 | 说明 |
|------|------|
| `AppState` | Tauri managed state，内含 `Arc<Mutex<AppStateInner>>` |
| `AppStateInner` | 所有游戏状态的聚合体（750 行核心逻辑） |
| `WaitingFor` | Host 侧等待状态枚举：Nothing / Click / Choice / Time / Cutscene / Signal |
| `UserSettings` | 用户可调设置（音量、文字速度、Auto 延迟、全屏） |
| `HistoryEntry` | 对话历史条目（speaker + text） |
| `PersistentStore` | 跨会话持久化变量存储（`$persistent.key`） |
| `Snapshot` | 状态快照（render_state + runtime_state + history_len） |
| `SnapshotStack` | 快照栈，容量上限 50，用于 Backspace 回退 |

## AppStateInner 字段

```
AppStateInner {
    runtime:           Option<VNRuntime>,       // vn-runtime 实例
    command_executor:  CommandExecutor,          // Command → RenderState 翻译器
    render_state:      RenderState,             // 当前渲染快照
    waiting:           WaitingFor,              // 当前等待状态
    typewriter_timer:  f32,                     // 打字机累积计时
    text_speed:        f32,                     // 基础文字速度 (CPS)
    script_finished:   bool,                    // 脚本是否执行完毕
    audio_manager:     Option<AudioManager>,    // 音频（初始化失败时 None）
    resource_manager:  Option<ResourceManager>, // 资源读取
    save_manager:      Option<SaveManager>,     // 存档
    config:            Option<AppConfig>,        // 运行时配置
    history:           Vec<HistoryEntry>,        // 对话历史（最新在前）
    user_settings:     UserSettings,            // 用户设置
    persistent_store:  PersistentStore,         // 持久化变量
    snapshot_stack:    SnapshotStack,           // 回退快照栈
    playback_mode:     PlaybackMode,            // Normal / Auto / Skip
    auto_timer:        f32,                     // Auto 模式计时器
}
```

## 数据流

### 游戏初始化 (`init_game_from_resource`)

1. ResourceManager 读取入口脚本文本
2. Parser 解析为 Script AST
3. 创建 VNRuntime，注册脚本到 registry
4. 递归预加载所有 `callScript` 引用的子脚本（`preload_called_scripts`）
5. 注入 PersistentStore 中的持久化变量到 runtime
6. 调用 `run_script_tick()` 执行首帧

### 每帧 tick (`process_tick`)

```
process_tick(dt)
  ├─ Skip 模式：若等待点击 → 立即完成打字机 → 清除等待
  ├─ Auto 模式：对话完成后累积计时 → 超过 auto_delay → 清除等待
  ├─ 更新 chapter_mark / title_card / background_transition / scene_transition 动画
  ├─ 解析信号等待：检查 Host 侧事件是否完成 → 发送 Signal 解除 Runtime 等待
  │   ├─ scene_transition 完成 → Signal("scene_transition")
  │   ├─ title_card 消失 → Signal("title_card")
  │   └─ scene_effect / cutscene 完成 → 对应信号
  ├─ 解析时间等待：递减 remaining_ms → 归零时清除 Runtime 等待
  ├─ 若 waiting == Nothing 且脚本未结束 → run_script_tick()
  ├─ 推进打字机（基于 text_speed × dt，处理 inline_wait / effective_cps）
  └─ audio_manager.update(dt) 推进淡入淡出和 duck
```

### 脚本推进 (`run_script_tick`)

1. `VNRuntime::tick(None)` → 产出 `(Vec<Command>, WaitingReason)`
2. `CommandExecutor::execute_batch()` → 翻译为 RenderState 变更，返回 `(ExecuteResult, Vec<AudioCommand>)`
3. 记录对话到 history（去重）
4. 分派所有 AudioCommand 到 AudioManager
5. 根据 Runtime 的 `WaitingReason`（权威来源）设置 `waiting` 状态
6. 同步 runtime persistent 变量到 PersistentStore
7. 若无命令且无等待 → 标记 `script_finished`

### 用户交互

| 操作 | 方法 | 行为 |
|------|------|------|
| 点击 | `process_click()` | 打字中→完成打字；inline_wait→清除；等待点击→捕获快照→清除等待 |
| 选择 | `process_choose(index)` | 注入 `RuntimeInput::choice` → runtime.tick → 清除选项 → 清除等待 |
| 回退 | `restore_snapshot()` | 弹出快照栈 → 恢复 render_state + runtime_state + history 截断 |
| 结束过场 | `finish_cutscene()` | 清除 cutscene 状态 + 清除 Cutscene 等待 |

## 关键不变量

- `AppStateInner` 始终通过 `Mutex` 保护，单线程串行访问
- 快照栈最大 50 条，超出时从底部淘汰
- `script_finished` 仅在 runtime.tick 返回空命令且无等待时设置
- 持久化变量在每次 `run_script_tick` 后同步，`return_to_title` 时写盘
- `init_game_from_resource` 会递归预加载所有子脚本，包括条件分支内的 `CallScript`
- `restore_from_save` 会重新加载 call_stack 中引用的所有脚本到 registry

## 与其他模块的关系

| 模块 | 关系 |
|------|------|
| `commands.rs` | 被依赖：所有 IPC command 调用 AppStateInner 方法 |
| `command_executor.rs` | 持有实例：`self.command_executor` |
| `render_state.rs` | 持有实例：`self.render_state` |
| `audio.rs` | 持有实例：`self.audio_manager` |
| `resources.rs` | 持有实例：`self.resource_manager` |
| `save_manager.rs` | 持有实例：`self.save_manager` |
| `config.rs` | 持有实例：`self.config` |
| `vn-runtime` | 依赖：VNRuntime, Parser, Command, RuntimeState, SaveData |

## 附录：PersistentStore

存储路径 `saves/persistent.json`，JSON 格式的 `HashMap<String, VarValue>`。加载时缺失或解析失败静默回退为空 store。每次 `return_to_title` 时写盘。

## 附录：SaveManager / AppConfig

- `SaveManager`：基于 JSON 文件的存档系统，slot 命名 `slot_NNN.json`，支持 continue 存档、缩略图、最多 99 槽位
- `AppConfig`：从 `config.json` 加载，缺失时 Default 回退。包含 assets_root、saves_dir、窗口配置、音频默认值等
