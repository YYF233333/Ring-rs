//! Renderer 场景效果与过渡管理
//!
//! 包含震动、模糊过渡、背景 dissolve 过渡和 changeScene 遮罩过渡的状态驱动方法。

use super::{Renderer, SceneEffectState, TransitionType};

impl Renderer {
    /// 启动震动效果
    pub fn start_shake(&mut self, amplitude_x: f32, amplitude_y: f32, duration: f32) {
        self.shake = super::ShakeState {
            active: true,
            amplitude_x,
            amplitude_y,
            elapsed: 0.0,
            duration,
        };
    }

    /// 启动模糊过渡
    pub fn start_blur_transition(&mut self, from: f32, to: f32, duration: f32) {
        self.blur_transition = super::BlurTransitionState {
            active: true,
            from,
            to,
            elapsed: 0.0,
            duration,
        };
    }

    /// 更新场景效果（每帧调用）
    pub fn update_scene_effects(&mut self, dt: f32, scene_effect: &mut SceneEffectState) -> bool {
        let mut any_active = false;

        if self.shake.active {
            self.shake.elapsed += dt;
            if self.shake.elapsed >= self.shake.duration {
                self.shake.active = false;
                scene_effect.shake_offset_x = 0.0;
                scene_effect.shake_offset_y = 0.0;
            } else {
                let progress = self.shake.elapsed / self.shake.duration;
                let decay = 1.0 - progress;
                let t = self.shake.elapsed * super::SHAKE_FREQUENCY;
                scene_effect.shake_offset_x = t.sin() * self.shake.amplitude_x * decay;
                scene_effect.shake_offset_y = (t * 1.3).cos() * self.shake.amplitude_y * decay;
                any_active = true;
            }
        }

        if self.blur_transition.active {
            self.blur_transition.elapsed += dt;
            if self.blur_transition.elapsed >= self.blur_transition.duration {
                self.blur_transition.active = false;
                scene_effect.blur_amount = self.blur_transition.to;
            } else {
                let progress =
                    (self.blur_transition.elapsed / self.blur_transition.duration).clamp(0.0, 1.0);
                let smoothed = progress * progress * (3.0 - 2.0 * progress);
                scene_effect.blur_amount = self.blur_transition.from
                    + (self.blur_transition.to - self.blur_transition.from) * smoothed;
                any_active = true;
            }
        }

        any_active
    }

    /// 检查场景效果是否仍在播放
    pub fn is_scene_effect_active(&self) -> bool {
        self.shake.active || self.blur_transition.active
    }

    pub(super) fn current_shake_offset(&self) -> (f32, f32) {
        if self.shake.active {
            let progress = self.shake.elapsed / self.shake.duration;
            let decay = 1.0 - progress;
            let t = self.shake.elapsed * super::SHAKE_FREQUENCY;
            (
                t.sin() * self.shake.amplitude_x * decay,
                (t * 1.3).cos() * self.shake.amplitude_y * decay,
            )
        } else {
            (0.0, 0.0)
        }
    }

    // ========== 背景 dissolve 过渡 ==========

    /// 更新过渡效果
    pub fn update_transition(&mut self, dt: f32) -> bool {
        self.transition.update(dt)
    }

    /// 开始背景过渡（保留兼容）
    pub fn start_background_transition(
        &mut self,
        old_bg: Option<String>,
        transition: Option<&vn_runtime::command::Transition>,
    ) {
        self.old_background = old_bg;

        if let Some(trans) = transition {
            self.transition.start_from_command(trans);
        } else {
            self.transition.start(TransitionType::Dissolve, 0.2);
        }
    }

    /// 开始背景过渡（基于 ResolvedEffect 的统一入口）
    pub fn start_background_transition_resolved(
        &mut self,
        old_bg: Option<String>,
        effect: &super::effects::ResolvedEffect,
    ) {
        self.old_background = old_bg;
        self.transition.start_from_resolved(effect);
    }

    /// 跳过当前过渡效果
    pub fn skip_transition(&mut self) {
        self.transition.skip();
        self.old_background = None;
    }

    // ========== changeScene 场景过渡 ==========

    pub fn start_scene_fade(&mut self, duration: f32, pending_background: String) {
        self.scene_transition
            .start_fade(duration, pending_background);
    }

    pub fn start_scene_fade_white(&mut self, duration: f32, pending_background: String) {
        self.scene_transition
            .start_fade_white(duration, pending_background);
    }

    pub fn start_scene_rule(
        &mut self,
        duration: f32,
        pending_background: String,
        mask_path: String,
        reversed: bool,
    ) {
        self.scene_transition
            .start_rule(duration, pending_background, mask_path, reversed);
    }

    pub fn update_scene_transition(&mut self, dt: f32) -> bool {
        self.scene_transition.update(dt)
    }

    pub fn is_scene_transition_at_midpoint(&self) -> bool {
        self.scene_transition.is_at_midpoint()
    }

    pub fn take_pending_background(&mut self) -> Option<String> {
        self.scene_transition.take_pending_background()
    }

    pub fn get_scene_transition_ui_alpha(&self) -> f32 {
        if self.scene_transition.is_active() {
            self.scene_transition.ui_alpha()
        } else {
            1.0
        }
    }

    pub fn skip_scene_transition_phase(&mut self) {
        self.scene_transition.skip_current_phase();
    }

