//! # UI 主题
//!
//! 定义 UI 的颜色、字体大小、间距等样式。

use macroquad::prelude::Color;

/// UI 主题配置
#[derive(Debug, Clone)]
pub struct Theme {
    // ===== 颜色 =====
    /// 主背景色（深色）
    pub bg_primary: Color,
    /// 次要背景色（稍浅）
    pub bg_secondary: Color,
    /// 面板背景色（半透明）
    pub bg_panel: Color,
    /// 覆盖层背景色（半透明黑）
    pub bg_overlay: Color,

    /// 主文字色
    pub text_primary: Color,
    /// 次要文字色
    pub text_secondary: Color,
    /// 禁用状态文字色
    pub text_disabled: Color,

    /// 强调色（用于高亮、选中等）
    pub accent: Color,
    /// 强调色悬停
    pub accent_hover: Color,
    /// 强调色按下
    pub accent_pressed: Color,

    /// 按钮默认背景
    pub button_bg: Color,
    /// 按钮悬停背景
    pub button_hover: Color,
    /// 按钮按下背景
    pub button_pressed: Color,
    /// 按钮禁用背景
    pub button_disabled: Color,

    /// 危险操作色（删除等）
    pub danger: Color,
    /// 成功色
    pub success: Color,
    /// 警告色
    pub warning: Color,

    // ===== 尺寸 =====
    /// 标题字号
    pub font_size_title: f32,
    /// 大字号
    pub font_size_large: f32,
    /// 正常字号
    pub font_size_normal: f32,
    /// 小字号
    pub font_size_small: f32,

    /// 按钮高度
    pub button_height: f32,
    /// 按钮最小宽度
    pub button_min_width: f32,
    /// 圆角半径
    pub corner_radius: f32,
    /// 标准间距
    pub spacing: f32,
    /// 大间距
    pub spacing_large: f32,
    /// 小间距
    pub spacing_small: f32,
    /// 内边距
    pub padding: f32,
}

impl Default for Theme {
    fn default() -> Self {
        Self::dark()
    }
}

impl Theme {
    /// 深色主题（默认）
    pub fn dark() -> Self {
        Self {
            // 背景
            bg_primary: Color::new(0.08, 0.08, 0.12, 1.0),
            bg_secondary: Color::new(0.12, 0.12, 0.18, 1.0),
            bg_panel: Color::new(0.15, 0.15, 0.22, 0.95),
            bg_overlay: Color::new(0.0, 0.0, 0.0, 0.7),

            // 文字
            text_primary: Color::new(0.95, 0.95, 0.97, 1.0),
            text_secondary: Color::new(0.7, 0.7, 0.75, 1.0),
            text_disabled: Color::new(0.4, 0.4, 0.45, 1.0),

            // 强调色（金色调）
            accent: Color::new(0.85, 0.65, 0.3, 1.0),
            accent_hover: Color::new(0.95, 0.75, 0.4, 1.0),
            accent_pressed: Color::new(0.7, 0.55, 0.25, 1.0),

            // 按钮
            button_bg: Color::new(0.2, 0.2, 0.28, 1.0),
            button_hover: Color::new(0.28, 0.28, 0.38, 1.0),
            button_pressed: Color::new(0.15, 0.15, 0.22, 1.0),
            button_disabled: Color::new(0.15, 0.15, 0.18, 1.0),

            // 状态色
            danger: Color::new(0.85, 0.3, 0.3, 1.0),
            success: Color::new(0.3, 0.75, 0.4, 1.0),
            warning: Color::new(0.9, 0.7, 0.2, 1.0),

            // 字号
            font_size_title: 48.0,
            font_size_large: 28.0,
            font_size_normal: 22.0,
            font_size_small: 16.0,

            // 尺寸
            button_height: 50.0,
            button_min_width: 200.0,
            corner_radius: 8.0,
            spacing: 16.0,
            spacing_large: 32.0,
            spacing_small: 8.0,
            padding: 20.0,
        }
    }

    /// 浅色主题
    pub fn light() -> Self {
        Self {
            // 背景
            bg_primary: Color::new(0.95, 0.95, 0.97, 1.0),
            bg_secondary: Color::new(0.9, 0.9, 0.92, 1.0),
            bg_panel: Color::new(1.0, 1.0, 1.0, 0.95),
            bg_overlay: Color::new(0.0, 0.0, 0.0, 0.5),

            // 文字
            text_primary: Color::new(0.1, 0.1, 0.15, 1.0),
            text_secondary: Color::new(0.4, 0.4, 0.45, 1.0),
            text_disabled: Color::new(0.6, 0.6, 0.65, 1.0),

            // 强调色（蓝色调）
            accent: Color::new(0.2, 0.5, 0.85, 1.0),
            accent_hover: Color::new(0.3, 0.6, 0.95, 1.0),
            accent_pressed: Color::new(0.15, 0.4, 0.7, 1.0),

            // 按钮
            button_bg: Color::new(0.85, 0.85, 0.88, 1.0),
            button_hover: Color::new(0.8, 0.8, 0.85, 1.0),
            button_pressed: Color::new(0.75, 0.75, 0.8, 1.0),
            button_disabled: Color::new(0.9, 0.9, 0.92, 1.0),

            // 状态色
            danger: Color::new(0.85, 0.25, 0.25, 1.0),
            success: Color::new(0.2, 0.7, 0.35, 1.0),
            warning: Color::new(0.9, 0.65, 0.1, 1.0),

            // 字号
            font_size_title: 48.0,
            font_size_large: 28.0,
            font_size_normal: 22.0,
            font_size_small: 16.0,

            // 尺寸
            button_height: 50.0,
            button_min_width: 200.0,
            corner_radius: 8.0,
            spacing: 16.0,
            spacing_large: 32.0,
            spacing_small: 8.0,
            padding: 20.0,
        }
    }
}
