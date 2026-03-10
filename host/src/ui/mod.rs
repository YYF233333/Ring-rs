//! # UI 组件模块
//!
//! 提供主题、Toast 和 UI 上下文等基础设施。
//! 旧的 macroquad UI 组件已移除，界面渲染由 egui 接管。

pub mod skin;
pub mod theme;
pub mod theme_loader;
pub mod toast;

pub use skin::{UiSkinConfig, load_skin};
pub use theme::Theme;
pub use theme_loader::load_theme_with_override;
pub use toast::{Toast, ToastManager, ToastType};

/// UI 上下文，存储 UI 渲染所需的共享状态
pub struct UiContext {
    /// 当前主题
    pub theme: Theme,
    /// 屏幕宽度
    pub screen_width: f32,
    /// 屏幕高度
    pub screen_height: f32,
    /// UI 皮肤配置（可选）
    pub skin: Option<UiSkinConfig>,
}

impl UiContext {
    /// 创建 UI 上下文
    pub fn new(theme: Theme, width: f32, height: f32) -> Self {
        Self {
            theme,
            screen_width: width,
            screen_height: height,
            skin: None,
        }
    }

    /// 每帧更新状态（当前为 no-op，由 egui 驱动 UI）
    pub fn update(&mut self) {}

    /// 更新屏幕尺寸（由 winit resize 事件驱动）
    pub fn set_screen_size(&mut self, width: f32, height: f32) {
        self.screen_width = width;
        self.screen_height = height;
    }
}
