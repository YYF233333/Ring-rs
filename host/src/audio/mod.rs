//! # Audio 模块
//!
//! 音频管理系统，使用 rodio 库实现。
//! 支持 MP3, WAV, FLAC, OGG 格式。
//!
//! ## 功能特性
//!
//! - BGM 播放：支持循环、淡入淡出、切换
//! - SFX 播放：支持多音效同时播放
//! - 音量控制：独立的 BGM/SFX 音量设置
//!
//! ## 路径处理
//!
//! 音频路径使用**逻辑路径**（相对于 assets_root），由调用方负责规范化。
//! 内部根据 `use_zip_mode` 决定从文件系统还是临时文件加载。

use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink, Source};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Cursor};
use std::path::PathBuf;
use tracing::{debug, error};

/// 音频管理器
///
/// 负责管理 BGM 和 SFX 的播放状态。
pub struct AudioManager {
    /// 音频输出流（必须保持存活）
    _stream: OutputStream,
    /// 音频输出句柄
    stream_handle: OutputStreamHandle,
    /// BGM 播放器
    bgm_sink: Option<Sink>,
    /// 当前 BGM 路径（逻辑路径）
    current_bgm_path: Option<String>,
    /// BGM 主音量 (0.0 - 1.0)
    bgm_volume: f32,
    /// SFX 主音量 (0.0 - 1.0)
    sfx_volume: f32,
    /// 是否静音
    muted: bool,
    /// 淡入淡出状态
    fade_state: FadeState,
    /// 资源基础路径（文件系统模式使用）
    base_path: PathBuf,
    /// 是否使用 ZIP 模式
    use_zip_mode: bool,
    /// 音频字节缓存（逻辑路径 -> 字节数据）
    /// 用于 ZIP 模式，避免重复从 ResourceManager 读取
    audio_cache: HashMap<String, Vec<u8>>,
}

/// 淡入淡出状态
#[derive(Debug, Clone)]
enum FadeState {
    /// 无淡入淡出
    None,
    /// 淡入中
    FadeIn {
        /// 目标音量
        target_volume: f32,
        /// 当前音量
        current_volume: f32,
        /// 每秒增加的音量
        rate: f32,
    },
    /// 淡出中
    FadeOut {
        /// 当前音量
        current_volume: f32,
        /// 每秒减少的音量
        rate: f32,
        /// 淡出完成后是否停止
        stop_after: bool,
        /// 淡出完成后要播放的新 BGM（如果有）
        next_bgm: Option<(String, bool)>,
    },
}

impl AudioManager {
    /// 创建新的音频管理器（文件系统模式）
    pub fn new(base_path: &str) -> Result<Self, String> {
        let (stream, stream_handle) =
            OutputStream::try_default().map_err(|e| format!("无法初始化音频输出: {}", e))?;

        Ok(Self {
            _stream: stream,
            stream_handle,
            bgm_sink: None,
            current_bgm_path: None,
            bgm_volume: 1.0,
            sfx_volume: 1.0,
            muted: false,
            fade_state: FadeState::None,
            base_path: PathBuf::from(base_path),
            use_zip_mode: false,
            audio_cache: HashMap::new(),
        })
    }

    /// 创建 ZIP 模式的音频管理器
    pub fn new_zip_mode(base_path: &str) -> Result<Self, String> {
        let (stream, stream_handle) =
            OutputStream::try_default().map_err(|e| format!("无法初始化音频输出: {}", e))?;

        Ok(Self {
            _stream: stream,
            stream_handle,
            bgm_sink: None,
            current_bgm_path: None,
            bgm_volume: 1.0,
            sfx_volume: 1.0,
            muted: false,
            fade_state: FadeState::None,
            base_path: PathBuf::from(base_path),
            use_zip_mode: true,
            audio_cache: HashMap::new(),
        })
    }

    /// 预加载音频字节数据（用于 ZIP 模式）
    ///
    /// 在 ZIP 模式下，需要先通过 ResourceManager 读取音频字节，
    /// 然后调用此方法缓存数据。
    pub fn cache_audio_bytes(&mut self, logical_path: &str, bytes: Vec<u8>) {
        self.audio_cache.insert(logical_path.to_string(), bytes);
    }

