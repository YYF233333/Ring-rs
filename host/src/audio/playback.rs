//! BGM/SFX 播放逻辑与淡入淡出状态机
//!
//! 包含 `AudioManager` 的 `play_bgm`、`stop_bgm`、`crossfade_bgm`、`play_sfx` 和 `update` 方法。

use rodio::{Decoder, Player, Source};
use std::io::Cursor;
use tracing::{debug, error};

use super::{AudioManager, FadeState};

impl AudioManager {
    /// 播放 BGM
    ///
    /// # 参数
    ///
    /// - `path`: BGM 逻辑路径（相对于 assets_root，如 `bgm/music.mp3`）
    /// - `looping`: 是否循环
    /// - `fade_in`: 淡入时长（秒），None 表示立即播放
    pub fn play_bgm(&mut self, path: &str, looping: bool, fade_in: Option<f32>) {
        use crate::resources::normalize_logical_path;

        if let Some(ref sink) = self.bgm_sink {
            sink.stop();
        }

        let logical_path = normalize_logical_path(path);

        let bytes = match self.audio_cache.get(&logical_path) {
            Some(b) => b.clone(),
            None => {
                error!(
                    path = %logical_path,
                    "Audio not cached (call cache_audio_bytes first)"
                );
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

        let sink = Player::connect_new(self.device_sink.mixer());

        let initial_volume = if fade_in.is_some() {
            0.0
        } else {
            self.get_effective_bgm_volume() * self.duck_multiplier
        };
        sink.set_volume(initial_volume);

        if looping {
            sink.append(source.repeat_infinite());
        } else {
            sink.append(source);
        }

        self.bgm_sink = Some(sink);
        self.current_bgm_path = Some(logical_path.clone());

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
            "Playing BGM"
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
            let base_volume = self.current_base_bgm_volume();

            self.fade_state = FadeState::FadeOut {
                current_volume: base_volume,
                rate: base_volume / duration,
                stop_after: true,
                next_bgm: None,
            };
            debug!(duration = duration, "BGM 淡出中");
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
        use crate::resources::normalize_logical_path;

        if self.muted {
            return;
        }

        let logical_path = normalize_logical_path(path);

        let bytes = match self.audio_cache.get(&logical_path) {
            Some(b) => b.clone(),
            None => {
                error!(
                    path = %logical_path,
                    "SFX not cached (call cache_audio_bytes first)"
                );
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

        let sink = Player::connect_new(self.device_sink.mixer());
        sink.set_volume(self.sfx_volume);
        sink.append(source);
        sink.detach();
        debug!(path = %logical_path, "Playing SFX");
    }

    /// 更新音频状态（每帧调用）
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
                } else {
                    if let Some(ref sink) = self.bgm_sink {
                        sink.set_volume(*current_volume * dm);
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
                    if let Some((path, looping)) = next_bgm.take() {
                        let duration = if *rate > 0.0 { 1.0 / *rate } else { 0.5 };
                        next_bgm_to_play = Some((path, looping, duration));
                    }
                    should_stop = *stop_after;
                    fade_completed = true;
                } else {
                    if let Some(ref sink) = self.bgm_sink {
                        sink.set_volume(*current_volume * dm);
                    }
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

        // Duck multiplier 平滑过渡
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
                sink.set_volume(self.get_effective_bgm_volume() * self.duck_multiplier);
            }
        }
    }
}
