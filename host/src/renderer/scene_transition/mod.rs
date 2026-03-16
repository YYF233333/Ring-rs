//! # SceneTransition 模块
//!
//! 基于 Trait-based 动画系统的场景切换效果。
//!
//! ## 设计理念
//!
//! 将场景切换（changeScene）的动画逻辑统一到 AnimationSystem：
//! - `AnimatableSceneTransition` 实现 `Animatable` trait，暴露 shader 需要的属性
//! - `SceneTransitionManager` 管理多阶段动画序列
//! - 动画系统负责时间轴管理，直接驱动 shader uniform
//!
//! ## 支持的过渡效果
//!
//! - **Fade（黑屏）**: mask_alpha 0→1, 切换背景, mask_alpha 1→0
//! - **FadeWhite（白屏）**: 同上，使用白色遮罩
//! - **Rule（图片遮罩）**: 使用 ImageDissolve shader，progress 控制溶解进度

mod animatable;
mod types;

pub use animatable::AnimatableSceneTransition;
pub use types::{SceneTransitionPhase, SceneTransitionType};

use std::rc::Rc;

use super::animation::{AnimationSystem, EasingFunction, ObjectId};
use tracing::{info, warn};

/// UI 淡入时长（秒）
const UI_FADE_DURATION: f32 = 0.2;

/// Rule 效果黑屏停顿时长（秒）
const RULE_BLACKOUT_DURATION: f32 = 0.2;

/// 场景过渡管理器
///
/// 使用 Trait-based AnimationSystem 管理场景切换动画。
/// 支持多阶段动画序列，自动处理阶段转换。
pub struct SceneTransitionManager {
    animation_system: AnimationSystem,
    transition_state: Rc<AnimatableSceneTransition>,
    /// 对象 ID（注册到动画系统）
    object_id: ObjectId,
    transition_type: Option<SceneTransitionType>,
    phase: SceneTransitionPhase,
    /// 过渡时长（每个主要阶段）
    duration: f32,
    /// 待切换的新背景路径
    pending_background: Option<String>,
    /// 阶段计时器（用于 Blackout 阶段）
    phase_timer: f32,
}

#[allow(clippy::new_without_default)]
impl SceneTransitionManager {
    pub fn new() -> Self {
        let mut animation_system = AnimationSystem::new();
        let transition_state = Rc::new(AnimatableSceneTransition::new());
        let object_id = animation_system.register(transition_state.clone());

        Self {
            animation_system,
            transition_state,
            object_id,
            transition_type: None,
            phase: SceneTransitionPhase::Idle,
            duration: 0.5,
            pending_background: None,
            phase_timer: 0.0,
        }
    }

    /// 开始 Fade（黑屏）过渡
    ///
    /// # 参数
    /// - `duration`: 每个淡入/淡出阶段的时长（秒）
    /// - `pending_background`: 待切换的新背景路径
    pub fn start_fade(&mut self, duration: f32, pending_background: String) {
        self.start_internal(SceneTransitionType::Fade, duration, pending_background);
    }

    /// 开始 FadeWhite（白屏）过渡
    pub fn start_fade_white(&mut self, duration: f32, pending_background: String) {
        self.start_internal(SceneTransitionType::FadeWhite, duration, pending_background);
    }

    /// 开始 Rule（图片遮罩）过渡
    pub fn start_rule(
        &mut self,
        duration: f32,
        pending_background: String,
        mask_path: String,
        reversed: bool,
    ) {
        self.start_internal(
            SceneTransitionType::Rule {
                mask_path,
                reversed,
            },
            duration,
            pending_background,
        );
    }

    fn start_internal(
        &mut self,
        transition_type: SceneTransitionType,
        duration: f32,
        pending_background: String,
    ) {
        self.animation_system.skip_all();
        self.animation_system.update(0.0);

        self.transition_type = Some(transition_type.clone());
        self.duration = duration.max(0.01);
        self.pending_background = Some(pending_background);
        self.phase_timer = 0.0;

        self.transition_state.reset();

        self.phase = SceneTransitionPhase::FadeIn;
        self.start_fade_in_animations();

        info!(transition_type = ?transition_type, duration = %duration, "SceneTransition: 开始过渡");
    }

