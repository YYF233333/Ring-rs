//! # Background Transition 模块
//!
//! 可动画的背景过渡状态实现，使用 `Animatable` trait。

use std::cell::RefCell;
use std::rc::Rc;

use super::animation::Animatable;

/// 背景过渡的内部数据
#[derive(Debug, Clone)]
pub struct BackgroundTransitionData {
    /// 旧背景透明度 (0.0 - 1.0)
    pub old_alpha: f32,
    /// 新背景透明度 (0.0 - 1.0)
    pub new_alpha: f32,
}

impl Default for BackgroundTransitionData {
    fn default() -> Self {
        Self {
            old_alpha: 0.0,
            new_alpha: 1.0,
        }
    }
}

impl BackgroundTransitionData {
    /// 创建新的背景过渡数据（初始状态：旧背景完全不透明，新背景完全透明）
    pub fn for_transition() -> Self {
        Self {
            old_alpha: 1.0,
            new_alpha: 0.0,
        }
    }

    /// 创建完成状态（新背景完全不透明）
    pub fn completed() -> Self {
        Self::default()
    }
}

/// 可动画的背景过渡
///
/// 使用 `Rc<RefCell<BackgroundTransitionData>>` 包装内部数据。
///
/// ## 支持的属性
///
/// - `"old_alpha"`: 旧背景透明度 (0.0 - 1.0)
/// - `"new_alpha"`: 新背景透明度 (0.0 - 1.0)
#[derive(Debug, Clone)]
pub struct AnimatableBackgroundTransition {
    data: Rc<RefCell<BackgroundTransitionData>>,
}

impl AnimatableBackgroundTransition {
    /// 支持的属性列表
    pub const PROPERTIES: &'static [&'static str] = &["old_alpha", "new_alpha"];

    /// 创建新的背景过渡对象（完成状态）
    pub fn new() -> Self {
        Self {
            data: Rc::new(RefCell::new(BackgroundTransitionData::default())),
        }
    }

    /// 创建用于过渡的背景过渡对象
    pub fn for_transition() -> Self {
        Self {
            data: Rc::new(RefCell::new(BackgroundTransitionData::for_transition())),
        }
    }

    /// 获取旧背景透明度
    pub fn old_alpha(&self) -> f32 {
        self.data.borrow().old_alpha
    }

    /// 获取新背景透明度
    pub fn new_alpha(&self) -> f32 {
        self.data.borrow().new_alpha
    }

    /// 设置旧背景透明度
    pub fn set_old_alpha(&self, alpha: f32) {
        self.data.borrow_mut().old_alpha = alpha.clamp(0.0, 1.0);
    }

    /// 设置新背景透明度
    pub fn set_new_alpha(&self, alpha: f32) {
        self.data.borrow_mut().new_alpha = alpha.clamp(0.0, 1.0);
    }

    /// 重置为过渡开始状态
    pub fn reset_for_transition(&self) {
        let mut data = self.data.borrow_mut();
        data.old_alpha = 1.0;
        data.new_alpha = 0.0;
    }

    /// 设置为完成状态
    pub fn set_completed(&self) {
        let mut data = self.data.borrow_mut();
        data.old_alpha = 0.0;
        data.new_alpha = 1.0;
    }

    /// 获取数据引用
    pub fn data_ref(&self) -> Rc<RefCell<BackgroundTransitionData>> {
        self.data.clone()
    }
}

impl Default for AnimatableBackgroundTransition {
    fn default() -> Self {
        Self::new()
    }
}

impl Animatable for AnimatableBackgroundTransition {
    fn get_property(&self, property_id: &str) -> Option<f32> {
        let data = self.data.borrow();
        match property_id {
            "old_alpha" => Some(data.old_alpha),
            "new_alpha" => Some(data.new_alpha),
            _ => None,
        }
    }

    fn set_property(&self, property_id: &str, value: f32) -> bool {
        let mut data = self.data.borrow_mut();
        match property_id {
            "old_alpha" => {
                data.old_alpha = value.clamp(0.0, 1.0);
                true
            }
            "new_alpha" => {
                data.new_alpha = value.clamp(0.0, 1.0);
                true
            }
            _ => false,
        }
    }

    fn property_list(&self) -> &'static [&'static str] {
        Self::PROPERTIES
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default() {
        let bg = AnimatableBackgroundTransition::new();
        assert_eq!(bg.old_alpha(), 0.0);
        assert_eq!(bg.new_alpha(), 1.0);
    }

    #[test]
    fn test_for_transition() {
        let bg = AnimatableBackgroundTransition::for_transition();
        assert_eq!(bg.old_alpha(), 1.0);
        assert_eq!(bg.new_alpha(), 0.0);
    }

    #[test]
    fn test_animatable_trait() {
        let bg = AnimatableBackgroundTransition::for_transition();

        assert_eq!(bg.get_property("old_alpha"), Some(1.0));
        assert_eq!(bg.get_property("new_alpha"), Some(0.0));

        assert!(bg.set_property("old_alpha", 0.5));
        assert_eq!(bg.old_alpha(), 0.5);

        assert!(bg.set_property("new_alpha", 0.8));
        assert_eq!(bg.new_alpha(), 0.8);

        assert!(!bg.set_property("unknown", 0.0));
    }

    #[test]
    fn test_reset_and_complete() {
        let bg = AnimatableBackgroundTransition::new();

        bg.reset_for_transition();
        assert_eq!(bg.old_alpha(), 1.0);
        assert_eq!(bg.new_alpha(), 0.0);

        bg.set_completed();
        assert_eq!(bg.old_alpha(), 0.0);
        assert_eq!(bg.new_alpha(), 1.0);
    }
}
