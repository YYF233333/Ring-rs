# host/video 摘要

## Purpose

`video` 负责 cutscene 播放：调用 FFmpeg 子进程解码视频/提取音轨，按时间戳推进帧，并在结束或跳过时清理资源。

## PublicSurface

- 入口：`host/src/video/mod.rs`
- 核心类型：`VideoPlayer`、`VideoDecoder`、`VideoAudio`、`VideoState`、`VideoError`
- 公共函数：`detect_ffmpeg()`

## KeyFlow

1. `app/update/script.rs` 先通过 `ResourceManager::materialize_to_fs()` 把逻辑路径解析成真实文件路径，再调用 `VideoPlayer::start()`。
2. `VideoPlayer::start()` 检查 FFmpeg 与文件存在性，启动解码器和音频提取线程，并记录 ZIP 模式下的临时文件。
3. `update(dt)` 依据 `elapsed` 消费当前帧与未来帧；音频提取完成后由上层驱动 `AudioManager::play_video_audio()`。
4. 窗口模式读取 `current_frame()` 上传到 GPU；结束或跳过后由 `finish_cutscene()` 清理播放器、恢复 BGM duck，并向 Runtime 回发 `SIGNAL_CUTSCENE`。
5. `detect_ffmpeg()` 的搜索顺序是 vendor 目录 -> 可执行文件同目录 -> `bin/` -> 系统 PATH。

## Invariants

- FFmpeg 不可用或文件缺失时应优雅降级，不得拖垮主循环。
- `VideoPlayer` 自身只管理播放状态与子进程资源，恢复 Runtime 的动作在 `app/update` 层完成。
- ZIP 模式下提取出的临时文件必须在清理阶段删除。

## WhenToReadSource

- 需要修改 cutscene 启停、跳过或结束恢复链路时。
- 需要排查 FFmpeg 查找、临时文件清理或音画同步时。
- 需要确认窗口渲染与 headless 对视频的处理差异时。

## RelatedDocs

- [host 总览](../host.md)
- [app_update 摘要](app-update.md)
- [audio 摘要](audio.md)
- [RFC-009: Cutscene 视频播放](../../../../../RFCs/Accepted/rfc-cutscene-video-playback.md)

## LastVerified

2026-03-24

## Owner

GPT-5.4
