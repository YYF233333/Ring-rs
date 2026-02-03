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

use std::cell::RefCell;
use std::rc::Rc;

use super::animation::{Animatable, AnimationSystem, EasingFunction, ObjectId};

/// 场景过渡类型
#[derive(Debug, Clone)]
pub enum SceneTransitionType {
    /// 黑屏淡入淡出
    Fade,
    /// 白屏淡入淡出
    FadeWhite,
    /// 图片遮罩（Rule-based dissolve）
    Rule {
        /// 遮罩图片路径
        mask_path: String,
        /// 是否反向
        reversed: bool,
    },
}

/// 场景过渡阶段
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SceneTransitionPhase {
    /// 空闲状态
    Idle,
    /// 阶段 1：遮罩淡入 / 旧背景溶解到黑屏
    FadeIn,
    /// 阶段 2：黑屏停顿（仅 Rule 效果）
    Blackout,
    /// 阶段 3：遮罩淡出 / 黑屏溶解到新背景
    FadeOut,
    /// 阶段 4：UI 淡入
    UIFadeIn,
    /// 完成
    Completed,
}

/// 可动画的场景过渡状态
///
/// 实现 `Animatable` trait，暴露以下属性供动画系统驱动：
/// - `progress`: 溶解进度 (0.0 - 1.0)，用于 ImageDissolve shader
/// - `mask_alpha`: 遮罩透明度 (0.0 - 1.0)，用于 Fade/FadeWhite
/// - `ui_alpha`: UI 透明度 (0.0 - 1.0)，用于 UI 淡入
#[derive(Debug)]
pub struct AnimatableSceneTransition {
    inner: RefCell<SceneTransitionData>,
}

/// 场景过渡内部数据
#[derive(Debug, Clone)]
struct SceneTransitionData {
    /// 溶解进度（用于 Rule 效果的 shader）
    progress: f32,
    /// 遮罩透明度（用于 Fade/FadeWhite）
    mask_alpha: f32,
    /// UI 透明度
    ui_alpha: f32,
}

impl AnimatableSceneTransition {
    /// 创建新的场景过渡状态
    pub fn new() -> Self {
        Self {
            inner: RefCell::new(SceneTransitionData {
                progress: 0.0,
                mask_alpha: 0.0,
                ui_alpha: 1.0,
            }),
        }
    }

    /// 重置为过渡开始状态
    pub fn reset(&self) {
        let mut data = self.inner.borrow_mut();
        data.progress = 0.0;
        data.mask_alpha = 0.0;
        data.ui_alpha = 0.0;
    }

    /// 设置为完成状态
    pub fn set_completed(&self) {
        let mut data = self.inner.borrow_mut();
        data.progress = 1.0;
        data.mask_alpha = 0.0;
        data.ui_alpha = 1.0;
    }

    /// 获取当前溶解进度
    pub fn progress(&self) -> f32 {
        self.inner.borrow().progress
    }

    /// 获取当前遮罩透明度
    pub fn mask_alpha(&self) -> f32 {
        self.inner.borrow().mask_alpha
    }

    /// 获取当前 UI 透明度
    pub fn ui_alpha(&self) -> f32 {
        self.inner.borrow().ui_alpha
    }

    /// 直接设置进度（用于跳过动画）
    pub fn set_progress(&self, value: f32) {
        self.inner.borrow_mut().progress = value;
    }

    /// 直接设置遮罩透明度
    pub fn set_mask_alpha(&self, value: f32) {
        self.inner.borrow_mut().mask_alpha = value;
    }

    /// 直接设置 UI 透明度
    pub fn set_ui_alpha(&self, value: f32) {
        self.inner.borrow_mut().ui_alpha = value;
    }
}

impl Default for AnimatableSceneTransition {
    fn default() -> Self {
        Self::new()
    }
}

impl Animatable for AnimatableSceneTransition {
    fn get_property(&self, property_id: &str) -> Option<f32> {
        let data = self.inner.borrow();
        match property_id {
            "progress" => Some(data.progress),
            "mask_alpha" => Some(data.mask_alpha),
            "ui_alpha" => Some(data.ui_alpha),
            _ => None,
        }
    }

    fn set_property(&self, property_id: &str, value: f32) -> bool {
        let mut data = self.inner.borrow_mut();
        match property_id {
            "progress" => {
                data.progress = value;
                true
            }
            "mask_alpha" => {
                data.mask_alpha = value;
                true
            }
            "ui_alpha" => {
                data.ui_alpha = value;
                true
            }
            _ => false,
        }
    }

    fn property_list(&self) -> &'static [&'static str] {
        &["progress", "mask_alpha", "ui_alpha"]
    }
}

/// UI 淡入时长（秒）
const UI_FADE_DURATION: f32 = 0.2;

/// Rule 效果黑屏停顿时长（秒）
const RULE_BLACKOUT_DURATION: f32 = 0.2;

