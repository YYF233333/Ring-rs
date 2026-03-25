//! 音频管理系统
//!
//! 使用 rodio 实现 BGM/SFX 播放、淡入淡出、duck。
//! `AudioManager` 不直接访问文件系统，音频字节通过 [`cache_audio_bytes`](AudioManager::cache_audio_bytes) 注入。

use rodio::{Decoder, DeviceSinkBuilder, MixerDeviceSink, Player, Source};
use std::collections::HashMap;
use std::io::Cursor;
use tracing::{debug, error};

use crate::resources::normalize_logical_path;

/// 音频管理器
///
/// 管理 BGM 和 SFX 播放状态。
/// 音频字节通过 [`cache_audio_bytes`](AudioManager::cache_audio_bytes) 注入，不直接持有文件路径。
pub struct AudioManager {
    device_sink: Option<MixerDeviceSink>,
    bgm_sink: Option<Player>,
    current_bgm_path: Option<String>,
    bgm_volume: f32,
    sfx_volume: f32,
    muted: bool,
    fade_state: FadeState,
    audio_cache: HashMap<String, Vec<u8>>,
    duck_multiplier: f32,
    duck_target: f32,
}

// rodio 的 OutputStream/MixerDeviceSink 内部使用线程安全机制，
// `!Send` 约束来自 cpal::Stream 的保守限制。
// Tauri 需要通过 Mutex 跨线程共享，此处放宽约束。
unsafe impl Send for AudioManager {}

/// 淡入淡出状态
#[derive(Debug, Clone)]
enum FadeState {
    None,
    FadeIn {
        target_volume: f32,
        current_volume: f32,
        rate: f32,
    },
    FadeOut {
        current_volume: f32,
        rate: f32,
        stop_after: bool,
        next_bgm: Option<(String, bool)>,
    },
}

impl AudioManager {
    const DUCK_VOLUME_RATIO: f32 = 0.3;
    const DUCK_FADE_SPEED: f32 = 3.0;

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

    /// 预缓存音频字节数据
    pub fn cache_audio_bytes(&mut self, logical_path: &str, bytes: Vec<u8>) {
        self.audio_cache.insert(logical_path.to_string(), bytes);
    }

