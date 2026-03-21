//! # UI 组件模块
//!
//! 提供布局配置、素材缓存、NinePatch、Toast、UI 上下文和界面行为定义等基础设施。

pub mod asset_cache;
pub mod image_slider;
pub mod layout;
pub mod map;
pub mod nine_patch;
pub mod render_context;
pub mod screen_defs;
pub mod toast;

pub use asset_cache::UiAssetCache;
pub use layout::{ScaleContext, UiLayoutConfig};
pub use render_context::UiRenderContext;
pub use screen_defs::{ConditionContext, ScreenDefinitions};
pub use toast::{Toast, ToastManager, ToastType};

/// UI 上下文，存储 UI 渲染所需的共享状态
pub struct UiContext {
    /// 屏幕宽度
    pub screen_width: f32,
    /// 屏幕高度
    pub screen_height: f32,
    /// 分辨率缩放上下文
    pub scale: ScaleContext,
}

impl UiContext {
    /// 创建 UI 上下文
    pub fn new(width: f32, height: f32, layout: &UiLayoutConfig) -> Self {
        Self {
            screen_width: width,
            screen_height: height,
            scale: ScaleContext::new(layout.base_width, layout.base_height, width, height),
        }
    }

    /// 每帧更新状态（当前为 no-op，由 egui 驱动 UI）
    pub fn update(&mut self) {}

    /// 更新屏幕尺寸（由 winit resize 事件驱动）
    pub fn set_screen_size(&mut self, width: f32, height: f32, layout: &UiLayoutConfig) {
        self.screen_width = width;
        self.screen_height = height;
        self.scale = ScaleContext::new(layout.base_width, layout.base_height, width, height);
    }
}