    pub fn skip_scene_transition_to_end(&mut self) -> Option<String> {
        self.scene_transition.skip_to_end()
    }

    pub fn is_scene_transition_active(&self) -> bool {
        self.scene_transition.is_active()
    }

    pub fn is_scene_transition_ui_fading_in(&self) -> bool {
        self.scene_transition.is_ui_fading_in()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_renderer() -> Renderer {
        Renderer::new(1920.0, 1080.0)
    }

    // ===== shake =====

    #[test]
    fn start_shake_sets_active() {
        let mut r = make_renderer();
        r.start_shake(10.0, 5.0, 0.5);
        assert!(r.is_scene_effect_active());
    }

    #[test]
    fn shake_completes_after_duration() {
        let mut r = make_renderer();
        let mut fx = SceneEffectState::default();
        r.start_shake(10.0, 5.0, 0.5);

        let active = r.update_scene_effects(0.6, &mut fx);
        assert!(!active);
        assert!(!r.is_scene_effect_active());
        assert_eq!(fx.shake_offset_x, 0.0);
        assert_eq!(fx.shake_offset_y, 0.0);
    }

    #[test]
    fn shake_mid_progress_produces_offset() {
        let mut r = make_renderer();
        let mut fx = SceneEffectState::default();
        r.start_shake(10.0, 5.0, 1.0);

        let active = r.update_scene_effects(0.3, &mut fx);
        assert!(active);
        assert!(fx.shake_offset_x != 0.0 || fx.shake_offset_y != 0.0);
    }

    #[test]
    fn current_shake_offset_inactive() {
        let r = make_renderer();
        assert_eq!(r.current_shake_offset(), (0.0, 0.0));
    }

    #[test]
    fn current_shake_offset_active() {
        let mut r = make_renderer();
        let mut fx = SceneEffectState::default();
        r.start_shake(10.0, 5.0, 1.0);
        r.update_scene_effects(0.2, &mut fx);

        let (ox, oy) = r.current_shake_offset();
        assert!(ox != 0.0 || oy != 0.0);
    }

    // ===== blur transition =====

    #[test]
    fn blur_transition_sets_active() {
        let mut r = make_renderer();
        r.start_blur_transition(0.0, 1.0, 0.5);
        assert!(r.is_scene_effect_active());
    }

    #[test]
    fn blur_transition_completes_to_target() {
        let mut r = make_renderer();
        let mut fx = SceneEffectState::default();
        r.start_blur_transition(0.0, 5.0, 0.5);

        let active = r.update_scene_effects(0.6, &mut fx);
        assert!(!active);
        assert_eq!(fx.blur_amount, 5.0);
    }

    #[test]
    fn blur_transition_mid_progress() {
        let mut r = make_renderer();
        let mut fx = SceneEffectState::default();
        r.start_blur_transition(0.0, 10.0, 1.0);

        let active = r.update_scene_effects(0.5, &mut fx);
        assert!(active);
        assert!(fx.blur_amount > 0.0);
        assert!(fx.blur_amount < 10.0);
    }

    // ===== combined =====

    #[test]
    fn both_effects_active() {
        let mut r = make_renderer();
        let mut fx = SceneEffectState::default();
        r.start_shake(10.0, 5.0, 1.0);
        r.start_blur_transition(0.0, 1.0, 1.0);

        let active = r.update_scene_effects(0.3, &mut fx);
        assert!(active);
        assert!(r.is_scene_effect_active());
    }

    #[test]
    fn no_effects_returns_false() {
        let mut r = make_renderer();
        let mut fx = SceneEffectState::default();
        let active = r.update_scene_effects(0.1, &mut fx);
        assert!(!active);
    }

    // ===== scene transition delegation =====

    #[test]
    fn scene_transition_ui_alpha_default() {
        let r = make_renderer();
        assert_eq!(r.get_scene_transition_ui_alpha(), 1.0);
    }

    #[test]
    fn scene_transition_not_active_by_default() {
        let r = make_renderer();
        assert!(!r.is_scene_transition_active());
        assert!(!r.is_scene_transition_at_midpoint());
        assert!(!r.is_scene_transition_ui_fading_in());
    }

    #[test]
    fn scene_fade_starts_transition() {
        let mut r = make_renderer();
        r.start_scene_fade(1.0, "bg/new.png".to_string());
        assert!(r.is_scene_transition_active());
    }

    #[test]
    fn scene_fade_white_starts_transition() {
        let mut r = make_renderer();
        r.start_scene_fade_white(1.0, "bg/new.png".to_string());
        assert!(r.is_scene_transition_active());
    }

    #[test]
    fn scene_rule_starts_transition() {
        let mut r = make_renderer();
        r.start_scene_rule(1.0, "bg/new.png".to_string(), "mask.png".to_string(), false);
        assert!(r.is_scene_transition_active());
    }

    #[test]
    fn skip_scene_transition_to_end_returns_bg() {
        let mut r = make_renderer();
        r.start_scene_fade(1.0, "bg/new.png".to_string());
        let bg = r.skip_scene_transition_to_end();
        assert_eq!(bg.as_deref(), Some("bg/new.png"));
        assert!(!r.is_scene_transition_active());
    }

    #[test]
    fn background_transition_start_and_skip() {
        let mut r = make_renderer();
        r.start_background_transition(Some("old.png".to_string()), None);
        assert!(r.update_transition(0.01));

        r.skip_transition();
        assert!(!r.update_transition(0.01));
    }
}