/// 场景过渡管理器
///
/// 使用 Trait-based AnimationSystem 管理场景切换动画。
/// 支持多阶段动画序列，自动处理阶段转换。
pub struct SceneTransitionManager {
    /// 内部动画系统
    animation_system: AnimationSystem,
    /// 场景过渡状态对象
    transition_state: Rc<AnimatableSceneTransition>,
    /// 对象 ID（注册到动画系统）
    object_id: ObjectId,
    /// 过渡类型
    transition_type: Option<SceneTransitionType>,
    /// 当前阶段
    phase: SceneTransitionPhase,
    /// 过渡时长（每个主要阶段）
    duration: f32,
    /// 待切换的新背景路径
    pending_background: Option<String>,
    /// 阶段计时器（用于 Blackout 阶段）
    phase_timer: f32,
}

impl SceneTransitionManager {
    /// 创建新的场景过渡管理器
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
            SceneTransitionType::Rule { mask_path, reversed },
            duration,
            pending_background,
        );
    }

    /// 内部启动方法
    fn start_internal(
        &mut self,
        transition_type: SceneTransitionType,
        duration: f32,
        pending_background: String,
    ) {
        // 跳过并清理之前的动画
        self.animation_system.skip_all();
        self.animation_system.update(0.0);

        // 保存参数
        self.transition_type = Some(transition_type);
        self.duration = duration.max(0.01);
        self.pending_background = Some(pending_background);
        self.phase_timer = 0.0;

        // 重置状态
        self.transition_state.reset();

        // 进入第一阶段
        self.phase = SceneTransitionPhase::FadeIn;
        self.start_fade_in_animations();

        println!(
            "🎬 SceneTransition: 开始 {:?} 过渡 ({}s)",
            self.transition_type, self.duration
        );
    }

    /// 启动 FadeIn 阶段的动画
    fn start_fade_in_animations(&mut self) {
        match &self.transition_type {
            Some(SceneTransitionType::Fade) | Some(SceneTransitionType::FadeWhite) => {
                // Fade/FadeWhite: mask_alpha 0 → 1
                let _ = self
                    .animation_system
                    .animate_object_with_easing::<AnimatableSceneTransition>(
                        self.object_id,
                        "mask_alpha",
                        0.0,
                        1.0,
                        self.duration,
                        EasingFunction::EaseInOutQuad,
                    );
            }
            Some(SceneTransitionType::Rule { .. }) => {
                // Rule: progress 0 → 1（旧背景溶解到黑屏）
                let _ = self
                    .animation_system
                    .animate_object_with_easing::<AnimatableSceneTransition>(
                        self.object_id,
                        "progress",
                        0.0,
                        1.0,
                        self.duration,
                        EasingFunction::EaseInOutQuad,
                    );
            }
            None => {}
        }
    }

    /// 启动 FadeOut 阶段的动画
    fn start_fade_out_animations(&mut self) {
        match &self.transition_type {
            Some(SceneTransitionType::Fade) | Some(SceneTransitionType::FadeWhite) => {
                // Fade/FadeWhite: mask_alpha 1 → 0
                let _ = self
                    .animation_system
                    .animate_object_with_easing::<AnimatableSceneTransition>(
                        self.object_id,
                        "mask_alpha",
                        1.0,
                        0.0,
                        self.duration,
                        EasingFunction::EaseInOutQuad,
                    );
            }
            Some(SceneTransitionType::Rule { .. }) => {
                // Rule: progress 0 → 1（黑屏溶解到新背景）
                // 注意：这里重新从 0 开始，因为是新的一轮溶解
                self.transition_state.set_progress(0.0);
                let _ = self
                    .animation_system
                    .animate_object_with_easing::<AnimatableSceneTransition>(
                        self.object_id,
                        "progress",
                        0.0,
                        1.0,
                        self.duration,
                        EasingFunction::EaseInOutQuad,
                    );
            }
            None => {}
        }
    }

    /// 启动 UI 淡入动画
    fn start_ui_fade_in_animations(&mut self) {
        let _ = self
            .animation_system
            .animate_object_with_easing::<AnimatableSceneTransition>(
                self.object_id,
                "ui_alpha",
                0.0,
                1.0,
                UI_FADE_DURATION,
                EasingFunction::EaseOutQuad,
            );
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

        // 更新动画系统
        self.animation_system.update(dt);

        // 检查阶段转换
        match self.phase {
            SceneTransitionPhase::FadeIn => {
                if !self.animation_system.has_active_animations() {
                    // FadeIn 完成，进入下一阶段
                    match &self.transition_type {
                        Some(SceneTransitionType::Rule { .. }) => {
                            // Rule: 进入黑屏停顿阶段
                            self.phase = SceneTransitionPhase::Blackout;
                            self.phase_timer = 0.0;
                            self.transition_state.set_progress(1.0); // 保持全黑
                        }
                        _ => {
                            // Fade/FadeWhite: 直接进入 FadeOut
                            self.phase = SceneTransitionPhase::FadeOut;
                            self.start_fade_out_animations();
                        }
                    }
                }
            }
            SceneTransitionPhase::Blackout => {
                // Rule 专用：黑屏停顿
                self.phase_timer += dt;
                if self.phase_timer >= RULE_BLACKOUT_DURATION {
                    self.phase = SceneTransitionPhase::FadeOut;
                    self.start_fade_out_animations();
                }
            }
            SceneTransitionPhase::FadeOut => {
                if !self.animation_system.has_active_animations() {
                    // FadeOut 完成，进入 UI 淡入
                    self.phase = SceneTransitionPhase::UIFadeIn;
                    self.start_ui_fade_in_animations();
                }
            }
            SceneTransitionPhase::UIFadeIn => {
                if !self.animation_system.has_active_animations() {
                    // UI 淡入完成，过渡结束
                    self.phase = SceneTransitionPhase::Completed;
                    self.transition_state.set_completed();
                    println!("🎬 SceneTransition: 过渡完成");
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
            SceneTransitionPhase::FadeIn => {
                // 跳到中间点（遮罩完全显现）
                match &self.transition_type {
                    Some(SceneTransitionType::Rule { .. }) => {
                        // Rule: 跳到 FadeOut 开始
                        self.phase = SceneTransitionPhase::FadeOut;
                        self.transition_state.set_progress(0.0);
                        self.start_fade_out_animations();
                    }
                    _ => {
                        // Fade/FadeWhite: 跳到 FadeOut 开始
                        self.phase = SceneTransitionPhase::FadeOut;
                        self.transition_state.set_mask_alpha(1.0);
                        self.start_fade_out_animations();
                    }
                }
            }
            SceneTransitionPhase::Blackout
            | SceneTransitionPhase::FadeOut
            | SceneTransitionPhase::UIFadeIn => {
                // 直接完成
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

    /// 获取当前阶段
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
        self.phase == SceneTransitionPhase::FadeOut
            && !self.animation_system.has_active_animations()
            && self.phase_timer < 0.01
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

    /// 获取过渡类型
    pub fn transition_type(&self) -> Option<&SceneTransitionType> {
        self.transition_type.as_ref()
    }

    /// 获取当前溶解进度（用于 shader）
    pub fn progress(&self) -> f32 {
        self.transition_state.progress()
    }

    /// 获取当前遮罩透明度
    pub fn mask_alpha(&self) -> f32 {
        self.transition_state.mask_alpha()
    }

    /// 获取当前 UI 透明度
    pub fn ui_alpha(&self) -> f32 {
        self.transition_state.ui_alpha()
    }
}

impl Default for SceneTransitionManager {
    fn default() -> Self {
        Self::new()
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
mod tests {
    use super::*;

    #[test]
    fn test_animatable_scene_transition() {
        let state = AnimatableSceneTransition::new();

        assert_eq!(state.progress(), 0.0);
        assert_eq!(state.mask_alpha(), 0.0);
        assert_eq!(state.ui_alpha(), 1.0);

        state.set_progress(0.5);
        assert_eq!(state.progress(), 0.5);

        state.reset();
        assert_eq!(state.progress(), 0.0);
        assert_eq!(state.ui_alpha(), 0.0);
    }

    #[test]
    fn test_scene_transition_manager_creation() {
        let manager = SceneTransitionManager::new();
        assert_eq!(manager.phase(), SceneTransitionPhase::Idle);
        assert!(!manager.is_active());
    }

    #[test]
    fn test_fade_transition() {
        let mut manager = SceneTransitionManager::new();
        manager.start_fade(0.5, "new_bg.png".to_string());

        assert!(manager.is_active());
        assert_eq!(manager.phase(), SceneTransitionPhase::FadeIn);

        // 模拟完成 FadeIn
        for _ in 0..10 {
            manager.update(0.1);
        }

        // 应该进入 FadeOut 或更后的阶段
        assert!(matches!(
            manager.phase(),
            SceneTransitionPhase::FadeOut
                | SceneTransitionPhase::UIFadeIn
                | SceneTransitionPhase::Completed
        ));
    }

    #[test]
    fn test_rule_transition() {
        let mut manager = SceneTransitionManager::new();
        manager.start_rule(
            0.3,
            "new_bg.png".to_string(),
            "mask.png".to_string(),
            false,
        );

        assert!(manager.is_active());
        assert_eq!(manager.phase(), SceneTransitionPhase::FadeIn);
        assert!(manager.transition_type().is_some());
    }

    #[test]
    fn test_skip_all() {
        let mut manager = SceneTransitionManager::new();
        manager.start_fade(1.0, "new_bg.png".to_string());

        assert!(manager.is_active());
        manager.skip_all();

        assert!(!manager.is_active());
        assert_eq!(manager.phase(), SceneTransitionPhase::Completed);
        assert_eq!(manager.ui_alpha(), 1.0);
    }
}
