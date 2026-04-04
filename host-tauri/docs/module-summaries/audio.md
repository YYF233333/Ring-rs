# host-tauri/audio

> LastVerified: 2026-03-30
> Owner: GPT-5.4

## 职责

**Headless 音频状态追踪器**：维护当前 BGM 路径、循环、音量、duck、静音、本帧 SFX 队列与待下发的 BGM 过渡信号；**不做解码、不缓存字节、不连接输出设备**。实际播放入口为前端 **Web Audio API**，由每帧序列化后的 `RenderState.audio` 驱动。

## 关键类型/结构

| 类型 | 说明 |
|------|------|
| `AudioManager` | 音频逻辑状态主体（仅内存状态） |

### 关联常量

| 常量 | 值 | 说明 |
|------|-----|------|
| `CROSSFADE_DURATION` | `1.0` | 已有 BGM 时切换曲目使用的交叉淡入淡出时长（秒） |
| `FADE_IN_DURATION` | `0.5` | 首条 BGM 或从无 BGM 切入时的淡入时长（秒） |

### AudioManager 字段

```
AudioManager {
    current_bgm_path:   Option<String>,   // 当前 BGM 逻辑路径（规范化后）
    bgm_looping:        bool,             // 当前 BGM 是否循环
    bgm_volume:         f32,              // BGM 音量 (0.0–1.0)
    sfx_volume:         f32,              // SFX 音量 (0.0–1.0)
    muted:              bool,             // 全局静音
    duck_multiplier:    f32,              // duck 当前乘数（平滑趋近 duck_target）
    duck_target:        f32,              // duck 目标值
    sfx_queue:          Vec<SfxRequest>,  // 本帧待输出的音效请求
    pending_transition: Option<f32>,      // 待下发的 BGM 过渡时长（秒），在 drain 时 take 消费
}
```

## 数据流

```
CommandExecutor::execute_batch()
  └─ 返回 Vec<AudioCommand> (PlayBgm / StopBgm / PlaySfx / BgmDuck / BgmUnduck)

state.rs::dispatch_audio_command()
  └─ 仅调用 AudioManager 方法更新内部状态（不读文件、不读 ResourceManager）

process_tick(dt) 每帧末尾
  ├─ audio_manager.update(dt)     // 平滑 duck_multiplier
  └─ render_state.audio = audio.drain_audio_state()
        ├─ 根据 current_bgm_path / muted / duck 生成 Option<BgmState>
        ├─ 取出 sfx_queue 写入 AudioRenderState.sfx_queue
        ├─ pending_transition.take() → Option<BgmTransition>（含 duration）写入 bgm_transition
        └─ 清空内部 SFX 队列（下帧重新累积）

JSON RenderState ──→ 前端 ──→ Web Audio API 按 bgm、sfx_queue、bgm_transition 播放
```

### BGM / SFX 语义

- `play_bgm(path, looping, fade_in)`：`_fade_in` 保留为 API 兼容（脚本侧仍可传）；**当路径相对上一帧实际发生变化时**，若已有 BGM 则设置 `pending_transition = Some(CROSSFADE_DURATION)`（1.0s），否则为 `Some(FADE_IN_DURATION)`（0.5s）。路径与当前相同则不产生过渡信号。
- `stop_bgm(fade_out)`：**不再忽略** `fade_out`——若当前有 BGM 且 `fade_out` 为 `Some(duration)`，则在清除路径前设置 `pending_transition = Some(duration)`；随后清除 `current_bgm_path`。
- `play_sfx(path)`：将 `SfxRequest { path, volume }` 入队；**下一帧** `drain_audio_state()` 输出后队列被 `take` 清空。
- `duck()` / `unduck()`：`duck_target` 在 `update(dt)` 中以 `DUCK_FADE_SPEED` 向目标平滑过渡；`BgmState.volume` 已乘 `duck_multiplier` 并尊重 `muted`。

## 关键不变量

- **无 rodio / 无设备 sink**：不持有 `device_sink`、`bgm_sink`，无 `FadeState`、无 `audio_cache`。
- **无 `unsafe impl Send`**：结构体为纯数据 + 浮点状态，可按普通 Rust 规则跨线程使用（由 `AppStateInner` 的 `Mutex` 保护）。
- **无 I/O**：不调用 `ResourceManager::read_bytes`；路径经 `normalize_logical_path` 规范化后存入状态。
- `drain_audio_state()` **消费** SFX 队列（`std::mem::take`）与 **消费** `pending_transition`（`Option::take`），保证每帧音效与 BGM 过渡信号只下发一次。
- 会话级「立刻停 BGM」仍由 `state.rs::reset_session()` 调用 `stop_bgm`；脚本 `StopBgm` 仍经 `dispatch_audio_command`。

## 与其他模块的关系

| 模块 | 关系 |
|------|------|
| `state.rs` | 经 `Services.audio` 持有；`process_tick` 末尾 `drain_audio_state()` 写入 `render_state.audio` |
| `command_executor.rs` | 间接输入：通过 `AudioCommand` 由 `dispatch_audio_command` 应用 |
| `render_state.rs` | 输出目标：`AudioRenderState` / `BgmState` / `BgmTransition` / `SfxRequest` 定义 |
| `commands.rs` | 使用：save_game 读取 `current_bgm_path`，update_settings 同步音量 |
| `resources` | **不读取音频字节**；仅使用 `normalize_logical_path` 规范化脚本/资源逻辑路径字符串 |
