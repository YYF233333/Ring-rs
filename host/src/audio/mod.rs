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
//! 音频字节统一由调用方通过 [`ResourceManager`](crate::resources::ResourceManager) 读取后，
//! 调用 [`AudioManager::cache_audio_bytes`] 预缓存。
//! `AudioManager` 不直接访问文件系统或 ZIP。

mod playback;

use rodio::{DeviceSinkBuilder, MixerDeviceSink, Player};
use std::collections::HashMap;
use tracing::debug;

/// 音频管理器
///
/// 负责管理 BGM 和 SFX 的播放状态。
/// 音频字节通过 [`cache_audio_bytes`](AudioManager::cache_audio_bytes) 注入，
/// 不直接持有文件系统路径或资源来源。
pub struct AudioManager {
    device_sink: Option<MixerDeviceSink>,
    bgm_sink: Option<Player>,
    current_bgm_path: Option<String>,
    bgm_volume: f32,
    sfx_volume: f32,
    muted: bool,
    fade_state: FadeState,
    /// 音频字节缓存（逻辑路径 -> 字节数据）
    audio_cache: HashMap<String, Vec<u8>>,
    duck_multiplier: f32,
    duck_target: f32,
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
    /// 创建新的音频管理器（连接真实音频设备）
    pub fn new() -> Result<Self, String> {
        let device_sink = DeviceSinkBuilder::open_default_sink()
            .map_err(|e| format!("Failed to initialize audio output: {}", e))?;

        Ok(Self {
            device_sink: Some(device_sink),
            bgm_sink: None,
            current_bgm_path: None,
            bgm_volume: 1.0,
            sfx_volume: 1.0,
            muted: false,
            fade_state: FadeState::None,
            audio_cache: HashMap::new(),
            duck_multiplier: 1.0,
            duck_target: 1.0,
        })
    }

    /// 创建 headless 音频管理器（无真实设备，仅追踪状态）
    pub fn new_headless() -> Self {
        Self {
            device_sink: None,
            bgm_sink: None,
            current_bgm_path: None,
            bgm_volume: 1.0,
            sfx_volume: 1.0,
            muted: false,
            fade_state: FadeState::None,
            audio_cache: HashMap::new(),
            duck_multiplier: 1.0,
            duck_target: 1.0,
        }
    }

    /// 预缓存音频字节数据。
    ///
    /// 调用方通过 ResourceManager 读取音频字节后，注入此缓存。
    pub fn cache_audio_bytes(&mut self, logical_path: &str, bytes: Vec<u8>) {
        self.audio_cache.insert(logical_path.to_string(), bytes);
    }

    /// 设置 BGM 音量
    pub fn set_bgm_volume(&mut self, volume: f32) {
        self.bgm_volume = volume.clamp(0.0, 1.0);

        if let Some(ref sink) = self.bgm_sink {
            sink.set_volume(self.get_effective_bgm_volume() * self.duck_multiplier);
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

        if let Some(ref sink) = self.bgm_sink {
            sink.set_volume(self.get_effective_bgm_volume() * self.duck_multiplier);
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

    /// 压低 BGM 音量（duck），平滑过渡到 30% 音量
    pub fn duck(&mut self) {
        self.duck_target = Self::DUCK_VOLUME_RATIO;
        debug!("BGM duck -> {:.0}%", self.duck_target * 100.0);
    }

    /// 恢复 BGM 音量（unduck），平滑过渡回正常音量
    pub fn unduck(&mut self) {
        self.duck_target = 1.0;
        debug!("BGM unduck -> 100%");
    }

    /// 获取有效的 BGM 音量（考虑静音状态，不含 duck）
    fn get_effective_bgm_volume(&self) -> f32 {
        if self.muted { 0.0 } else { self.bgm_volume }
    }

    /// 从 sink 当前音量反推 base volume（去除 duck_multiplier）
    fn current_base_bgm_volume(&self) -> f32 {
        let sink_vol = self
            .bgm_sink
            .as_ref()
            .map(|s| s.volume())
            .unwrap_or(self.get_effective_bgm_volume());
        if self.duck_multiplier > 0.001 {
            sink_vol / self.duck_multiplier
        } else {
            self.get_effective_bgm_volume()
        }
    }

    /// 播放视频音频（f32 PCM 样本，fire-and-forget）
    ///
    /// 创建 `rodio::buffer::SamplesBuffer` 并通过 mixer 播放。
    /// 返回 `Player` 句柄，调用方可用于提前停止。
    /// headless 模式下返回 `None`。
    pub fn play_video_audio(
        &self,
        samples: Vec<f32>,
        channels: u16,
        sample_rate: u32,
    ) -> Option<Player> {
        let device_sink = self.device_sink.as_ref()?;
        use std::num::NonZero;
        let source = rodio::buffer::SamplesBuffer::new(
            NonZero::new(channels).expect("invariant: channels > 0"),
            NonZero::new(sample_rate).expect("invariant: sample_rate > 0"),
            samples,
        );
        let player = Player::connect_new(device_sink.mixer());
        player.set_volume(self.sfx_volume);
        player.append(source);
        Some(player)
    }

    /// Duck 音量比例
    const DUCK_VOLUME_RATIO: f32 = 0.3;
    /// Duck 过渡速度（每秒变化量，越大越快）
    const DUCK_FADE_SPEED: f32 = 3.0;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_volume_settings() {
        if let Ok(mut manager) = AudioManager::new() {
            manager.set_bgm_volume(0.5);
            assert_eq!(manager.bgm_volume(), 0.5);

            manager.set_sfx_volume(0.7);
            assert_eq!(manager.sfx_volume(), 0.7);

            manager.set_bgm_volume(1.5);
            assert_eq!(manager.bgm_volume(), 1.0);

            manager.set_bgm_volume(-0.5);
            assert_eq!(manager.bgm_volume(), 0.0);
        }
    }

    #[test]
    fn test_mute_toggle() {
        if let Ok(mut manager) = AudioManager::new() {
            assert!(!manager.is_muted());
            manager.toggle_mute();
            assert!(manager.is_muted());
            manager.toggle_mute();
            assert!(!manager.is_muted());
        }
    }

    #[test]
    fn test_duck_unduck_state() {
        if let Ok(mut manager) = AudioManager::new() {
            assert_eq!(manager.duck_multiplier, 1.0);
            assert_eq!(manager.duck_target, 1.0);

            manager.duck();
            assert_eq!(manager.duck_target, AudioManager::DUCK_VOLUME_RATIO);
            // multiplier hasn't changed yet (needs update ticks)
            assert_eq!(manager.duck_multiplier, 1.0);

            // Simulate enough update ticks for convergence
            for _ in 0..100 {
                manager.update(0.05);
            }
            assert!((manager.duck_multiplier - AudioManager::DUCK_VOLUME_RATIO).abs() < 0.01);

            manager.unduck();
            assert_eq!(manager.duck_target, 1.0);

            for _ in 0..100 {
                manager.update(0.05);
            }
            assert!((manager.duck_multiplier - 1.0).abs() < 0.01);
        }
    }
}
