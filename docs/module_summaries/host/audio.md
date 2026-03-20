# host/audio 摘要

## Purpose

`audio` 提供 Host 音频管理：BGM/SFX 播放、淡入淡出、交叉淡化、静音与音量状态管理。

## PublicSurface

- 模块入口：`host/src/audio/mod.rs`（AudioManager struct、音量/duck/静音控制）
- 播放逻辑：`host/src/audio/playback.rs`（play_bgm、stop_bgm、crossfade_bgm、play_sfx、update + 淡入淡出状态机）
- 核心类型：`AudioManager`（`device_sink` 为 `Option<MixerDeviceSink>`，headless 时为 `None`）
- 构造：`new()` 连接真实设备；`new_headless()` 无设备、仅追踪状态
- 关键接口：`play_bgm`、`stop_bgm`、`crossfade_bgm`、`play_sfx`、`play_video_audio`、`duck`、`unduck`、`update`

## KeyFlow

1. 调用方通过 `ResourceManager.read_bytes()` 读取音频字节，然后 `AudioManager.cache_audio_bytes()` 预缓存。
2. `play_bgm`：先更新状态（`current_bgm_path`、`fade_state`），再在有 `device_sink` 时执行 I/O，headless 下状态仍正确推进。
3. `play_sfx`、`play_video_audio` 在无 `device_sink` 时提前返回（headless 安全）。
4. BGM 路径进入独立播放器并维护 `FadeState`。
5. 每帧 `update(dt)` 推进淡入淡出状态机与 duck multiplier 过渡。
6. SFX 采用一次性播放器通道播放并自动释放。
7. `duck()` / `unduck()` 通过独立的 `duck_multiplier` 平滑压低/恢复 BGM 音量。

## Dependencies

- 依赖 `rodio` 进行音频解码与播放
- 依赖 `resources::normalize_logical_path` 统一逻辑路径
- **不直接访问文件系统或 ZIP**，音频字节由外部注入

## Invariants

- `AudioManager` 不持有 `base_path` 或 `use_zip_mode`，所有音频字节通过 `cache_audio_bytes` 注入。
- BGM 与 SFX 音量配置独立，静音状态统一影响有效输出。
- 淡入淡出状态在 `update` 中推进，避免多处并发改写。
- Duck multiplier 作为独立乘数叠加于所有 BGM 音量输出。

## FailureModes

- 音频设备不可用导致初始化失败。
- 音频字节未缓存导致播放失败（日志警告）。
- 字节解码失败导致播放降级。

## WhenToReadSource

- 需要调整 BGM 切换策略或淡入淡出曲线时。
- 需要排查音频缓存行为时。

## RelatedDocs

- [host 总览](../host.md)
- [resources 摘要](resources.md)
- [config 摘要](config.md)

## LastVerified

2026-03-19

## Owner

Composer