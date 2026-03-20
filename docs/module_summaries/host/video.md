# host/video 摘要

## Purpose

`video` 模块负责 cutscene 视频播放：通过 FFmpeg 子进程解码视频帧和提取音频，按时间戳调度帧显示，支持跳过和优雅降级。

## PublicSurface

- 模块入口：`host/src/video/mod.rs`
- 关键类型：`VideoPlayer`（状态机编排）、`VideoDecoder`（帧解码）、`VideoAudio`（音频提取）、`VideoFrame`、`VideoState`、`VideoError`
- 公共函数：`detect_ffmpeg()`（FFmpeg 二进制检测）

## KeyFlow

1. `script.rs` 拦截 `Command::Cutscene`，通过 `resolve_video_path()` 解析视频路径：
   - 使用 `ResourceManager::materialize_to_fs()` 统一处理 FS/ZIP 模式。
   - FS 模式：返回底层文件路径，无需清理。
   - ZIP 模式：提取到临时文件（`%TEMP%/ring-vn-video/`），返回临时路径。
2. 调用 `VideoPlayer::start(resolved_path, temp_file)` 启动播放。`start()` 接收已解析的真实文件系统路径，检测 FFmpeg、验证文件存在，启动 `VideoDecoder`（后台线程 ffmpeg-sidecar -> RGB24 -> RGBA via mpsc channel）和 `VideoAudio`（后台线程 FFmpeg -> f32le PCM）。
3. 每帧 `update(dt)` 推进 elapsed 时间，从 decoder channel 消费帧至当前时间戳。
4. `update/mod.rs` 中 `try_start_video_audio()` 检查音频提取完成，通过 `AudioManager::play_video_audio()` 播放。
5. `host_app.rs` 中将当前帧 RGBA 数据上传到 `WgpuBackend` 视频纹理，生成全屏 `DrawCommand::Sprite`（信箱模式保持宽高比）。
6. 播放完成/跳过 → `finish_cutscene()` 清理资源（含 ZIP 模式临时文件）、unduck BGM、发送 `SIGNAL_CUTSCENE` 恢复 Runtime。

## Dependencies

- `ffmpeg-sidecar`：FFmpeg 子进程封装（视频帧解码）
- `std::process::Command`：FFmpeg 音频提取
- `rodio`：通过 `AudioManager` 播放 PCM 音频
- `host/backend`：`WgpuBackend` 视频纹理上传与渲染

## Invariants

- FFmpeg 不可用或文件不存在时优雅降级（warn + 跳过），不崩溃。
- `VideoPlayer` 是状态机（Idle/Playing/Finished/Skipped），所有终态通过 `is_done()` 检测。
- 后台线程通过 `stop()` / `Drop` 确保清理，不泄漏子进程。
- Windows 平台使用 `CREATE_NO_WINDOW` 防止 FFmpeg 弹出控制台窗口。
- ZIP 模式下的临时视频文件在 `cleanup_internal()` 中删除（`temp_video_file` 字段追踪）。

## FailureModes

- FFmpeg 二进制不在 vendor/bin/PATH 中 → `VideoError::FfmpegNotFound` → 跳过
- 视频文件不存在 → `VideoError::FileNotFound` → 跳过
- FFmpeg 子进程崩溃 → decoder 线程记录错误 + finished 标记 → 播放结束
- 音频提取失败 → 静音播放视频

## WhenToReadSource

- 修改视频播放行为（帧调度、跳过逻辑）时。
- 调试 FFmpeg 子进程问题或音频同步问题时。
- 扩展支持新的视频格式时。

## RelatedDocs

- [RFC-009: Cutscene 视频播放](../../../RFCs/rfc-cutscene-video-playback.md)
- [host 总览](../host.md)
- [app_update 摘要](app_update.md)
- [audio 摘要](audio.md)
- [仓库导航地图](../../navigation_map.md)

## LastVerified

2026-03-18

## Owner

Composer