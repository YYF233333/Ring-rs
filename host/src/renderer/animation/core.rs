//! # Animation 模块
//!
//! 通用动画实例定义。
//!
//! 核心设计：动画只关注 f32 值的时间轴变化，不假设对象类型。

use super::EasingFunction;

/// 动画 ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AnimationId(pub u64);

impl AnimationId {
    /// 创建新的动画 ID
    pub fn new(id: u64) -> Self {
        Self(id)
    }
}

/// 动画状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AnimationState {
    /// 等待开始（有延迟）
    #[default]
    Pending,
    /// 正在播放
    Playing,
    /// 已暂停
    Paused,
    /// 已完成
    Completed,
    /// 已跳过
    Skipped,
}

impl AnimationState {
    /// 是否为活跃状态（需要更新）
    pub fn is_active(&self) -> bool {
        matches!(self, Self::Pending | Self::Playing)
    }

    /// 是否已结束
    pub fn is_finished(&self) -> bool {
        matches!(self, Self::Completed | Self::Skipped)
    }
}

/// 通用动画实例
///
/// 管理单个 f32 值从 `from` 到 `to` 在 `duration` 时间内的变化。
#[derive(Debug, Clone)]
pub struct Animation {
    /// 动画 ID
    pub id: AnimationId,
    /// 起始值
    pub from: f32,
    /// 目标值
    pub to: f32,
    /// 动画时长（秒）
    pub duration: f32,
    /// 缓动函数
    pub easing: EasingFunction,
    /// 延迟启动（秒）
    pub delay: f32,
    /// 当前状态
    pub state: AnimationState,
    /// 当前进度（0.0 - 1.0，已应用缓动）
    pub progress: f32,
    /// 已经过的时间
    elapsed: f32,
    /// 是否可跳过
    pub skippable: bool,
}

impl Animation {
    /// 创建新的动画（内部使用）
    pub fn new_internal(id: AnimationId, from: f32, to: f32, duration: f32) -> Self {
        let state = if duration <= 0.0 {
            AnimationState::Completed
        } else {
            AnimationState::Pending
        };

        Self {
            id,
            from,
            to,
            duration: duration.max(0.0),
            easing: EasingFunction::default(),
            delay: 0.0,
            state,
            progress: 0.0,
            elapsed: 0.0,
            skippable: true,
        }
    }

    /// 设置缓动函数
    pub fn with_easing(mut self, easing: EasingFunction) -> Self {
        self.easing = easing;
        self
    }

    /// 设置延迟
    pub fn with_delay(mut self, delay: f32) -> Self {
        self.delay = delay.max(0.0);
        self
    }

    /// 设置是否可跳过
    pub fn with_skippable(mut self, skippable: bool) -> Self {
        self.skippable = skippable;
        self
    }

    /// 更新动画
    ///
    /// # 返回
    /// - `true`: 动画仍在进行中
    /// - `false`: 动画已结束
    pub fn update(&mut self, dt: f32) -> bool {
        match self.state {
            AnimationState::Pending => {
                self.elapsed += dt;
                if self.elapsed >= self.delay {
                    self.state = AnimationState::Playing;
                    self.elapsed -= self.delay;
                    self.update_playing(self.elapsed)
                } else {
                    true
                }
            }
            AnimationState::Playing => {
                self.elapsed += dt;
                self.update_playing(self.elapsed)
            }
            AnimationState::Paused => true,
            AnimationState::Completed | AnimationState::Skipped => false,
        }
    }

    /// 更新播放中的动画
    fn update_playing(&mut self, elapsed: f32) -> bool {
        if self.duration <= 0.0 {
            self.progress = 1.0;
            self.state = AnimationState::Completed;
            return false;
        }

        let raw_progress = elapsed / self.duration;
        if raw_progress >= 1.0 {
            self.progress = 1.0;
            self.state = AnimationState::Completed;
            false
        } else {
            self.progress = self.easing.apply(raw_progress);
            true
        }
    }

    /// 跳过动画
    pub fn skip(&mut self) {
        if self.skippable && self.state.is_active() {
            self.progress = 1.0;
            self.state = AnimationState::Skipped;
        }
    }

    /// 强制完成动画（忽略 skippable）
    pub fn force_complete(&mut self) {
        self.progress = 1.0;
        self.state = AnimationState::Completed;
    }

    /// 暂停动画
    pub fn pause(&mut self) {
        if self.state == AnimationState::Playing {
            self.state = AnimationState::Paused;
        }
    }

    /// 恢复动画
    pub fn resume(&mut self) {
        if self.state == AnimationState::Paused {
            self.state = AnimationState::Playing;
        }
    }

    /// 获取当前值
    pub fn current_value(&self) -> f32 {
        self.from + (self.to - self.from) * self.progress
    }

    /// 获取最终值
    pub fn final_value(&self) -> f32 {
        self.to
    }

    /// 获取初始值
    pub fn initial_value(&self) -> f32 {
        self.from
    }

    /// 是否正在播放
    pub fn is_playing(&self) -> bool {
        self.state == AnimationState::Playing
    }

    /// 是否已结束
    pub fn is_finished(&self) -> bool {
        self.state.is_finished()
    }

    /// 是否为活跃状态
    pub fn is_active(&self) -> bool {
        self.state.is_active()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_animation() -> Animation {
        Animation::new_internal(AnimationId::new(1), 0.0, 1.0, 1.0)
    }

    #[test]
    fn test_animation_creation() {
        let anim = create_test_animation();
        assert_eq!(anim.state, AnimationState::Pending);
        assert_eq!(anim.progress, 0.0);
        assert_eq!(anim.from, 0.0);
        assert_eq!(anim.to, 1.0);
    }

    #[test]
    fn test_animation_update() {
        let mut anim = create_test_animation();

        // 开始时是 Pending
        assert!(anim.update(0.1));
        assert_eq!(anim.state, AnimationState::Playing);

        // 进行中
        assert!(anim.update(0.4));
        assert!(anim.progress > 0.0);
        assert!(anim.progress < 1.0);

        // 当前值应该在 0 和 1 之间
        let value = anim.current_value();
        assert!(value > 0.0);
        assert!(value < 1.0);

        // 完成
        assert!(!anim.update(0.6));
        assert_eq!(anim.state, AnimationState::Completed);
        assert_eq!(anim.progress, 1.0);
        assert_eq!(anim.current_value(), 1.0);
    }

    #[test]
    fn test_animation_skip() {
        let mut anim = create_test_animation();
        anim.update(0.1); // 进入 Playing 状态

        anim.skip();
        assert_eq!(anim.state, AnimationState::Skipped);
        assert_eq!(anim.progress, 1.0);
        assert_eq!(anim.current_value(), 1.0);
    }

    #[test]
    fn test_animation_with_delay() {
        let mut anim = create_test_animation().with_delay(0.5);

        // 延迟期间
        assert!(anim.update(0.3));
        assert_eq!(anim.state, AnimationState::Pending);

        // 延迟结束，进入播放
        assert!(anim.update(0.3));
        assert_eq!(anim.state, AnimationState::Playing);
    }

    #[test]
    fn test_zero_duration() {
        let anim = Animation::new_internal(AnimationId::new(1), 0.0, 1.0, 0.0);
        assert_eq!(anim.state, AnimationState::Completed);
    }
}