    fn start_fade_in_animations(&mut self) {
        match &self.transition_type {
            Some(SceneTransitionType::Fade) | Some(SceneTransitionType::FadeWhite) => {
                // Fade/FadeWhite: mask_alpha 0 → 1
                if let Err(e) = self
                    .animation_system
                    .animate_object_with_easing::<AnimatableSceneTransition>(
                        self.object_id,
                        "mask_alpha",
                        0.0,
                        1.0,
                        self.duration,
                        EasingFunction::EaseInOutQuad,
                    )
                {
                    warn!(error = %e, "场景过渡 FadeIn 动画启动失败");
                }
            }
            Some(SceneTransitionType::Rule { .. }) => {
                // Rule: progress 0 → 1（旧背景溶解到黑屏）
                if let Err(e) = self
                    .animation_system
                    .animate_object_with_easing::<AnimatableSceneTransition>(
                        self.object_id,
                        "progress",
                        0.0,
                        1.0,
                        self.duration,
                        EasingFunction::EaseInOutQuad,
                    )
                {
                    warn!(error = %e, "场景过渡 Rule FadeIn 动画启动失败");
                }
            }
            None => {}
        }
    }

    fn start_fade_out_animations(&mut self) {
        match &self.transition_type {
            Some(SceneTransitionType::Fade) | Some(SceneTransitionType::FadeWhite) => {
                // Fade/FadeWhite: mask_alpha 1 → 0
                if let Err(e) = self
                    .animation_system
                    .animate_object_with_easing::<AnimatableSceneTransition>(
                        self.object_id,
                        "mask_alpha",
                        1.0,
                        0.0,
                        self.duration,
                        EasingFunction::EaseInOutQuad,
                    )
                {
                    warn!(error = %e, "场景过渡 FadeOut 动画启动失败");
                }
            }
            Some(SceneTransitionType::Rule { .. }) => {
                // Rule: progress 0 → 1（黑屏溶解到新背景）
                // 注意：这里重新从 0 开始，因为是新的一轮溶解
                self.transition_state.set_progress(0.0);
                if let Err(e) = self
                    .animation_system
                    .animate_object_with_easing::<AnimatableSceneTransition>(
                        self.object_id,
                        "progress",
                        0.0,
                        1.0,
                        self.duration,
                        EasingFunction::EaseInOutQuad,
                    )
                {
                    warn!(error = %e, "场景过渡 Rule FadeOut 动画启动失败");
                }
            }
            None => {}
        }
    }

    fn start_ui_fade_in_animations(&mut self) {
        if let Err(e) = self
            .animation_system
            .animate_object_with_easing::<AnimatableSceneTransition>(
                self.object_id,
                "ui_alpha",
                0.0,
                1.0,
                UI_FADE_DURATION,
                EasingFunction::EaseOutQuad,
            )
        {
            warn!(error = %e, "场景过渡 UI 淡入动画启动失败");
        }
    }

    /// 更新过渡效果
    ///
    /// # 返回
    /// - `true`: 过渡仍在进行中
    /// - `false`: 过渡已完成或处于空闲状态
    pub fn update(&mut self, dt: f32) -> bool {
        if self.phase == SceneTransitionPhase::Idle || self.phase == SceneTransitionPhase::Completed
        {
            return false;
        }

        self.animation_system.update(dt);

        match self.phase {
            SceneTransitionPhase::FadeIn => {
                if !self.animation_system.has_active_animations() {
                    match &self.transition_type {
                        Some(SceneTransitionType::Rule { .. }) => {
                            self.phase = SceneTransitionPhase::Blackout;
                            self.phase_timer = 0.0;
                            self.transition_state.set_progress(1.0);
                        }
                        _ => {
                            self.phase = SceneTransitionPhase::FadeOut;
                            self.start_fade_out_animations();
                        }
                    }
                }
            }
            SceneTransitionPhase::Blackout => {
                self.phase_timer += dt;
                if self.phase_timer >= RULE_BLACKOUT_DURATION {
                    self.phase = SceneTransitionPhase::FadeOut;
                    self.start_fade_out_animations();
                }
            }
            SceneTransitionPhase::FadeOut => {
                if !self.animation_system.has_active_animations() {
                    self.phase = SceneTransitionPhase::UIFadeIn;
                    self.start_ui_fade_in_animations();
                }
            }
            SceneTransitionPhase::UIFadeIn => {
                if !self.animation_system.has_active_animations() {
                    self.phase = SceneTransitionPhase::Completed;
                    self.transition_state.set_completed();
                    info!("SceneTransition: 过渡完成");
                }
            }
            _ => {}
        }

        self.phase != SceneTransitionPhase::Completed
    }

