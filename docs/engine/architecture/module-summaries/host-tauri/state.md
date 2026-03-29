# host-tauri/state

> LastVerified: 2026-03-30
> Owner: GPT-5.4

## 职责

核心应用状态管理——持有 VNRuntime、RenderState、`Services` 聚合的子系统以及游戏循环（tick/click/choose）逻辑。

## 关键类型/结构

| 类型 | 说明 |
|------|------|
| `AppState` | Tauri managed state，内含 `Arc<Mutex<AppStateInner>>` |
| `AppStateInner` | 所有游戏状态的聚合体（核心逻辑） |
| `Services` | setup 一次性注入：`AudioManager`、`ResourceManager`、`SaveManager`、`AppConfig`、`Manifest`；经 `services()` / `services_mut()` 访问 |
| `WaitingFor` | Host 侧等待状态枚举：Nothing / Click / Choice / Time / Cutscene / Signal / UIResult { key: String } |
| `FrontendSession` | `frontend_connected()` 返回的 owner token + 当前 `RenderState` |
| `HarnessTraceBundle` | `debug_run_until()` 的机读 trace bundle |
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
    host_screen:               HostScreen,              // 后端 authoritative 宿主模式
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
    client_owner:              Option<SessionOwner>,    // 当前会话 owner
    logical_time_ms:           u64,                     // deterministic harness 逻辑时间
    trace_events:              Vec<HarnessTraceEvent>,  // 机读 trace 缓冲区
}
```

## 数据流

### 游戏初始化 (`init_game` / `init_game_from_resource`)

1. `build_runtime_from_resource()` 先通过 `ResourceManager` 读取入口脚本并递归预加载所有 `callScript` 子脚本
2. `start_runtime()` 再统一执行 `reset_session()`、安装 runtime、注入 `PersistentStore`、切换 `host_screen = InGame`
3. 普通新游戏走 `init_game_from_resource(script_path)`；开发重入走 `init_game_from_resource_at_label(script_path, label)`
4. 命令层在新开游戏前会删除旧 `continue`
5. 最后调用 `run_script_tick()` 执行首帧

### 每帧 tick (`process_tick`)

`process_tick` 先检查 `host_screen`；只有 `InGame` 允许推进。随后按顺序委托子方法：

```
process_tick(dt)
  ├─ 若 host_screen != InGame → 仅回写投影字段，不推进脚本
  ├─ advance_playback_mode(dt)
  │   ├─ Skip + 等待点击/时间/Signal/Cutscene → 尽量快进 host wait
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
  └─ sync_audio(dt)：services.audio.update(dt) → render_state.audio = to_audio_state()
  （末尾统一回写 `playback_mode + host_screen` 到 `RenderState`，并记录 trace）
