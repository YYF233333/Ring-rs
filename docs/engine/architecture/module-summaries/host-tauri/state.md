# host-tauri/state

> LastVerified: 2026-03-25
> Owner: Claude

## 职责

核心应用状态管理——持有 VNRuntime、RenderState、`Services` 聚合的子系统以及游戏循环（tick/click/choose）逻辑。

## 关键类型/结构

| 类型 | 说明 |
|------|------|
| `AppState` | Tauri managed state，内含 `Arc<Mutex<AppStateInner>>` |
| `AppStateInner` | 所有游戏状态的聚合体（核心逻辑） |
| `Services` | setup 一次性注入：`AudioManager`、`ResourceManager`、`SaveManager`、`AppConfig`；经 `services()` / `services_mut()` 访问 |
| `WaitingFor` | Host 侧等待状态枚举：Nothing / Click / Choice / Time / Cutscene / Signal |
| `UserSettings` | 用户可调设置（音量、文字速度、Auto 延迟、全屏） |
| `HistoryEntry` | 对话历史条目（speaker + text） |
| `PersistentStore` | 跨会话持久化变量存储（`$persistent.key`） |
| `Snapshot` | 状态快照（render_state + runtime_state + history_len） |
| `SnapshotStack` | 快照栈，容量上限 50，用于 Backspace 回退 |

## AppStateInner 字段

```
AppStateInner {
    runtime:                   Option<VNRuntime>,       // vn-runtime 实例
    command_executor:          CommandExecutor,         // Command → RenderState 翻译器
    render_state:              RenderState,             // 当前渲染快照
    waiting:                   WaitingFor,              // 当前等待状态
    typewriter_timer:          f32,                     // 打字机累积计时
    text_speed:                f32,                     // 基础文字速度 (CPS)
    script_finished:           bool,                    // 脚本是否执行完毕
    services:                  Option<Services>,        // 子系统集合（setup 注入；未就绪时 None，业务路径经访问器断言）
    history:                   Vec<HistoryEntry>,       // 对话历史（最新在前）
    user_settings:             UserSettings,            // 用户设置
    persistent_store:          PersistentStore,         // 持久化变量
    snapshot_stack:            SnapshotStack,           // 回退快照栈
    playback_mode:             PlaybackMode,            // Normal / Auto / Skip
    auto_timer:                f32,                     // Auto 模式计时器
    bg_transition_elapsed:     f32,                     // 背景 dissolve 内部计时（不写入 RenderState.progress）
    scene_transition_elapsed:  f32,                     // 场景过渡各阶段内部计时
}
```

## 数据流

### 游戏初始化 (`init_game` / `init_game_from_resource`)

1. `init_game` 与 `init_game_from_resource` 均先调用 `reset_session()`，清理上一会话遗留状态（runtime、render_state、history、快照栈、playback_mode、音频等），再进入后续步骤
2. ResourceManager 读取入口脚本文本
3. Parser 解析为 Script AST
4. 创建 VNRuntime，注册脚本到 registry
5. 递归预加载所有 `callScript` 引用的子脚本（`preload_called_scripts`）
6. 通过 `inject_persistent_vars()` 将 PersistentStore 中的持久化变量注入 runtime
7. 调用 `run_script_tick()` 执行首帧

### 每帧 tick (`process_tick`)

`process_tick` 按顺序委托子方法（原单体逻辑已拆分）：

```
process_tick(dt)
  ├─ advance_playback_mode(dt)
  │   ├─ Skip + 等待点击 → 完成打字机 → clear_click_wait
  │   └─ Auto + 等待点击 + 对话完成 → 累积 auto_timer → 超时 clear_click_wait
  ├─ update_animations(dt)
  │   ├─ update_chapter_mark
  │   ├─ title_card 计时到期清除
  │   ├─ update_background_transition：bg_transition_elapsed 累计，到期清除 background_transition
  │   └─ update_scene_transition：scene_transition_elapsed 推进 FadeIn → Hold(0.2s) → FadeOut → Completed
  ├─ resolve_waits(dt)
  │   ├─ Signal 等待：scene_transition / title_card / scene_effect / cutscene 等条件满足 → clear_wait
  │   └─ Time 等待：递减 remaining_ms → 归零 clear_wait
  ├─ 若 waiting == Nothing 且脚本未结束 → run_script_tick()
  ├─ advance_typewriter(dt)：effective_cps、inline_wait / inline 定时等待
  └─ sync_audio(dt)：services.audio.update(dt) → render_state.audio = to_audio_state()（headless → IPC → 前端 Web Audio）
  （末尾将 playback_mode 写回 render_state）
```

### 脚本推进 (`run_script_tick`)

1. `VNRuntime::tick(None)` → 产出 `(Vec<Command>, WaitingReason)`
2. `CommandExecutor::execute_batch()` → 翻译为 RenderState 变更，返回 `(ExecuteResult, Vec<AudioCommand>)`
3. 记录对话到 history（去重）
4. 分派所有 AudioCommand 到 `dispatch_audio_command`（仅更新 `services().audio` 内存状态，**不** `services().resources.read_bytes`、不 `cache_audio_bytes`）
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
- 会话级清理（停止音频、清空 runtime / render_state / history / snapshot_stack / playback_mode 等）统一由私有方法 `reset_session()` 承担；`init_game`、`init_game_from_resource` 开局前调用，`return_to_title` 内部亦调用（再额外做持久化变量保存与写盘），避免散落重复重置
- `init_game_from_resource` 会递归预加载所有子脚本，包括条件分支内的 `CallScript`
- `restore_from_save` 会重新加载 call_stack 中引用的所有脚本到 registry
- `services` 在 Tauri `setup()` 中整包注入；凡需访问 audio/resources/saves/config 的业务路径使用 `services()` / `services_mut()`，以 `expect("invariant: services initialized in setup()")` 断言已初始化（与原先四处 `Option` 解包语义一致，但聚合为单点不变量）

## 与其他模块的关系

| 模块 | 关系 |
|------|------|
| `commands.rs` | 被依赖：所有 IPC command 调用 AppStateInner 方法 |
| `command_executor.rs` | 持有实例：`self.command_executor` |
| `render_state.rs` | 持有实例：`self.render_state` |
| `audio.rs` | 经 `Services`：`self.services().audio` |
| `resources.rs` | 经 `Services`：`self.services().resources` |
| `save_manager.rs` | 经 `Services`：`self.services().saves` |
| `config.rs` | 经 `Services`：`self.services().config` |
| `vn-runtime` | 依赖：VNRuntime, Parser, Command, RuntimeState, SaveData |

## 附录：PersistentStore

存储路径 `saves/persistent.json`，JSON 格式的 `HashMap<String, VarValue>`。加载时缺失或解析失败静默回退为空 store。每次 `return_to_title` 时写盘。

## 附录：SaveManager / AppConfig

- `SaveManager`：基于 JSON 文件的存档系统，slot 命名 `slot_NNN.json`，支持 continue 存档、缩略图、最多 99 槽位
- `AppConfig`：从 `config.json` 加载，缺失时 Default 回退。包含 assets_root、saves_dir、窗口配置、音频默认值等
