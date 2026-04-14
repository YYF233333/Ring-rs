use crate::render_state::{PlaybackMode, SceneTransitionPhaseState};

use super::*;

impl AppStateInner {
    /// 每帧调用，推进打字机和计时器
    pub fn process_tick(&mut self, dt: f32) {
        if !self.host_screen.allows_progression() {
            self.project_render_state();
            return;
        }

        self.advance_playback_mode(dt);
        self.update_animations(dt);
        self.resolve_waits(dt);

        if self.waiting == WaitingFor::Nothing && !self.script_finished {
            self.run_script_tick();
        }

        self.advance_typewriter(dt);
        self.sync_audio(dt);
        self.project_render_state();
    }

    /// Skip 模式立即推进 + Auto 模式计时推进
    ///
    /// Skip 采用两帧策略（与旧 host 一致）：
    /// - 第一帧：完成打字机，让完整文本显示一帧
    /// - 第二帧：打字机已完成，推进到下一句
    pub(super) fn advance_playback_mode(&mut self, dt: f32) {
        if self.playback_mode == PlaybackMode::Skip {
            let typewriter_was_incomplete = !self.render_state.is_dialogue_complete();

            // 始终先完成打字机和所有效果
            if typewriter_was_incomplete {
                self.render_state.complete_typewriter();
            }

            // 如果打字机刚完成，让完整文本显示至少一帧再推进
            if typewriter_was_incomplete {
                return;
            }

            match self.waiting.clone() {
                WaitingFor::Click => {
                    self.clear_click_wait();
                }
                WaitingFor::Time { .. } => {
                    self.clear_wait();
                }
                WaitingFor::Signal(signal_kind) => {
                    self.complete_signal_wait(signal_kind);
                }
                WaitingFor::Cutscene => {
                    self.finish_cutscene();
                }
                _ => {}
            }
        }

        if self.playback_mode == PlaybackMode::Auto
            && self.waiting == WaitingFor::Click
            && self.render_state.is_dialogue_complete()
        {
            self.auto_timer += dt;
            if self.auto_timer >= self.user_settings.auto_delay {
                self.auto_timer = 0.0;
                self.clear_click_wait();
            }
        }
    }

    pub(super) fn complete_signal_wait(&mut self, signal_kind: SignalKind) {
        match signal_kind {
            SignalKind::SceneTransition => {
                if let Some(st) = self.render_state.scene_transition.as_mut() {
                    if let Some(bg) = st.pending_background.take() {
                        self.render_state.current_background = Some(bg);
                    }
                    st.phase = SceneTransitionPhaseState::Completed;
                }
            }
            SignalKind::TitleCard => {
                self.render_state.title_card = None;
            }
            SignalKind::SceneEffect => {
                self.anim.scene_effect_active = false;
            }
            SignalKind::Cutscene => {
                self.finish_cutscene();
            }
        }

        self.clear_wait();
    }

    /// 推进 chapter_mark / title_card / background_transition / scene_transition / 角色 alpha
    pub(super) fn update_animations(&mut self, dt: f32) {
        self.render_state.update_chapter_mark(dt);

        if let Some(tc) = self.render_state.title_card.as_mut() {
            tc.elapsed += dt;
            if tc.elapsed >= tc.duration {
                self.render_state.title_card = None;
            }
        }

        self.update_background_transition(dt);
        self.update_scene_transition(dt);
        self.update_character_alpha(dt);
        self.update_shake(dt);
    }

    /// 推进角色 alpha 过渡，淡出完成后移除
    pub(super) fn update_character_alpha(&mut self, dt: f32) {
        for c in self.render_state.visible_characters.values_mut() {
            let duration = c.transition_duration.unwrap_or(0.0);
            if duration > 0.0 && (c.alpha - c.target_alpha).abs() > f32::EPSILON {
                let speed = dt / duration;
                if c.alpha < c.target_alpha {
                    c.alpha = (c.alpha + speed).min(c.target_alpha);
                } else {
                    c.alpha = (c.alpha - speed).max(c.target_alpha);
                }
                if (c.alpha - c.target_alpha).abs() <= f32::EPSILON {
                    c.alpha = c.target_alpha;
                    c.transition_duration = None;
                }
            } else if duration <= 0.0 {
                c.alpha = c.target_alpha;
            }
        }
        self.render_state
            .visible_characters
            .retain(|_, c| !(c.fading_out && c.alpha <= f32::EPSILON));
    }

