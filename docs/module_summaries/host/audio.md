# host/audio 摘要

## Purpose

`audio` 提供 Host 音频管理：BGM/SFX 播放、淡入淡出、交叉淡化、静音与音量状态管理。

## PublicSurface

- 模块入口：`host/src/audio/mod.rs`（AudioManager struct、音量/duck/静音控制）
- 播放逻辑：`host/src/audio/playback.rs`（play_bgm、stop_bgm、crossfade_bgm、play_sfx、update + 淡入淡出状态机）
- 核心类型：`AudioManager`
- 关键接口：`play_bgm`、`stop_bgm`、`crossfade_bgm`、`play_sfx`、`duck`、`unduck`、`update`

## KeyFlow

1. 根据运行模式（FS/ZIP）解析并加载音频输入源。
2. BGM 路径进入独立播放器并维护 `FadeState`。
3. 每帧 `update(dt)` 推进淡入淡出状态机与 duck multiplier 过渡。
4. SFX 采用一次性播放器通道播放并自动释放。
5. `duck()` / `unduck()` 通过独立的 `duck_multiplier` 平滑压低/恢复 BGM 音量（叠加于 FadeState 之上，互不干扰）。

## Dependencies

- 依赖 `rodio` 进行音频解码与播放
- 依赖 `resources::normalize_logical_path` 统一逻辑路径

## Invariants

- BGM 与 SFX 音量配置独立，静音状态统一影响有效输出。
- 淡入淡出状态在 `update` 中推进，避免多处并发改写。
- Duck multiplier 作为独立乘数叠加于所有 BGM 音量输出（含 FadeIn/FadeOut），不修改 `bgm_volume` 本身。

## FailureModes

- 音频设备不可用导致初始化失败。
- ZIP 模式未缓存音频字节导致播放失败。
- 文件不存在或解码失败导致播放降级。

## WhenToReadSource

- 需要调整 BGM 切换策略或淡入淡出曲线时。
- 需要排查不同资源模式下音频行为差异时。

## RelatedDocs

- [host 总览](../host.md)
- [resources 摘要](resources.md)
- [config 摘要](config.md)

## LastVerified

2026-03-11

## Owner

Ring-rs 维护者