    /// 跳过当前阶段
    ///
    /// 行为与原 SceneMaskState::skip_current_phase() 一致
    pub fn skip_current_phase(&mut self) {
        self.animation_system.skip_all();
        self.animation_system.update(0.0);

        match self.phase {
            SceneTransitionPhase::FadeIn => match &self.transition_type {
                Some(SceneTransitionType::Rule { .. }) => {
                    self.phase = SceneTransitionPhase::FadeOut;
                    self.transition_state.set_progress(0.0);
                    self.start_fade_out_animations();
                }
                _ => {
                    self.phase = SceneTransitionPhase::FadeOut;
                    self.transition_state.set_mask_alpha(1.0);
                    self.start_fade_out_animations();
                }
            },
            SceneTransitionPhase::Blackout
            | SceneTransitionPhase::FadeOut
            | SceneTransitionPhase::UIFadeIn => {
                self.phase = SceneTransitionPhase::Completed;
                self.transition_state.set_completed();
            }
            _ => {}
        }
    }

    /// 完全跳过过渡
    pub fn skip_all(&mut self) {
        self.animation_system.skip_all();
        self.animation_system.update(0.0);
        self.phase = SceneTransitionPhase::Completed;
        self.transition_state.set_completed();
    }

    /// 完全跳过过渡并返回待切换的背景
    ///
    /// 与 `skip_all()` 不同，本方法**保证** `pending_background` 被正确返回，
    /// 调用方可据此完成背景切换，避免 Skip 模式下背景丢失。
    pub fn skip_to_end(&mut self) -> Option<String> {
        let bg = self.pending_background.take();
        self.skip_all();
        bg
    }

    pub fn phase(&self) -> SceneTransitionPhase {
        self.phase
    }

    /// 是否正在过渡中
    pub fn is_active(&self) -> bool {
        self.phase != SceneTransitionPhase::Idle && self.phase != SceneTransitionPhase::Completed
    }

    /// 判断是否处于中间点（可以进行场景切换）
    ///
    /// 对于 Fade/FadeWhite：FadeOut 阶段刚开始时
    /// 对于 Rule：FadeOut 阶段刚开始时（黑屏停顿结束后）
    pub fn is_at_midpoint(&self) -> bool {
        if self.phase != SceneTransitionPhase::FadeOut {
            return false;
        }

        // 只要 pending_background 仍存在，就意味着"背景尚未切换"，
        // 中间点只应触发一次：切换后会通过 take_pending_background() 清空。
        if self.pending_background.is_none() {
            return false;
        }

        // 关键：这里的"中间点"必须发生在进入 FadeOut 之后、FadeOut 动画推进之前。
        // 由于 update() 先推进动画，再进行阶段转换并启动 FadeOut 动画，
        // 因此在进入 FadeOut 的那一帧，属性值仍保持在 FadeOut 的起始值：
        // - Fade/FadeWhite：mask_alpha == 1.0（全遮罩）
        // - Rule：progress == 0.0（从黑屏溶解到新背景的起点）
        match &self.transition_type {
            Some(SceneTransitionType::Fade) | Some(SceneTransitionType::FadeWhite) => {
                self.mask_alpha() >= 0.999
            }
            Some(SceneTransitionType::Rule { .. }) => self.progress() <= 0.001,
            None => false,
        }
    }

    /// 判断是否正在进行 UI 淡入
    pub fn is_ui_fading_in(&self) -> bool {
        self.phase == SceneTransitionPhase::UIFadeIn
    }

    /// 判断遮罩是否已完成（不再需要渲染遮罩效果）
    pub fn is_mask_complete(&self) -> bool {
        matches!(
            self.phase,
            SceneTransitionPhase::UIFadeIn | SceneTransitionPhase::Completed
        )
    }

    /// 获取并清除待切换的背景
    pub fn take_pending_background(&mut self) -> Option<String> {
        self.pending_background.take()
    }

    /// 查看待切换的背景（不移除）
    pub fn pending_background(&self) -> Option<&str> {
        self.pending_background.as_deref()
    }

    pub fn transition_type(&self) -> Option<&SceneTransitionType> {
        self.transition_type.as_ref()
    }

    /// 获取当前溶解进度（用于 shader）
    pub fn progress(&self) -> f32 {
        self.transition_state.progress()
    }

    pub fn mask_alpha(&self) -> f32 {
        self.transition_state.mask_alpha()
    }

    pub fn ui_alpha(&self) -> f32 {
        self.transition_state.ui_alpha()
    }
}

impl std::fmt::Debug for SceneTransitionManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SceneTransitionManager")
            .field("phase", &self.phase)
            .field("progress", &self.progress())
            .field("mask_alpha", &self.mask_alpha())
            .field("ui_alpha", &self.ui_alpha())
            .field("is_active", &self.is_active())
            .finish()
    }
}

#[cfg(test)]
mod tests;
