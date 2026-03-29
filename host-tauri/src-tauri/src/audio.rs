//! 音频状态追踪器（headless）
//!
//! `AudioManager` 只追踪音频逻辑状态（当前 BGM、音量、duck），
//! 不做任何 I/O。实际播放由前端 Web Audio API 负责。

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
    /// 待下发给前端的 BGM 过渡信号，`drain_audio_state()` 时 take 消费
    pending_transition: Option<f32>,
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

    /// 停止 BGM（仅更新状态）
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

    /// 播放音效（加入队列，下帧由 `drain_audio_state()` 输出后清空）
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

    // ── play_bgm 同曲判定 ──────────────────────────────────────────────────

    #[test]
    fn play_bgm_same_path_no_transition() {
        let mut am = new_manager();
        am.play_bgm("bgm/track1.ogg", true, None);
        am.drain_audio_state(); // consume first transition

        am.play_bgm("bgm/track1.ogg", true, None);
        let state = am.drain_audio_state();
        assert!(
            state.bgm_transition.is_none(),
            "same BGM should not produce a transition"
        );
    }

    // ── play_bgm crossfade vs fade_in ──────────────────────────────────────

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

    // ── stop_bgm ───────────────────────────────────────────────────────────

    #[test]
    fn stop_bgm_with_fade_out() {
        let mut am = new_manager();
        am.play_bgm("bgm/track1.ogg", true, None);
        am.drain_audio_state();

        am.stop_bgm(Some(2.0));
        let state = am.drain_audio_state();
        assert_eq!(
            state.bgm_transition.expect("should have transition").duration,
            2.0
        );
        assert!(state.bgm.is_none(), "BGM should be cleared after stop");
    }

    #[test]
    fn stop_bgm_no_bgm_does_not_panic() {
        let mut am = new_manager();
        am.stop_bgm(Some(1.0));
        let state = am.drain_audio_state();
        assert!(state.bgm.is_none());
        assert!(state.bgm_transition.is_none());
    }

    // ── play_sfx + drain ───────────────────────────────────────────────────

    #[test]
    fn play_sfx_enqueue_and_drain() {
        let mut am = new_manager();
        am.play_sfx("sfx/click.ogg");

        let state = am.drain_audio_state();
        assert_eq!(state.sfx_queue.len(), 1);
        assert_eq!(state.sfx_queue[0].path, "sfx/click.ogg");
        assert_eq!(state.sfx_queue[0].volume, 1.0);

        let state2 = am.drain_audio_state();
        assert!(state2.sfx_queue.is_empty(), "SFX queue should be drained");
    }

    // ── drain 消费语义 ─────────────────────────────────────────────────────

    #[test]
    fn drain_consumes_pending_transition() {
        let mut am = new_manager();
        am.play_bgm("bgm/track1.ogg", true, None);

        let first = am.drain_audio_state();
        assert!(first.bgm_transition.is_some());

        let second = am.drain_audio_state();
        assert!(
            second.bgm_transition.is_none(),
            "transition should be consumed after first drain"
        );
    }

    // ── duck / unduck 平滑收敛 ─────────────────────────────────────────────

    #[test]
    fn duck_converges_toward_target() {
        let mut am = new_manager();
        am.duck();

        for _ in 0..100 {
            am.update(1.0 / 60.0);
        }

        let state = am.drain_audio_state();
        // duck_multiplier 不直接暴露，但通过 bgm volume 间接验证
        // 需要有 BGM 才能观测
        drop(state);

        am.play_bgm("bgm/track1.ogg", true, None);
        let state = am.drain_audio_state();
        let bgm = state.bgm.expect("BGM should be present");
        let expected = AudioManager::DUCK_VOLUME_RATIO;
        assert!(
            (bgm.volume - expected).abs() < 0.01,
            "duck_multiplier should converge to {expected}, got volume {}",
            bgm.volume
        );
    }

    #[test]
    fn unduck_restores_volume() {
        let mut am = new_manager();
        am.play_bgm("bgm/track1.ogg", true, None);
        am.duck();
        for _ in 0..100 {
            am.update(1.0 / 60.0);
        }

        am.unduck();
        for _ in 0..100 {
            am.update(1.0 / 60.0);
        }

        am.drain_audio_state(); // consume transition
        let state = am.drain_audio_state();
        let bgm = state.bgm.expect("BGM should be present");
        assert!(
            (bgm.volume - 1.0).abs() < 0.01,
            "volume should restore to 1.0 after unduck, got {}",
            bgm.volume
        );
    }

    // ── set_bgm_volume / set_sfx_volume ────────────────────────────────────

    #[test]
    fn set_bgm_volume_clamps_and_applies() {
        let mut am = new_manager();
        am.play_bgm("bgm/track1.ogg", true, None);

        am.set_bgm_volume(0.5);
        let state = am.drain_audio_state();
        let bgm = state.bgm.expect("BGM should be present");
        assert_eq!(bgm.volume, 0.5);

        am.set_bgm_volume(2.0);
        let state = am.drain_audio_state();
        let bgm = state.bgm.expect("BGM should be present");
        assert_eq!(bgm.volume, 1.0, "volume should be clamped to 1.0");

        am.set_bgm_volume(-1.0);
        let state = am.drain_audio_state();
        let bgm = state.bgm.expect("BGM should be present");
        assert_eq!(bgm.volume, 0.0, "volume should be clamped to 0.0");
    }

    #[test]
    fn set_sfx_volume_clamps_and_applies() {
        let mut am = new_manager();

        am.set_sfx_volume(0.7);
        am.play_sfx("sfx/click.ogg");
        let state = am.drain_audio_state();
        assert_eq!(state.sfx_queue[0].volume, 0.7);

        am.set_sfx_volume(5.0);
        am.play_sfx("sfx/click.ogg");
        let state = am.drain_audio_state();
        assert_eq!(state.sfx_queue[0].volume, 1.0, "should clamp to 1.0");
    }
}
