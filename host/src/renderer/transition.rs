//! # Transition 模块
//!
//! 过渡效果系统，负责管理背景切换的双纹理混合过渡动画。
//! 内部使用 AnimationSystem 管理动画状态。
//!
//! ## 支持的过渡效果
//!
//! - `dissolve` / `Dissolve(duration)`: 淡入淡出（交叉溶解）
//! - `none`: 无过渡，立即切换
//!
//! 注意：`fade` 和 `fadewhite` 效果由 `changeScene` 命令使用 `SceneMaskState` 处理，
//! 不在本模块中实现。

use super::animation::{AnimationId, AnimationSystem, EasingFunction, PropertyKey};

/// 过渡效果类型
#[derive(Debug, Clone, PartialEq)]
pub enum TransitionType {
    /// 无过渡
    None,
    /// 淡入淡出（交叉溶解）
    Dissolve,
}

/// 过渡效果状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransitionPhase {
    /// 空闲状态
    Idle,
    /// 淡入阶段（新内容出现，旧内容淡出）
    FadeIn,
}

/// 过渡效果管理器
///
/// 内部使用 AnimationSystem 管理背景过渡动画。
/// 同时管理两个动画：
/// - 旧背景：淡出动画（alpha: 1.0 → 0.0）
/// - 新背景：淡入动画（alpha: 0.0 → 1.0）
#[derive(Debug)]
pub struct TransitionManager {
    /// 内部动画系统
    animation_system: AnimationSystem,
    /// 旧背景淡出动画 ID
    old_bg_animation_id: Option<AnimationId>,
    /// 新背景淡入动画 ID
    new_bg_animation_id: Option<AnimationId>,
}

impl TransitionManager {
    /// 创建新的过渡效果管理器
    pub fn new() -> Self {
        Self {
            animation_system: AnimationSystem::new(),
            old_bg_animation_id: None,
            new_bg_animation_id: None,
        }
    }

    /// 开始过渡效果
    ///
    /// # 参数
    ///
    /// - `transition_type`: 过渡类型
    /// - `duration`: 过渡时长（秒）
    pub fn start(&mut self, transition_type: TransitionType, duration: f32) {
        // 清除之前的动画
        self.animation_system.skip_all();
        self.animation_system.update(0.0); // 清理已完成的动画
        self.old_bg_animation_id = None;
        self.new_bg_animation_id = None;

        if transition_type == TransitionType::None {
            return;
        }

        let duration = duration.max(0.01); // 避免除零

        // 启动旧背景淡出动画 (1.0 → 0.0)
        let old_key = PropertyKey::old_background_alpha();
        let old_id = self.animation_system.animate_with_easing(
            old_key,
            1.0,
            0.0,
            duration,
            EasingFunction::EaseInOutQuad,
        );
        self.old_bg_animation_id = Some(old_id);

        // 启动新背景淡入动画 (0.0 → 1.0)
        let new_key = PropertyKey::background_alpha();
        let new_id = self.animation_system.animate_with_easing(
            new_key,
            0.0,
            1.0,
            duration,
            EasingFunction::EaseInOutQuad,
        );
        self.new_bg_animation_id = Some(new_id);
    }

    /// 从 vn-runtime 的 Transition 解析
    ///
    /// 注意：只支持 `dissolve` 效果。`fade` 和 `fadewhite` 效果应使用 `changeScene` 命令。
    pub fn start_from_command(&mut self, transition: &vn_runtime::command::Transition) {
        let name = transition.name.to_lowercase();
        let duration = transition.get_duration().map(|d| d as f32).unwrap_or(0.3);

        let transition_type = match name.as_str() {
            "dissolve" => TransitionType::Dissolve,
            "none" => TransitionType::None,
            "fade" | "fadewhite" | "fade_white" => {
                println!(
                    "⚠️ {} 效果应由 changeScene 命令使用 SceneMaskState 处理，使用 dissolve 代替",
                    name
                );
                TransitionType::Dissolve
            }
            _ => {
                println!("⚠️ 未知过渡效果: {}, 使用 dissolve", name);
                TransitionType::Dissolve
            }
        };

        self.start(transition_type, duration);
    }

    /// 更新过渡效果
    ///
    /// # 返回
    ///
    /// - `true`: 过渡效果仍在进行中
    /// - `false`: 过渡效果已完成或处于空闲状态
    pub fn update(&mut self, dt: f32) -> bool {
        self.animation_system.update(dt);
        self.is_active()
    }

    /// 跳过过渡效果
    pub fn skip(&mut self) {
        self.animation_system.skip_all();
        // 立即更新以应用最终状态
        self.animation_system.update(0.0);
    }

    /// 获取当前阶段
    pub fn phase(&self) -> TransitionPhase {
        if self.is_active() {
            TransitionPhase::FadeIn
        } else {
            TransitionPhase::Idle
        }
    }

    /// 是否正在过渡中
    pub fn is_active(&self) -> bool {
        self.animation_system.has_active_animations()
    }

    /// 获取当前进度（0.0 - 1.0）
    pub fn progress(&self) -> f32 {
        // 使用新背景动画的进度作为整体进度
        if let Some(id) = self.new_bg_animation_id {
            self.animation_system.get_progress(id).unwrap_or(1.0)
        } else {
            1.0
        }
    }

    /// 获取用于渲染新内容的 alpha 值
    ///
    /// Dissolve: 新内容从 0 淡入到 1
    pub fn new_content_alpha(&self) -> f32 {
        self.animation_system.get_background_alpha()
    }

    /// 获取用于渲染旧内容的 alpha 值
    ///
    /// Dissolve: 旧内容从 1 淡出到 0
    pub fn old_content_alpha(&self) -> f32 {
        self.animation_system.get_old_background_alpha()
    }
}

impl Default for TransitionManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transition_manager_creation() {
        let manager = TransitionManager::new();
        assert_eq!(manager.phase(), TransitionPhase::Idle);
        assert!(!manager.is_active());
    }

    #[test]
    fn test_dissolve_transition() {
        let mut manager = TransitionManager::new();
        manager.start(TransitionType::Dissolve, 1.0);

        assert!(manager.is_active());
        assert_eq!(manager.phase(), TransitionPhase::FadeIn);
        // 初始时新内容 alpha 应该接近 0
        assert!(manager.new_content_alpha() < 0.1);

        // 模拟半程
        manager.update(0.5);
        assert!(manager.new_content_alpha() > 0.0);
        assert!(manager.new_content_alpha() < 1.0);

        // 完成
        manager.update(0.6);
        assert!(!manager.is_active());
        assert_eq!(manager.new_content_alpha(), 1.0);
    }

    #[test]
    fn test_skip_transition() {
        let mut manager = TransitionManager::new();
        manager.start(TransitionType::Dissolve, 1.0);

        assert!(manager.is_active());
        manager.skip();
        assert!(!manager.is_active());
        // 跳过后应该是最终状态
        assert_eq!(manager.new_content_alpha(), 1.0);
        assert_eq!(manager.old_content_alpha(), 0.0);
    }

    #[test]
    fn test_old_and_new_alpha_inverse() {
        let mut manager = TransitionManager::new();
        manager.start(TransitionType::Dissolve, 1.0);

        // 更新到中间位置
        manager.update(0.5);

        let new_alpha = manager.new_content_alpha();
        let old_alpha = manager.old_content_alpha();

        // 新旧 alpha 应该互补（接近 1.0）
        assert!((new_alpha + old_alpha - 1.0).abs() < 0.1);
    }
}
