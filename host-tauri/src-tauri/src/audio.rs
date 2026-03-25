//! 音频状态追踪器（headless）
//!
//! `AudioManager` 只追踪音频逻辑状态（当前 BGM、音量、duck），
//! 不做任何 I/O。实际播放由前端 Web Audio API 负责。

use tracing::debug;

use crate::render_state::{AudioRenderState, BgmState, SfxRequest};
use crate::resources::normalize_logical_path;

/// 音频管理器（headless 状态追踪）
pub struct AudioManager {
    current_bgm_path: Option<String>,
    bgm_looping: bool,
    bgm_volume: f32,
    sfx_volume: f32,
    muted: bool,
    duck_multiplier: f32,
    duck_target: f32,
    sfx_queue: Vec<SfxRequest>,
}

impl AudioManager {
    const DUCK_VOLUME_RATIO: f32 = 0.3;
    const DUCK_FADE_SPEED: f32 = 3.0;

    pub fn new() -> Self {
        Self {
            current_bgm_path: None,
            bgm_looping: true,
            bgm_volume: 1.0,
            sfx_volume: 1.0,
            muted: false,
            duck_multiplier: 1.0,
            duck_target: 1.0,
            sfx_queue: Vec::new(),
        }
    }

    /// 设置 BGM 音量
    pub fn set_bgm_volume(&mut self, volume: f32) {
        self.bgm_volume = volume.clamp(0.0, 1.0);
    }

    /// 设置 SFX 音量
    pub fn set_sfx_volume(&mut self, volume: f32) {
        self.sfx_volume = volume.clamp(0.0, 1.0);
    }

    /// 获取当前 BGM 路径
    pub fn current_bgm_path(&self) -> Option<&str> {
        self.current_bgm_path.as_deref()
    }

    // ── BGM 状态操作 ─────────────────────────────────────────────────────────

    /// 播放 BGM（仅更新状态）
    pub fn play_bgm(&mut self, path: &str, looping: bool, _fade_in: Option<f32>) {
        let logical_path = normalize_logical_path(path);
        self.current_bgm_path = Some(logical_path.clone());
        self.bgm_looping = looping;
        debug!(path = %logical_path, looping, "BGM state: play");
    }

    /// 停止 BGM（仅更新状态）
    pub fn stop_bgm(&mut self, _fade_out: Option<f32>) {
        if self.current_bgm_path.is_none() {
            return;
        }
        self.current_bgm_path = None;
        debug!("BGM state: stop");
    }

    /// 播放音效（加入队列，下帧由 `to_audio_state()` 输出后清空）
    pub fn play_sfx(&mut self, path: &str) {
        let logical_path = normalize_logical_path(path);
        let volume = if self.muted { 0.0 } else { self.sfx_volume };
        self.sfx_queue.push(SfxRequest {
            path: logical_path.clone(),
            volume,
        });
        debug!(path = %logical_path, "SFX state: queued");
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

    // ── 每帧更新 ─────────────────────────────────────────────────────────────

    /// 每帧更新 duck 平滑过渡
    pub fn update(&mut self, dt: f32) {
        let diff = self.duck_target - self.duck_multiplier;
        if diff.abs() > 0.001 {
            let step = Self::DUCK_FADE_SPEED * dt;
            if diff > 0.0 {
                self.duck_multiplier = (self.duck_multiplier + step).min(self.duck_target);
            } else {
                self.duck_multiplier = (self.duck_multiplier - step).max(self.duck_target);
            }
        }
    }

    /// 生成当前帧的音频声明式状态，并清空 SFX 队列
    pub fn to_audio_state(&mut self) -> AudioRenderState {
        let bgm = self.current_bgm_path.as_ref().map(|path| {
            let volume = if self.muted {
                0.0
            } else {
                self.bgm_volume * self.duck_multiplier
            };
            BgmState {
                path: path.clone(),
                looping: self.bgm_looping,
                volume,
            }
        });

        let sfx_queue = std::mem::take(&mut self.sfx_queue);

        AudioRenderState { bgm, sfx_queue }
    }
}
