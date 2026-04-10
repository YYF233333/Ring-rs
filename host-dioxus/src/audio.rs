//! 音频状态追踪器（headless）
//!
//! `AudioManager` 只追踪音频逻辑状态（当前 BGM、音量、duck），
//! 不做任何 I/O。实际播放由前端负责。

use tracing::debug;

use crate::render_state::{AudioRenderState, BgmState, BgmTransition, SfxRequest};
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
    pending_transition: Option<f32>,
}

impl Default for AudioManager {
    fn default() -> Self {
        Self::new()
    }
}

impl AudioManager {
    const DUCK_VOLUME_RATIO: f32 = 0.3;
    const DUCK_FADE_SPEED: f32 = 3.0;
    const CROSSFADE_DURATION: f32 = 1.0;
    const FADE_IN_DURATION: f32 = 0.5;

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
            pending_transition: None,
        }
    }

    pub fn set_bgm_volume(&mut self, volume: f32) {
        self.bgm_volume = volume.clamp(0.0, 1.0);
    }

    pub fn set_sfx_volume(&mut self, volume: f32) {
        self.sfx_volume = volume.clamp(0.0, 1.0);
    }

    pub fn current_bgm_path(&self) -> Option<&str> {
        self.current_bgm_path.as_deref()
    }

    pub fn play_bgm(&mut self, path: &str, looping: bool, _fade_in: Option<f32>) {
        let logical_path = normalize_logical_path(path);
        let is_same = self
            .current_bgm_path
            .as_ref()
            .is_some_and(|p| *p == logical_path);
        if !is_same {
            let duration = if self.current_bgm_path.is_some() {
                Self::CROSSFADE_DURATION
            } else {
                Self::FADE_IN_DURATION
            };
            self.pending_transition = Some(duration);
        }
        self.current_bgm_path = Some(logical_path.clone());
        self.bgm_looping = looping;
        debug!(path = %logical_path, looping, "BGM state: play");
    }

    pub fn stop_bgm(&mut self, fade_out: Option<f32>) {
        if self.current_bgm_path.is_none() {
            return;
        }
        if let Some(duration) = fade_out {
            self.pending_transition = Some(duration);
        }
        self.current_bgm_path = None;
        debug!("BGM state: stop");
    }

    pub fn play_sfx(&mut self, path: &str) {
        let logical_path = normalize_logical_path(path);
        let volume = if self.muted { 0.0 } else { self.sfx_volume };
        self.sfx_queue.push(SfxRequest {
            path: logical_path.clone(),
            volume,
        });
        debug!(path = %logical_path, "SFX state: queued");
    }

    pub fn duck(&mut self) {
        self.duck_target = Self::DUCK_VOLUME_RATIO;
        debug!("BGM duck -> {:.0}%", self.duck_target * 100.0);
    }

    pub fn unduck(&mut self) {
        self.duck_target = 1.0;
        debug!("BGM unduck -> 100%");
    }

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

    pub fn drain_audio_state(&mut self) -> AudioRenderState {
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
        let bgm_transition = self
            .pending_transition
            .take()
            .map(|duration| BgmTransition { duration });
        AudioRenderState {
            bgm,
            sfx_queue,
            bgm_transition,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn new_manager() -> AudioManager {
        AudioManager::new()
    }

    #[test]
    fn play_bgm_same_path_no_transition() {
        let mut am = new_manager();
        am.play_bgm("bgm/track1.ogg", true, None);
        am.drain_audio_state();
        am.play_bgm("bgm/track1.ogg", true, None);
        let state = am.drain_audio_state();
        assert!(state.bgm_transition.is_none());
    }

    #[test]
    fn play_bgm_first_time_uses_fade_in() {
        let mut am = new_manager();
        am.play_bgm("bgm/track1.ogg", true, None);
        let state = am.drain_audio_state();
        let t = state.bgm_transition.expect("should have transition");
        assert_eq!(t.duration, AudioManager::FADE_IN_DURATION);
    }

    #[test]
    fn play_bgm_switch_uses_crossfade() {
        let mut am = new_manager();
        am.play_bgm("bgm/track1.ogg", true, None);
        am.drain_audio_state();
        am.play_bgm("bgm/track2.ogg", true, None);
        let state = am.drain_audio_state();
        let t = state.bgm_transition.expect("should have transition");
        assert_eq!(t.duration, AudioManager::CROSSFADE_DURATION);
    }

    #[test]
    fn stop_bgm_with_fade_out() {
        let mut am = new_manager();
        am.play_bgm("bgm/track1.ogg", true, None);
        am.drain_audio_state();
        am.stop_bgm(Some(2.0));
        let state = am.drain_audio_state();
        assert_eq!(state.bgm_transition.unwrap().duration, 2.0);
        assert!(state.bgm.is_none());
    }

    #[test]
    fn play_sfx_enqueue_and_drain() {
        let mut am = new_manager();
        am.play_sfx("sfx/click.ogg");
        let state = am.drain_audio_state();
        assert_eq!(state.sfx_queue.len(), 1);
        assert_eq!(state.sfx_queue[0].path, "sfx/click.ogg");
        let state2 = am.drain_audio_state();
        assert!(state2.sfx_queue.is_empty());
    }

    #[test]
    fn drain_consumes_pending_transition() {
        let mut am = new_manager();
        am.play_bgm("bgm/track1.ogg", true, None);
        let first = am.drain_audio_state();
        assert!(first.bgm_transition.is_some());
        let second = am.drain_audio_state();
        assert!(second.bgm_transition.is_none());
    }
}
