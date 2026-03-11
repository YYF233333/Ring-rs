//! # Video 模块
//!
//! Cutscene 视频播放系统，使用 FFmpeg 子进程解码视频帧，rodio 播放音频轨。
//!
//! ## 架构
//!
//! - `VideoDecoder`：后台线程通过 ffmpeg-sidecar 解码视频帧（FFmpeg 直出 RGBA）
//! - `VideoAudio`：后台线程通过 FFmpeg 子进程提取 PCM 音频，rodio 播放
//! - `VideoPlayer`：状态机，编排解码与播放，按时间戳调度帧显示
//!
//! ## FFmpeg 依赖
//!
//! 运行时需要 FFmpeg 二进制。检测顺序：
//! `vendor/ffmpeg/{platform}/` → `bin/` → 系统 PATH。
//! 不可用时优雅降级（跳过视频，不崩溃）。

mod audio;
mod decoder;

use std::path::{Path, PathBuf};

use thiserror::Error;
use tracing::{debug, info, warn};

pub use audio::VideoAudio;
pub use decoder::VideoDecoder;

/// 视频播放错误
#[derive(Debug, Error)]
pub enum VideoError {
    #[error("FFmpeg binary not found; video playback requires FFmpeg")]
    FfmpegNotFound,
    #[error("video file not found: {0}")]
    FileNotFound(String),
    #[error("FFmpeg process error: {0}")]
    ProcessError(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// 解码后的视频帧
pub struct VideoFrame {
    pub width: u32,
    pub height: u32,
    /// RGBA 像素数据 (width * height * 4 bytes)
    pub data: Vec<u8>,
    /// 帧时间戳（秒）
    pub timestamp: f32,
}

/// 视频播放器状态
#[derive(Debug, Clone, PartialEq)]
pub enum VideoState {
    Idle,
    Playing,
    Finished,
    Skipped,
}

/// Cutscene 视频播放器
///
/// 管理 FFmpeg 视频解码子进程和音频播放，
/// 按时间戳调度帧显示，支持跳过。
pub struct VideoPlayer {
    state: VideoState,
    decoder: Option<VideoDecoder>,
    audio: Option<VideoAudio>,
    elapsed: f32,
    current_frame: Option<VideoFrame>,
    pending_frame: Option<VideoFrame>,
    /// ZIP 模式下提取的临时视频文件路径，cleanup 时删除
    temp_video_file: Option<PathBuf>,
}

impl VideoPlayer {
    pub fn new() -> Self {
        Self {
            state: VideoState::Idle,
            decoder: None,
            audio: None,
            elapsed: 0.0,
            current_frame: None,
            pending_frame: None,
            temp_video_file: None,
        }
    }

    /// 开始播放视频文件。
    ///
    /// `resolved_path` 必须是真实文件系统路径（FS 模式为 assets 下的路径，
    /// ZIP 模式为调用方提取到临时文件后的路径）。
    /// `temp_file` 为 ZIP 模式下需要在播放结束后清理的临时文件路径。
    pub fn start(
        &mut self,
        resolved_path: &Path,
        temp_file: Option<PathBuf>,
    ) -> Result<(), VideoError> {
        let ffmpeg_path = detect_ffmpeg().ok_or(VideoError::FfmpegNotFound)?;

        if !resolved_path.exists() {
            return Err(VideoError::FileNotFound(
                resolved_path.to_string_lossy().to_string(),
            ));
        }

        let path_str = resolved_path.to_string_lossy().to_string();
        info!(path = %path_str, "Starting cutscene video playback");

        self.temp_video_file = temp_file;

        // Ensure FFmpeg is discoverable by FfmpegCommand
        ensure_ffmpeg_in_path(&ffmpeg_path);

        let decoder = VideoDecoder::start(&path_str)?;
        let audio = match VideoAudio::start_extraction(&path_str, &ffmpeg_path) {
            Ok(a) => Some(a),
            Err(e) => {
                warn!(error = %e, "Failed to extract video audio, playing silent");
                None
            }
        };

        self.state = VideoState::Playing;
        self.decoder = Some(decoder);
        self.audio = audio;
        self.elapsed = 0.0;
        self.current_frame = None;
        self.pending_frame = None;

        Ok(())
    }

    /// 推进播放 dt 秒。
    ///
    /// 解码帧至当前时间戳，返回 true 表示仍在播放。
    pub fn update(&mut self, dt: f32) -> bool {
        if self.state != VideoState::Playing {
            return false;
        }

        self.elapsed += dt;

        if let Some(audio) = &mut self.audio {
            audio.try_start_playback();
        }

        let decoder = match &self.decoder {
            Some(d) => d,
            None => {
                self.state = VideoState::Finished;
                return false;
            }
        };

        // 检查缓冲的未来帧是否到达显示时间
        if self
            .pending_frame
            .as_ref()
            .is_some_and(|f| f.timestamp <= self.elapsed)
        {
            self.current_frame = self.pending_frame.take();
        }

        // 从解码器消费帧至当前时间戳
        while self.pending_frame.is_none() {
            match decoder.next_frame() {
                Some(frame) => {
                    if frame.timestamp <= self.elapsed {
                        // 此帧应该已经显示或已过期，更新为当前帧
                        self.current_frame = Some(frame);
                    } else {
                        // 此帧在未来，缓冲等待
                        self.pending_frame = Some(frame);
                    }
                }
                None => {
                    if decoder.is_finished() && self.pending_frame.is_none() {
                        self.state = VideoState::Finished;
                        debug!(elapsed = self.elapsed, "Cutscene playback finished");
                        self.cleanup_internal();
                        return false;
                    }
                    break;
                }
            }
        }

        true
    }

    /// 跳过当前视频。
    pub fn skip(&mut self) {
        if self.state == VideoState::Playing {
            info!("Cutscene video skipped");
            self.state = VideoState::Skipped;
            self.cleanup_internal();
        }
    }

    /// 获取当前帧的像素数据用于渲染。
    pub fn current_frame(&self) -> Option<&VideoFrame> {
        self.current_frame.as_ref()
    }

    /// 视频是否已结束（播完或跳过）。
    pub fn is_done(&self) -> bool {
        matches!(self.state, VideoState::Finished | VideoState::Skipped)
    }

    /// 视频是否正在播放。
    pub fn is_playing(&self) -> bool {
        self.state == VideoState::Playing
    }

    pub fn state(&self) -> &VideoState {
        &self.state
    }

    /// 获取音频子系统的可变引用（用于检查/启动播放）。
    pub fn audio_mut(&mut self) -> Option<&mut VideoAudio> {
        self.audio.as_mut()
    }

    /// 清理所有资源，重置为 Idle。
    pub fn cleanup(&mut self) {
        self.cleanup_internal();
        self.state = VideoState::Idle;
    }

    fn cleanup_internal(&mut self) {
        if let Some(mut decoder) = self.decoder.take() {
            decoder.stop();
        }
        if let Some(mut audio) = self.audio.take() {
            audio.stop();
        }
        self.current_frame = None;
        self.pending_frame = None;

        if let Some(temp_path) = self.temp_video_file.take()
            && let Err(e) = std::fs::remove_file(&temp_path)
        {
            debug!(error = %e, path = %temp_path.display(),
                    "Failed to clean up temp video file");
        }
    }
}

/// 检测 FFmpeg 二进制可用性。
///
/// 搜索顺序：
/// 1. `vendor/ffmpeg/{platform}/ffmpeg[.exe]`
/// 2. 当前可执行文件同目录（发布模式）
/// 3. `bin/ffmpeg[.exe]`
/// 4. 系统 PATH
pub fn detect_ffmpeg() -> Option<PathBuf> {
    let exe_name = if cfg!(windows) {
        "ffmpeg.exe"
    } else {
        "ffmpeg"
    };

    // 1. vendor 目录（开发模式）
    let vendor_dir = if cfg!(windows) {
        "vendor/ffmpeg/win-x64"
    } else if cfg!(target_os = "macos") {
        "vendor/ffmpeg/macos-x64"
    } else {
        "vendor/ffmpeg/linux-x64"
    };
    let vendor_path = PathBuf::from(vendor_dir).join(exe_name);
    if vendor_path.exists() {
        debug!(path = %vendor_path.display(), "Found FFmpeg in vendor directory");
        return Some(vendor_path);
    }

    // 2. 可执行文件同目录（发布模式）
    if let Ok(exe_path) = std::env::current_exe()
        && let Some(exe_dir) = exe_path.parent()
    {
        let beside_exe = exe_dir.join(exe_name);
        if beside_exe.exists() {
            debug!(path = %beside_exe.display(), "Found FFmpeg beside executable");
            return Some(beside_exe);
        }
    }

    // 3. bin 目录
    let bin_path = PathBuf::from("bin").join(exe_name);
    if bin_path.exists() {
        debug!(path = %bin_path.display(), "Found FFmpeg in bin directory");
        return Some(bin_path);
    }

    // 4. 系统 PATH
    match std::process::Command::new(exe_name)
        .arg("-version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
    {
        Ok(status) if status.success() => {
            debug!("Found FFmpeg in system PATH");
            Some(PathBuf::from(exe_name))
        }
        _ => {
            warn!("FFmpeg not found; cutscene video playback unavailable");
            None
        }
    }
}

/// 将 FFmpeg 所在目录添加到 PATH，使 FfmpegCommand 能找到它。
fn ensure_ffmpeg_in_path(ffmpeg_path: &Path) {
    let Some(parent) = ffmpeg_path.parent() else {
        return;
    };
    if parent.as_os_str().is_empty() {
        return; // already just "ffmpeg" from PATH
    }

    if let Some(current_path) = std::env::var_os("PATH") {
        let parent_str = parent.as_os_str();
        // 避免重复添加
        let separator = if cfg!(windows) { ";" } else { ":" };
        let current = current_path.to_string_lossy();
        let parent_s = parent_str.to_string_lossy();
        if !current.contains(&*parent_s) {
            let mut new_path = std::ffi::OsString::from(parent_str);
            new_path.push(separator);
            new_path.push(&current_path);
            // SAFETY: 单进程 VN 游戏，修改 PATH 无并发风险
            unsafe { std::env::set_var("PATH", &new_path) };
            debug!(dir = %parent_s, "Added FFmpeg directory to PATH");
        }
    }
}
