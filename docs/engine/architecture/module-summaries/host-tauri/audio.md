# host-tauri/audio

> LastVerified: 2026-03-25
> Owner: Claude

## 职责

基于 rodio 的音频管理系统——BGM/SFX 播放、淡入淡出、duck。不直接访问文件系统，音频字节通过 `cache_audio_bytes` 注入。

## 关键类型/结构

| 类型 | 说明 |
|------|------|
| `AudioManager` | 音频管理器主体 |
| `FadeState` | 淡入淡出状态机：None / FadeIn / FadeOut |

### AudioManager 字段

```
AudioManager {
    device_sink:     Option<MixerDeviceSink>,  // rodio 输出设备
    bgm_sink:        Option<Player>,           // 当前 BGM 播放器
    current_bgm_path: Option<String>,          // 当前 BGM 逻辑路径
    bgm_volume:      f32,                      // BGM 音量 (0.0–1.0)
    sfx_volume:      f32,                      // SFX 音量 (0.0–1.0)
    muted:           bool,                     // 全局静音
    fade_state:      FadeState,                // 淡入淡出进行中状态
    audio_cache:     HashMap<String, Vec<u8>>, // 已缓存的音频字节
    duck_multiplier: f32,                      // duck 当前乘数
    duck_target:     f32,                      // duck 目标值
}
```

## 数据流

### 音频播放流程

```
CommandExecutor::execute()
  └─ 返回 AudioCommand (PlayBgm/StopBgm/PlaySfx/BgmDuck/BgmUnduck)

state.rs::dispatch_audio_command()
  ├─ PlayBgm/PlaySfx → ResourceManager.read_bytes() → cache_audio_bytes() → play
  ├─ StopBgm → stop_bgm(fade_out)
  └─ BgmDuck/Unduck → duck()/unduck()

AudioManager::update(dt) (每帧)
  ├─ FadeIn: current_volume += rate × dt → 到达 target → None
  ├─ FadeOut: current_volume -= rate × dt → 到达 0 → stop/切换下一首
  └─ duck: duck_multiplier 平滑趋近 duck_target (速度 3.0/s)
```

### BGM 生命周期

1. `play_bgm(path, looping, fade_in)` → 停止旧 BGM → 更新 current_bgm_path → 设置 FadeState → 从 cache 解码 → 创建 Player
2. `stop_bgm(fade_out)` → 有 fade_out 则设置 FadeOut 状态 → 无则立即 stop + 清除 path
3. `crossfade_bgm(path, looping, duration)` → 设置 FadeOut(next_bgm) → update 中淡出完成后自动 play_bgm

### Duck 机制

- `duck()` → target = 0.3（DUCK_VOLUME_RATIO）
- `unduck()` → target = 1.0
- `update()` 中平滑过渡，速度 DUCK_FADE_SPEED = 3.0/s
- 实际音量 = base_volume × duck_multiplier

## 关键不变量

- `AudioManager` 不直接读文件——音频字节必须先通过 `cache_audio_bytes` 注入
- `unsafe impl Send`：rodio 的 MixerDeviceSink 内部线程安全但 cpal::Stream 保守标记 `!Send`，Tauri 需要跨线程共享
- `new_headless()` 创建无真实设备的实例，仅追踪状态（测试用）
- Headless 模式下 `play_bgm` 等状态更新在前、I/O 在后，确保 current_bgm_path 和 fade_state 正确
- SFX 使用 `Player::connect_new` + `detach()`，播放完自动释放
- fade_state 的生命周期以单个 BGM 操作为单位

## 与其他模块的关系

| 模块 | 关系 |
|------|------|
| `state.rs` | 被持有：`AppStateInner.audio_manager` |
| `command_executor.rs` | 间接输入：通过 AudioCommand 中转 |
| `resources.rs` | 使用：`normalize_logical_path` 规范化路径 |
| `commands.rs` | 使用：save_game 获取 current_bgm_path，update_settings 同步音量 |