    /// 解析音频路径到完整文件系统路径（仅文件系统模式使用）
    fn resolve_fs_path(&self, logical_path: &str) -> PathBuf {
        use crate::resources::normalize_logical_path;

        // 规范化逻辑路径
        let normalized = normalize_logical_path(logical_path);

        // 拼接 base_path
        self.base_path.join(&normalized)
    }

    /// 播放 BGM
    ///
    /// # 参数
    ///
    /// - `path`: BGM 逻辑路径（相对于 assets_root，如 `bgm/music.mp3`）
    /// - `looping`: 是否循环
    /// - `fade_in`: 淡入时长（秒），None 表示立即播放
    pub fn play_bgm(&mut self, path: &str, looping: bool, fade_in: Option<f32>) {
        use crate::resources::normalize_logical_path;

        // 如果当前有 BGM 在播放，先停止
        if let Some(ref sink) = self.bgm_sink {
            sink.stop();
        }

        // 规范化路径
        let logical_path = normalize_logical_path(path);

        // 根据模式加载音频
        let source: Box<dyn Source<Item = i16> + Send> = if self.use_zip_mode {
            // ZIP 模式：从缓存读取字节
            let bytes = match self.audio_cache.get(&logical_path) {
                Some(b) => b.clone(),
                None => {
                    error!(
                        path = %logical_path,
                        "音频未缓存 (请先调用 cache_audio_bytes)"
                    );
                    return;
                }
            };

            let cursor = Cursor::new(bytes);
            match Decoder::new(cursor) {
                Ok(s) => Box::new(s.convert_samples::<i16>()),
                Err(e) => {
                    error!(path = %logical_path, error = %e, "无法解码音频");
                    return;
                }
            }
        } else {
            // 文件系统模式：直接读取文件
            let full_path = self.resolve_fs_path(&logical_path);

            let file = match File::open(&full_path) {
                Ok(f) => f,
                Err(e) => {
                    error!(path = %full_path.display(), error = %e, "无法打开音频文件");
                    return;
                }
            };

            match Decoder::new(BufReader::new(file)) {
                Ok(s) => Box::new(s.convert_samples::<i16>()),
                Err(e) => {
                    error!(path = %full_path.display(), error = %e, "无法解码音频文件");
                    return;
                }
            }
        };

        // 创建新的 Sink
        let sink = match Sink::try_new(&self.stream_handle) {
            Ok(s) => s,
            Err(e) => {
                error!(error = %e, "无法创建音频播放器");
                return;
            }
        };

        // 设置初始音量
        let initial_volume = if fade_in.is_some() {
            0.0
        } else {
            self.get_effective_bgm_volume()
        };
        sink.set_volume(initial_volume);

        // 添加音源（循环或单次）
        if looping {
            sink.append(source.repeat_infinite());
        } else {
            sink.append(source);
        }

        self.bgm_sink = Some(sink);
        self.current_bgm_path = Some(logical_path.clone());

        // 设置淡入状态
        if let Some(duration) = fade_in
            && duration > 0.0
        {
            self.fade_state = FadeState::FadeIn {
                target_volume: self.get_effective_bgm_volume(),
                current_volume: 0.0,
                rate: self.get_effective_bgm_volume() / duration,
            };
        }

        debug!(
            path = %logical_path,
            looping = looping,
            fade_in = ?fade_in,
            "开始播放 BGM"
        );
    }

    /// 停止 BGM
    ///
    /// # 参数
    ///
    /// - `fade_out`: 淡出时长（秒），None 表示立即停止
    pub fn stop_bgm(&mut self, fade_out: Option<f32>) {
        if self.bgm_sink.is_none() {
            return;
        }

        if let Some(duration) = fade_out
            && duration > 0.0
        {
            let current_volume = self
                .bgm_sink
                .as_ref()
                .map(|s| s.volume())
                .unwrap_or(self.get_effective_bgm_volume());

            self.fade_state = FadeState::FadeOut {
                current_volume,
                rate: current_volume / duration,
                stop_after: true,
                next_bgm: None,
            };
            debug!(duration = duration, "BGM 淡出中");
            return;
        }

        // 立即停止
        if let Some(ref sink) = self.bgm_sink {
            sink.stop();
        }
        self.bgm_sink = None;
        self.current_bgm_path = None;
        self.fade_state = FadeState::None;
        debug!("BGM 已停止");
    }

