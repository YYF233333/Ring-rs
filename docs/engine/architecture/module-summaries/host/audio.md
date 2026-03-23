# host/audio 摘要

## Purpose

`audio` 管理 Host 侧音频状态与播放：BGM、SFX、duck、静音，以及淡入淡出/切歌过程。

## PublicSurface

- 入口：`host/src/audio/mod.rs`
- 播放实现：`host/src/audio/playback.rs`
- 核心类型：`AudioManager`
- 关键接口：`new`、`new_headless`、`cache_audio_bytes`、`play_bgm`、`stop_bgm`、`crossfade_bgm`、`play_sfx`、`play_video_audio`、`duck`、`unduck`、`update`

## KeyFlow

1. 上层先通过 `ResourceManager` 取字节，再调用 `cache_audio_bytes()` 注入缓存。
2. `play_bgm` / `crossfade_bgm` / `stop_bgm` 先更新状态，再在有真实设备时做 I/O，因此 headless 也能正确推进状态。
3. `update(dt)` 统一推进淡入淡出与 duck multiplier。
4. `play_video_audio()` 为 cutscene 音轨提供一次性播放器，不复用 BGM 通道。

## Invariants

- `AudioManager` 不直接访问文件系统或资源来源；所有字节由外部注入。
- BGM/SFX 音量、静音和 duck 是独立叠加关系。
- 淡入淡出状态只在 `update()` 中推进，避免多处同时改写。

## WhenToReadSource

- 需要调整切歌、淡入淡出或 duck 语义时。
- 需要排查 headless 与真实设备模式的差异时。
- 需要确认视频音轨与普通 SFX/BGM 的边界时。

## RelatedDocs

- [host 总览](../host.md)
- [app_command_handlers 摘要](app-command-handlers.md)
- [video 摘要](video.md)
- [resources 摘要](resources.md)

## LastVerified

2026-03-24

## Owner

GPT-5.4
