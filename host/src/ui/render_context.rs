//! # UI 渲染上下文
//!
//! 合并 `(layout, assets, scale, screen_defs)` 为统一上下文包，
//! 替代 `build_*` 函数的多参数传递模式。

use crate::ui::asset_cache::UiAssetCache;
use crate::ui::layout::{ScaleContext, UiLayoutConfig};
use crate::ui::screen_defs::{ConditionContext, ScreenDefinitions};

/// 所有 `build_*` 函数的公共 UI 上下文
///
/// 在 `host_app.rs` 渲染循环开始时一次性构造，传给所有 `build_*` 函数，
/// 使其不再需要直接访问 `AppState`。
pub struct UiRenderContext<'a> {
    pub layout: &'a UiLayoutConfig,
    pub assets: Option<&'a UiAssetCache>,
    pub scale: &'a ScaleContext,
    pub screen_defs: &'a ScreenDefinitions,
    pub conditions: ConditionContext<'a>,
}