    /// 切换 BGM（带交叉淡入淡出）
    ///
    /// # 参数
    ///
    /// - `path`: 新 BGM 路径
    /// - `looping`: 是否循环
    /// - `fade_duration`: 交叉淡入淡出时长（秒）
    pub fn crossfade_bgm(&mut self, path: &str, looping: bool, fade_duration: f32) {
        if self.bgm_sink.is_none() {
            // 没有当前 BGM，直接播放新的（带淡入）
            self.play_bgm(path, looping, Some(fade_duration));
            return;
        }

        // 设置淡出状态，并记录要播放的新 BGM
        let current_volume = self
            .bgm_sink
            .as_ref()
            .map(|s| s.volume())
            .unwrap_or(self.get_effective_bgm_volume());

        self.fade_state = FadeState::FadeOut {
            current_volume,
            rate: current_volume / fade_duration,
            stop_after: false,
            next_bgm: Some((path.to_string(), looping)),
        };

        debug!(duration = fade_duration, "BGM 切换: 淡出中");
    }

    /// 播放音效
    ///
    /// # 参数
    ///
    /// - `path`: 音效逻辑路径（相对于 assets_root，如 `sfx/click.mp3`）
    pub fn play_sfx(&self, path: &str) {
        use crate::resources::normalize_logical_path;

        if self.muted {
            return;
        }

        // 规范化路径
        let logical_path = normalize_logical_path(path);

        // 根据模式加载音频
        let source: Box<dyn Source<Item = i16> + Send> = if self.use_zip_mode {
            // ZIP 模式：从缓存读取字节
            let bytes = match self.audio_cache.get(&logical_path) {
                Some(b) => b.clone(),
                None => {
                    error!(
                        path = %logical_path,
                        "音效未缓存 (请先调用 cache_audio_bytes)"
                    );
                    return;
                }
            };

            let cursor = Cursor::new(bytes);
            match Decoder::new(cursor) {
                Ok(s) => Box::new(s.convert_samples::<i16>()),
                Err(e) => {
                    error!(path = %logical_path, error = %e, "无法解码音效");
                    return;
                }
            }
        } else {
            // 文件系统模式：直接读取文件
            let full_path = self.resolve_fs_path(&logical_path);

            let file = match File::open(&full_path) {
                Ok(f) => f,
                Err(e) => {
                    error!(path = %full_path.display(), error = %e, "无法打开音效文件");
                    return;
                }
            };

            match Decoder::new(BufReader::new(file)) {
                Ok(s) => Box::new(s.convert_samples::<i16>()),
                Err(e) => {
                    error!(path = %full_path.display(), error = %e, "无法解码音效文件");
                    return;
                }
            }
        };

        // 创建一次性播放器
        if let Ok(sink) = Sink::try_new(&self.stream_handle) {
            sink.set_volume(self.sfx_volume);
            sink.append(source);
            sink.detach(); // 分离后自动播放完毕
            debug!(path = %logical_path, "播放音效");
        }
    }

    /// 更新音频状态（每帧调用）
    ///
    /// # 参数
    ///
    /// - `dt`: 帧间隔时间（秒）
    pub fn update(&mut self, dt: f32) {
        // 收集需要延后执行的操作
        let mut next_bgm_to_play: Option<(String, bool, f32)> = None;
        let mut fade_completed = false;
        let mut should_stop = false;

        match &mut self.fade_state {
            FadeState::None => {}
            FadeState::FadeIn {
                target_volume,
                current_volume,
                rate,
            } => {
                *current_volume += *rate * dt;
                if *current_volume >= *target_volume {
                    // 淡入完成
                    if let Some(ref sink) = self.bgm_sink {
                        sink.set_volume(*target_volume);
                    }
                    fade_completed = true;
                    debug!("BGM 淡入完成");
                } else {
                    // 更新音量
                    if let Some(ref sink) = self.bgm_sink {
                        sink.set_volume(*current_volume);
                    }
                }
            }
            FadeState::FadeOut {
                current_volume,
                rate,
                stop_after,
                next_bgm,
            } => {
                *current_volume -= *rate * dt;
                if *current_volume <= 0.0 {
                    // 淡出完成
                    if let Some((path, looping)) = next_bgm.take() {
                        let duration = if *rate > 0.0 { 1.0 / *rate } else { 0.5 };
                        next_bgm_to_play = Some((path, looping, duration));
                    }
                    should_stop = *stop_after;
                    fade_completed = true;
                } else {
                    // 更新音量
                    if let Some(ref sink) = self.bgm_sink {
                        sink.set_volume(*current_volume);
                    }
                }
            }
        }

        // 在 match 结束后执行延后操作
        if fade_completed {
            self.fade_state = FadeState::None;

            if should_stop {
                if let Some(ref sink) = self.bgm_sink {
                    sink.stop();
                }
                self.bgm_sink = None;
                self.current_bgm_path = None;
                debug!("BGM 淡出完成，已停止");
            }
        }

        if let Some((path, looping, duration)) = next_bgm_to_play {
            // 先停止当前 BGM
            if let Some(ref sink) = self.bgm_sink {
                sink.stop();
            }
            self.bgm_sink = None;
            self.current_bgm_path = None;

            // 播放新 BGM（带淡入）
            self.play_bgm(&path, looping, Some(duration));
        }
    }

