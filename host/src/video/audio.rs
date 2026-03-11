//! 视频音频提取与播放。
//!
//! 通过 FFmpeg 子进程将视频的音频轨提取为 f32le PCM，
//! 收集完成后通过 rodio SamplesBuffer 播放。

use std::io::Read;
use std::path::Path;
use std::process::{Command, Stdio};
use std::thread::{self, JoinHandle};

use tracing::{debug, error, warn};

use super::VideoError;

const SAMPLE_RATE: u32 = 44100;
const CHANNELS: u16 = 2;

/// 视频音频播放器
///
/// 后台线程提取音频 PCM 数据。提取完成后通过 `try_start_playback()` 启动播放。
pub struct VideoAudio {
    extraction_thread: Option<JoinHandle<Result<Vec<f32>, String>>>,
    /// 提取完成的音频样本（等待播放）
    samples: Option<Vec<f32>>,
    /// 是否已开始播放
    playback_started: bool,
}

impl VideoAudio {
    /// 启动音频提取（后台线程）。
    ///
    /// FFmpeg 将视频的音频轨转为 f32le PCM 输出到 stdout，
    /// 后台线程读取全部数据后存储待播放。
    pub fn start_extraction(video_path: &str, ffmpeg_path: &Path) -> Result<Self, VideoError> {
        let path = video_path.to_string();
        let ffmpeg = ffmpeg_path.to_path_buf();

        let handle = thread::spawn(move || Self::extract_audio_pcm(&path, &ffmpeg));

        Ok(Self {
            extraction_thread: Some(handle),
            samples: None,
            playback_started: false,
        })
    }

    fn extract_audio_pcm(video_path: &str, ffmpeg_path: &Path) -> Result<Vec<f32>, String> {
        let mut cmd = Command::new(ffmpeg_path);
        cmd.args([
            "-i",
            video_path,
            "-vn", // 禁用视频
            "-f",
            "f32le",
            "-acodec",
            "pcm_f32le",
            "-ac",
            &CHANNELS.to_string(),
            "-ar",
            &SAMPLE_RATE.to_string(),
            "-v",
            "quiet",
            "pipe:1",
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::null());

        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            const CREATE_NO_WINDOW: u32 = 0x08000000;
            cmd.creation_flags(CREATE_NO_WINDOW);
        }

        let mut child = cmd
            .spawn()
            .map_err(|e| format!("Failed to spawn FFmpeg audio extraction: {e}"))?;

        let mut pcm_bytes = Vec::new();
        if let Some(mut stdout) = child.stdout.take() {
            stdout
                .read_to_end(&mut pcm_bytes)
                .map_err(|e| format!("Failed to read audio PCM: {e}"))?;
        }

        let status = child
            .wait()
            .map_err(|e| format!("Failed to wait for FFmpeg audio: {e}"))?;
        if !status.success() {
            return Err(format!(
                "FFmpeg audio extraction failed with exit code: {:?}",
                status.code()
            ));
        }

        if pcm_bytes.is_empty() {
            debug!("Video has no audio track");
            return Ok(Vec::new());
        }

        // f32le bytes → Vec<f32>
        let samples: Vec<f32> = pcm_bytes
            .chunks_exact(4)
            .map(|chunk| f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
            .collect();

        debug!(
            sample_count = samples.len(),
            duration_secs = samples.len() as f32 / (SAMPLE_RATE as f32 * CHANNELS as f32),
            "Audio extraction complete"
        );

        Ok(samples)
    }

    /// 检查音频提取是否完成，如完成则缓存样本数据。
    ///
    /// 由 `VideoPlayer::update()` 每帧调用。
    /// 实际播放需要外部传入 rodio mixer（Phase 2 集成时实现）。
    pub fn try_start_playback(&mut self) {
        if self.playback_started || self.samples.is_some() {
            return;
        }

        let Some(handle) = &self.extraction_thread else {
            return;
        };

        if !handle.is_finished() {
            return;
        }

        // 线程已完成，取出结果
        if let Some(handle) = self.extraction_thread.take() {
            match handle.join() {
                Ok(Ok(samples)) => {
                    if samples.is_empty() {
                        debug!("No audio samples to play");
                        self.playback_started = true;
                    } else {
                        debug!(samples = samples.len(), "Audio samples ready for playback");
                        self.samples = Some(samples);
                    }
                }
                Ok(Err(e)) => {
                    warn!(error = %e, "Audio extraction failed");
                    self.playback_started = true;
                }
                Err(_) => {
                    error!("Audio extraction thread panicked");
                    self.playback_started = true;
                }
            }
        }
    }

    /// 获取已提取的音频样本（消耗）。
    ///
    /// Phase 2 中 Host 集成代码调用此方法获取样本数据，
    /// 然后通过 AudioManager 的 mixer 创建 SamplesBuffer 播放。
    pub fn take_samples(&mut self) -> Option<Vec<f32>> {
        let samples = self.samples.take();
        if samples.is_some() {
            self.playback_started = true;
        }
        samples
    }

    /// 音频采样率。
    pub fn sample_rate(&self) -> u32 {
        SAMPLE_RATE
    }

    /// 音频声道数。
    pub fn channels(&self) -> u16 {
        CHANNELS
    }

    /// 停止音频提取和播放。
    pub fn stop(&mut self) {
        self.samples = None;
        // 提取线程自然结束（读完 stdout 或 FFmpeg 退出）
        if let Some(handle) = self.extraction_thread.take() {
            let _ = handle.join();
        }
    }
}
