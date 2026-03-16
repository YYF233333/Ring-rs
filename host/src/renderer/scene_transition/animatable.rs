use std::cell::RefCell;

use super::super::animation::Animatable;

/// 场景过渡内部数据
#[derive(Debug, Clone)]
struct SceneTransitionData {
    /// 溶解进度（用于 Rule 效果的 shader）
    progress: f32,
    /// 遮罩透明度（用于 Fade/FadeWhite）
    mask_alpha: f32,
    ui_alpha: f32,
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

#[allow(clippy::new_without_default)]
impl AnimatableSceneTransition {
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

    pub fn progress(&self) -> f32 {
        self.inner.borrow().progress
    }

    pub fn mask_alpha(&self) -> f32 {
        self.inner.borrow().mask_alpha
    }

    pub fn ui_alpha(&self) -> f32 {
        self.inner.borrow().ui_alpha
    }

    /// 直接设置进度（用于跳过动画）
    pub fn set_progress(&self, value: f32) {
        self.inner.borrow_mut().progress = value;
    }

    pub fn set_mask_alpha(&self, value: f32) {
        self.inner.borrow_mut().mask_alpha = value;
    }

    pub fn set_ui_alpha(&self, value: f32) {
        self.inner.borrow_mut().ui_alpha = value;
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