    /// 设置 BGM 音量
    pub fn set_bgm_volume(&mut self, volume: f32) {
        self.bgm_volume = volume.clamp(0.0, 1.0);
        if let Some(ref sink) = self.bgm_sink {
            sink.set_volume(self.effective_bgm_volume() * self.duck_multiplier);
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

    /// 是否正在播放 BGM
    pub fn is_bgm_playing(&self) -> bool {
        self.bgm_sink.as_ref().map(|s| !s.empty()).unwrap_or(false)
    }

    /// 获取当前 BGM 路径
    pub fn current_bgm_path(&self) -> Option<&str> {
        self.current_bgm_path.as_deref()
    }

    /// 压低 BGM 音量（duck）
    pub fn duck(&mut self) {
        self.duck_target = Self::DUCK_VOLUME_RATIO;
        debug!("BGM duck -> {:.0}%", self.duck_target * 100.0);
    }

    /// 恢复 BGM 音量（unduck）
    pub fn unduck(&mut self) {
        self.duck_target = 1.0;
        debug!("BGM unduck -> 100%");
    }

    fn effective_bgm_volume(&self) -> f32 {
        if self.muted { 0.0 } else { self.bgm_volume }
    }

    fn current_base_bgm_volume(&self) -> f32 {
        let sink_vol = self
            .bgm_sink
            .as_ref()
            .map(|s| s.volume())
            .unwrap_or(self.effective_bgm_volume());
        if self.duck_multiplier > 0.001 {
            sink_vol / self.duck_multiplier
        } else {
            self.effective_bgm_volume()
        }
    }

    // ── 播放 ─────────────────────────────────────────────────────────────────

    /// 播放 BGM
    ///
    /// 状态更新在前、I/O 在后：headless 下 `current_bgm_path` 和 `fade_state` 正确推进。
    pub fn play_bgm(&mut self, path: &str, looping: bool, fade_in: Option<f32>) {
        if let Some(ref sink) = self.bgm_sink {
            sink.stop();
        }
        self.bgm_sink = None;

        let logical_path = normalize_logical_path(path);
        self.current_bgm_path = Some(logical_path.clone());

        if let Some(duration) = fade_in
            && duration > 0.0
        {
            self.fade_state = FadeState::FadeIn {
                target_volume: self.effective_bgm_volume(),
                current_volume: 0.0,
                rate: self.effective_bgm_volume() / duration,
            };
        } else {
            self.fade_state = FadeState::None;
        }

        debug!(path = %logical_path, looping, fade_in = ?fade_in, "Playing BGM");

        let Some(ref device_sink) = self.device_sink else {
            return;
        };

        let bytes = match self.audio_cache.get(&logical_path) {
            Some(b) => b.clone(),
            None => {
                error!(path = %logical_path, "Audio not cached (call cache_audio_bytes first)");
                return;
            }
        };

        let source = match Decoder::new(Cursor::new(bytes)) {
            Ok(s) => s,
            Err(e) => {
                error!(path = %logical_path, error = %e, "Cannot decode audio");
                return;
            }
        };

        let sink = Player::connect_new(device_sink.mixer());
        let initial_volume = if fade_in.is_some() {
            0.0
        } else {
            self.effective_bgm_volume() * self.duck_multiplier
        };
        sink.set_volume(initial_volume);

        if looping {
            sink.append(source.repeat_infinite());
        } else {
            sink.append(source);
        }

        self.bgm_sink = Some(sink);
    }

    /// 停止 BGM
    pub fn stop_bgm(&mut self, fade_out: Option<f32>) {
        if self.bgm_sink.is_none() {
            return;
        }

        if let Some(duration) = fade_out
            && duration > 0.0
        {
            let base_volume = self.current_base_bgm_volume();
            self.fade_state = FadeState::FadeOut {
                current_volume: base_volume,
                rate: base_volume / duration,
                stop_after: true,
                next_bgm: None,
            };
            debug!(duration, "BGM 淡出中");
            return;
        }

        if let Some(ref sink) = self.bgm_sink {
            sink.stop();
        }
        self.bgm_sink = None;
        self.current_bgm_path = None;
        self.fade_state = FadeState::None;
        debug!("BGM 已停止");
    }

    /// 切换 BGM（带交叉淡入淡出）
    pub fn crossfade_bgm(&mut self, path: &str, looping: bool, fade_duration: f32) {
        if self.bgm_sink.is_none() {
            self.play_bgm(path, looping, Some(fade_duration));
            return;
        }

        let base_volume = self.current_base_bgm_volume();
        self.fade_state = FadeState::FadeOut {
            current_volume: base_volume,
            rate: base_volume / fade_duration,
            stop_after: false,
            next_bgm: Some((path.to_string(), looping)),
        };
        debug!(duration = fade_duration, "BGM 切换: 淡出中");
    }

    /// 播放音效
    pub fn play_sfx(&self, path: &str) {
        if self.muted {
            return;
        }

        let Some(ref device_sink) = self.device_sink else {
            return;
        };

        let logical_path = normalize_logical_path(path);

        let bytes = match self.audio_cache.get(&logical_path) {
            Some(b) => b.clone(),
            None => {
                error!(path = %logical_path, "SFX not cached (call cache_audio_bytes first)");
                return;
            }
        };

        let source = match Decoder::new(Cursor::new(bytes)) {
            Ok(s) => s,
            Err(e) => {
                error!(path = %logical_path, error = %e, "Cannot decode SFX");
                return;
            }
        };

        let sink = Player::connect_new(device_sink.mixer());
        sink.set_volume(self.sfx_volume);
        sink.append(source);
        sink.detach();
        debug!(path = %logical_path, "Playing SFX");
    }

    // ── 更新 ─────────────────────────────────────────────────────────────────

    /// 每帧更新（推进淡入淡出和 duck）
    pub fn update(&mut self, dt: f32) {
        let mut next_bgm_to_play: Option<(String, bool, f32)> = None;
        let mut fade_completed = false;
        let mut should_stop = false;
        let dm = self.duck_multiplier;

        match &mut self.fade_state {
            FadeState::None => {}
            FadeState::FadeIn {
                target_volume,
                current_volume,
                rate,
            } => {
                *current_volume += *rate * dt;
                if *current_volume >= *target_volume {
                    if let Some(ref sink) = self.bgm_sink {
                        sink.set_volume(*target_volume * dm);
                    }
                    fade_completed = true;
                    debug!("BGM 淡入完成");
                } else if let Some(ref sink) = self.bgm_sink {
                    sink.set_volume(*current_volume * dm);
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
                    if let Some((path, looping)) = next_bgm.take() {
                        let duration = if *rate > 0.0 { 1.0 / *rate } else { 0.5 };
                        next_bgm_to_play = Some((path, looping, duration));
                    }
                    should_stop = *stop_after;
                    fade_completed = true;
                } else if let Some(ref sink) = self.bgm_sink {
                    sink.set_volume(*current_volume * dm);
                }
            }
        }

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

        if let Some((ref path, looping, duration)) = next_bgm_to_play {
            if let Some(ref sink) = self.bgm_sink {
                sink.stop();
            }
            self.bgm_sink = None;
            self.current_bgm_path = None;
            self.play_bgm(path, looping, Some(duration));
        }

        let diff = self.duck_target - self.duck_multiplier;
        if diff.abs() > 0.001 {
            let step = Self::DUCK_FADE_SPEED * dt;
            if diff > 0.0 {
                self.duck_multiplier = (self.duck_multiplier + step).min(self.duck_target);
            } else {
                self.duck_multiplier = (self.duck_multiplier - step).max(self.duck_target);
            }
            if matches!(self.fade_state, FadeState::None)
                && let Some(ref sink) = self.bgm_sink
            {
                sink.set_volume(self.effective_bgm_volume() * self.duck_multiplier);
            }
        }
    }
}