    /// 设置 BGM 音量
    pub fn set_bgm_volume(&mut self, volume: f32) {
        self.bgm_volume = volume.clamp(0.0, 1.0);

        // 更新当前 BGM 的音量
        if let Some(ref sink) = self.bgm_sink {
            let effective_volume = if self.muted { 0.0 } else { self.bgm_volume };
            sink.set_volume(effective_volume);
        }
    }

    /// 设置 SFX 音量
    pub fn set_sfx_volume(&mut self, volume: f32) {
        self.sfx_volume = volume.clamp(0.0, 1.0);
    }

    /// 获取 BGM 音量
    pub fn bgm_volume(&self) -> f32 {
        self.bgm_volume
    }

    /// 获取 SFX 音量
    pub fn sfx_volume(&self) -> f32 {
        self.sfx_volume
    }

    /// 设置静音状态
    pub fn set_muted(&mut self, muted: bool) {
        self.muted = muted;

        // 更新当前 BGM 的音量
        if let Some(ref sink) = self.bgm_sink {
            let effective_volume = if muted { 0.0 } else { self.bgm_volume };
            sink.set_volume(effective_volume);
        }
    }

    /// 切换静音状态
    pub fn toggle_mute(&mut self) {
        self.set_muted(!self.muted);
    }

    /// 是否静音
    pub fn is_muted(&self) -> bool {
        self.muted
    }

    /// 是否正在播放 BGM
    pub fn is_bgm_playing(&self) -> bool {
        self.bgm_sink.as_ref().map(|s| !s.empty()).unwrap_or(false)
    }

    /// 获取当前 BGM 路径
    pub fn current_bgm_path(&self) -> Option<&str> {
        self.current_bgm_path.as_deref()
    }

    /// 暂停 BGM
    pub fn pause_bgm(&self) {
        if let Some(ref sink) = self.bgm_sink {
            sink.pause();
            debug!("BGM 已暂停");
        }
    }

    /// 恢复 BGM
    pub fn resume_bgm(&self) {
        if let Some(ref sink) = self.bgm_sink {
            sink.play();
            debug!("BGM 已恢复");
        }
    }

    /// 获取有效的 BGM 音量（考虑静音状态）
    fn get_effective_bgm_volume(&self) -> f32 {
        if self.muted { 0.0 } else { self.bgm_volume }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_volume_settings() {
        // 注意：这个测试可能在没有音频设备的环境下失败
        if let Ok(mut manager) = AudioManager::new("assets") {
            manager.set_bgm_volume(0.5);
            assert_eq!(manager.bgm_volume(), 0.5);

            manager.set_sfx_volume(0.7);
            assert_eq!(manager.sfx_volume(), 0.7);

            // 测试音量限制
            manager.set_bgm_volume(1.5);
            assert_eq!(manager.bgm_volume(), 1.0);

            manager.set_bgm_volume(-0.5);
            assert_eq!(manager.bgm_volume(), 0.0);
        }
    }

    #[test]
    fn test_mute_toggle() {
        if let Ok(mut manager) = AudioManager::new("assets") {
            assert!(!manager.is_muted());
            manager.toggle_mute();
            assert!(manager.is_muted());
            manager.toggle_mute();
            assert!(!manager.is_muted());
        }
    }
}