```

### 脚本推进 (`run_script_tick`)

1. `VNRuntime::tick(None)` → 产出 `(Vec<Command>, WaitingReason)`
2. `CommandExecutor::execute_batch()` → 翻译为 RenderState 变更，返回 `(ExecuteResult, Vec<AudioCommand>)`
3. 记录对话到 history（去重）
4. 分派所有 AudioCommand 到 `dispatch_audio_command`（仅更新 `services().audio` 内存状态，**不** `services().resources.read_bytes`、不 `cache_audio_bytes`）
5. 根据 Runtime 的 `WaitingReason`（权威来源）设置 `waiting` 状态
6. 同步 runtime persistent 变量到 PersistentStore
7. 若无命令且无等待 → 视为脚本自然结束，统一执行 `return_to_title(false)` 返回标题并清理 `continue`

当 `ExecuteResult::RequestUI` 出现时：将 `mode` / `key` / `params` 写入 `render_state.active_ui_mode`（`UiModeRequest`），并设置 `waiting = UIResult { key }`，**不再**将 UI 结果降级为空字符串；前端通过 `handle_ui_result(key, value)` 回传后，Host 会直接消费这次 `runtime.tick(UIResult)` 产出的首批 `Command`，避免首句后续对话被跳过。

### 用户交互

| 操作 | 方法 | 行为 |
|------|------|------|
| 点击 | `process_click()` | 打字中→完成打字；inline_wait→清除；等待点击→捕获快照→清除等待 |
| 选择 | `process_choose(index)` | 注入 `RuntimeInput::choice` → 清除旧选项 → 立即消费该 tick 产出的 `Command` / `WaitingReason`，避免分支后首句文本被吞 |
| UI结果 | `handle_ui_result(key, value)` | 校验 `key` 与当前 `UIResult` 等待一致 → 注入 `RuntimeInput::UIResult` → 立即消费该 tick 产出的 `Command` / `WaitingReason` → 清除旧 `active_ui_mode` 并渲染下一帧 |
| 回退 | `restore_snapshot()` | 弹出快照栈 → 恢复 render_state + runtime_state + runtime history；同步恢复 BGM，随后把 `playback_mode` 归一到 `Normal` |
| 结束过场 | `finish_cutscene()` | 清除 cutscene 状态；恢复 BGM duck；等待态在下一次 tick / signal 归一 |

### 存档恢复边界（当前实现）

- `build_save_data()` 默认保存当前 `runtime_state + history + RenderSnapshot + AudioState`
- 若当前等待态属于 `Choice / UIResult / Signal / Cutscene` 这类宿主中间态，则改为**从最近一次 snapshot 边界生成 slot/continue**，避免把当前宿主无法直接重建的中间态直接写入存档
- `restore_from_save()` 对旧存档中残留的 `Choice / UIResult / Signal / Cutscene` 等待态做保护性归一：若命中了当前宿主无法直接重建的等待态，则把 runtime/host waiting 一并收敛到 `WaitForClick`

### Owner / harness

- `frontend_connected(client_label?)`：生成新的 `client_token`，抢占当前会话 owner，并返回 `FrontendSession`
- `assert_owner(client_token)`：所有推进类命令在修改状态前先校验 owner
- `debug_run_until(dt, max_steps, ...)`：fixed-step 驱动 `process_tick()`，生成 `HarnessTraceBundle`

## 关键不变量

- `AppStateInner` 始终通过 `Mutex` 保护，单线程串行访问
- `host_screen` 是后端 authoritative 的推进边界；前端页面状态只能投影，不能绕过它推进会话
- 会推进会话的命令必须先通过 `assert_owner(client_token)`，防止多客户端同时驱动同一 `AppStateInner`
- 快照栈最大 50 条，超出时从底部淘汰
- 脚本自然结束时不会停留在 `InGame` 空转，而是统一走 `return_to_title(false)` 做会话清理与标题返回
- 持久化变量在每次 `run_script_tick` 后同步，`return_to_title(save_continue)` 时写盘
- 会话级清理（停止音频、清空 runtime / render_state / history / snapshot_stack / playback_mode 等）统一由私有方法 `reset_session()` 承担；`init_game`、`init_game_from_resource` 开局前调用，`return_to_title` 内部亦调用（再额外做持久化变量保存与写盘），避免散落重复重置
- `init_game_from_resource` 会递归预加载所有子脚本，包括条件分支内的 `CallScript`
- `restore_from_save` 会重新加载 call_stack 中引用的所有脚本到 registry，并且**不再**额外执行入口首 tick
- 当前 `restore_from_save` 的渲染恢复边界仍是“背景 + 角色 + 音频 + history + waiting”；不会重建存档瞬间的 `dialogue` / `choices` / `active_ui_mode`，但 `build_save_data` 已避免把这些宿主中间态直接落盘
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

- `SaveManager`：基于 JSON 文件的存档系统，slot 命名 `slot_NNN.json`，支持 `save_continue` / `delete_continue` / 缩略图 / 最多 99 槽位
- `AppConfig`：从 `config.json` 严格加载，并在 setup 阶段执行 `validate()`；缺失字段、未知字段、非法路径和值域错误都会 fail-fast