    /// 解析 Signal 等待 + Time 等待
    pub(super) fn resolve_waits(&mut self, dt: f32) {
        if let WaitingFor::Signal(signal_kind) = self.waiting {
            let resolved = match signal_kind {
                SignalKind::SceneTransition => self
                    .render_state
                    .scene_transition
                    .as_ref()
                    .is_none_or(|st| st.phase == SceneTransitionPhaseState::Completed),
                SignalKind::TitleCard => self.render_state.title_card.is_none(),
                SignalKind::SceneEffect => !self.anim.scene_effect_active,
                SignalKind::Cutscene => self.render_state.cutscene.is_none(),
            };
            if resolved {
                self.clear_wait();
            }
        }

        if let WaitingFor::Time { remaining_ms } = &self.waiting {
            let elapsed_ms = (dt * 1000.0) as u64;
            if elapsed_ms >= *remaining_ms {
                self.clear_wait();
            } else {
                let decrement = elapsed_ms;
                if let WaitingFor::Time { remaining_ms } = &mut self.waiting {
                    *remaining_ms -= decrement;
                }
            }
        }
    }

    /// 推进打字机 + inline wait
    pub(super) fn advance_typewriter(&mut self, dt: f32) {
        if !self.render_state.is_dialogue_complete() && !self.render_state.has_inline_wait() {
            let speed = self.render_state.effective_text_speed(self.text_speed);
            self.typewriter_timer += dt * speed;
            while self.typewriter_timer >= 1.0 {
                self.typewriter_timer -= 1.0;
                let done = self.render_state.advance_typewriter();
                if done {
                    self.typewriter_timer = 0.0;
                    if self
                        .render_state
                        .dialogue
                        .as_ref()
                        .is_some_and(|d| d.no_wait)
                        && self.waiting == WaitingFor::Click
                    {
                        self.clear_click_wait();
                    }
                    break;
                }
                if self.render_state.has_inline_wait() {
                    break;
                }
            }
        }

        if self.render_state.has_inline_wait() && !self.render_state.is_inline_click_wait() {
            let finished = self.render_state.update_inline_wait(dt as f64);
            if finished {
                // 定时等待结束，继续打字
            }
        }
    }

    /// 同步音频状态到 render_state
    pub(super) fn sync_audio(&mut self, dt: f32) {
        if let Some(svc) = self.services.as_mut() {
            svc.audio.update(dt);
            self.render_state.audio = svc.audio.drain_audio_state();
        }
    }

    /// 推进背景 dissolve 过渡（内部计时器，不推到 RenderState）
    pub(super) fn update_background_transition(&mut self, dt: f32) {
        if let Some(bt) = self.render_state.background_transition.as_mut() {
            self.anim.bg_transition_elapsed += dt;
            if self.anim.bg_transition_elapsed >= bt.duration {
                self.render_state.background_transition = None;
                self.anim.bg_transition_elapsed = 0.0;
            }
        }
    }

    /// 推进场景遮罩过渡（内部计时器推进 phase，不计算渐变值）
    pub(super) fn update_scene_transition(&mut self, dt: f32) {
        const HOLD_DURATION: f32 = 0.2;

        let Some(st) = self.render_state.scene_transition.as_mut() else {
            return;
        };

        self.anim.scene_transition_elapsed += dt;

        match st.phase {
            SceneTransitionPhaseState::FadeIn => {
                if self.anim.scene_transition_elapsed >= st.duration {
                    if let Some(bg) = st.pending_background.take() {
                        self.render_state.current_background = Some(bg);
                    }
                    st.phase = SceneTransitionPhaseState::Hold;
                    self.anim.scene_transition_elapsed = 0.0;
                }
            }
            SceneTransitionPhaseState::Hold => {
                if self.anim.scene_transition_elapsed >= HOLD_DURATION {
                    st.phase = SceneTransitionPhaseState::FadeOut;
                    self.anim.scene_transition_elapsed = 0.0;
                }
            }
            SceneTransitionPhaseState::FadeOut => {
                if self.anim.scene_transition_elapsed >= st.duration {
                    st.phase = SceneTransitionPhaseState::Completed;
                    self.anim.scene_transition_elapsed = 0.0;
                }
            }
            SceneTransitionPhaseState::Completed => {
                self.render_state.scene_transition = None;
                self.anim.scene_transition_elapsed = 0.0;
            }
        }
    }

    /// 推进 shake 动画
    pub(super) fn update_shake(&mut self, dt: f32) {
        let Some(shake) = self.anim.active_shake.as_mut() else {
            return;
        };
        shake.elapsed += dt;
        if shake.elapsed >= shake.duration {
            self.render_state.scene_effect.shake_offset_x = 0.0;
            self.render_state.scene_effect.shake_offset_y = 0.0;
            self.anim.active_shake = None;
            self.anim.scene_effect_active = false;
        } else {
            let progress = shake.elapsed / shake.duration;
            let decay = 1.0 - progress;
            let freq = 30.0;
            let phase = shake.elapsed * freq;
            self.render_state.scene_effect.shake_offset_x = shake.amplitude_x * decay * phase.sin();
            self.render_state.scene_effect.shake_offset_y = shake.amplitude_y * decay * phase.cos();
        }
    }
}
