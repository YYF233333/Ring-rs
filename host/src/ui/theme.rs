//! # UI 主题
//!
//! 定义 UI 的颜色、字体大小、间距等样式。

use macroquad::prelude::Color;

/// 颜色 tokens
#[derive(Debug, Clone)]
pub struct PaletteTokens {
    pub bg_primary: Color,
    pub bg_secondary: Color,
    pub bg_panel: Color,
    pub bg_overlay: Color,
    pub text_primary: Color,
    pub text_secondary: Color,
    pub text_disabled: Color,
    pub accent: Color,
    pub accent_hover: Color,
    pub accent_pressed: Color,
    pub button_bg: Color,
    pub button_hover: Color,
    pub button_pressed: Color,
    pub button_disabled: Color,
    pub danger: Color,
    pub success: Color,
    pub warning: Color,
}

/// 排版 tokens
#[derive(Debug, Clone)]
pub struct TypographyTokens {
    pub title: f32,
    pub large: f32,
    pub normal: f32,
    pub small: f32,
}

/// 间距 tokens
#[derive(Debug, Clone)]
pub struct SpacingTokens {
    pub small: f32,
    pub medium: f32,
    pub large: f32,
    pub padding: f32,
}

/// 圆角 tokens
#[derive(Debug, Clone)]
pub struct RadiusTokens {
    pub small: f32,
    pub medium: f32,
}

/// 视觉层级 tokens
#[derive(Debug, Clone)]
pub struct ElevationTokens {
    pub panel_border_thickness: f32,
    pub separator_alpha: f32,
    pub overlay_alpha: f32,
}

/// 控件尺寸 tokens
#[derive(Debug, Clone)]
pub struct ControlTokens {
    pub button_height: f32,
    pub button_min_width: f32,
    pub tab_height: f32,
    pub slider_track_height: f32,
    pub slider_knob_radius: f32,
    pub toggle_height: f32,
    pub list_item_height: f32,
    pub scroll_bar_width: f32,
    pub toast_width: f32,
    pub toast_height: f32,
    pub modal_width: f32,
    pub modal_height: f32,
}

/// UI 样式 token 聚合
#[derive(Debug, Clone)]
pub struct ThemeTokens {
    pub palette: PaletteTokens,
    pub typography: TypographyTokens,
    pub spacing: SpacingTokens,
    pub radius: RadiusTokens,
    pub elevation: ElevationTokens,
    pub control: ControlTokens,
}

/// UI 主题配置
#[derive(Debug, Clone)]
pub struct Theme {
    /// 分组 token（阶段29主入口）
    pub tokens: ThemeTokens,

    // ===== 兼容字段（阶段29过渡期）=====
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
    fn from_tokens(tokens: ThemeTokens) -> Self {
        let mut theme = Self {
            tokens,

            // 兼容字段先初始化为占位，随后统一同步
            bg_primary: Color::new(0.0, 0.0, 0.0, 1.0),
            bg_secondary: Color::new(0.0, 0.0, 0.0, 1.0),
            bg_panel: Color::new(0.0, 0.0, 0.0, 1.0),
            bg_overlay: Color::new(0.0, 0.0, 0.0, 1.0),
            text_primary: Color::new(1.0, 1.0, 1.0, 1.0),
            text_secondary: Color::new(1.0, 1.0, 1.0, 1.0),
            text_disabled: Color::new(1.0, 1.0, 1.0, 1.0),
            accent: Color::new(1.0, 1.0, 1.0, 1.0),
            accent_hover: Color::new(1.0, 1.0, 1.0, 1.0),
            accent_pressed: Color::new(1.0, 1.0, 1.0, 1.0),
            button_bg: Color::new(1.0, 1.0, 1.0, 1.0),
            button_hover: Color::new(1.0, 1.0, 1.0, 1.0),
            button_pressed: Color::new(1.0, 1.0, 1.0, 1.0),
            button_disabled: Color::new(1.0, 1.0, 1.0, 1.0),
            danger: Color::new(1.0, 0.0, 0.0, 1.0),
            success: Color::new(0.0, 1.0, 0.0, 1.0),
            warning: Color::new(1.0, 1.0, 0.0, 1.0),
            font_size_title: 0.0,
            font_size_large: 0.0,
            font_size_normal: 0.0,
            font_size_small: 0.0,
            button_height: 0.0,
            button_min_width: 0.0,
            corner_radius: 0.0,
            spacing: 0.0,
            spacing_large: 0.0,
            spacing_small: 0.0,
            padding: 0.0,
        };

        theme.sync_legacy_fields();
        theme
    }

    /// 将 token 同步到旧字段，保证阶段29改造期间 API 兼容
    pub fn sync_legacy_fields(&mut self) {
        self.bg_primary = self.tokens.palette.bg_primary;
        self.bg_secondary = self.tokens.palette.bg_secondary;
        self.bg_panel = self.tokens.palette.bg_panel;
        self.bg_overlay = self.tokens.palette.bg_overlay;
        self.text_primary = self.tokens.palette.text_primary;
        self.text_secondary = self.tokens.palette.text_secondary;
        self.text_disabled = self.tokens.palette.text_disabled;
        self.accent = self.tokens.palette.accent;
        self.accent_hover = self.tokens.palette.accent_hover;
        self.accent_pressed = self.tokens.palette.accent_pressed;
        self.button_bg = self.tokens.palette.button_bg;
        self.button_hover = self.tokens.palette.button_hover;
        self.button_pressed = self.tokens.palette.button_pressed;
        self.button_disabled = self.tokens.palette.button_disabled;
        self.danger = self.tokens.palette.danger;
        self.success = self.tokens.palette.success;
        self.warning = self.tokens.palette.warning;

        self.font_size_title = self.tokens.typography.title;
        self.font_size_large = self.tokens.typography.large;
        self.font_size_normal = self.tokens.typography.normal;
        self.font_size_small = self.tokens.typography.small;

        self.button_height = self.tokens.control.button_height;
        self.button_min_width = self.tokens.control.button_min_width;
        self.corner_radius = self.tokens.radius.medium;
        self.spacing = self.tokens.spacing.medium;
        self.spacing_large = self.tokens.spacing.large;
        self.spacing_small = self.tokens.spacing.small;
        self.padding = self.tokens.spacing.padding;
    }

    /// 深色主题（默认）
    pub fn dark() -> Self {
        Self::from_tokens(ThemeTokens {
            palette: PaletteTokens {
                bg_primary: Color::new(0.08, 0.08, 0.12, 1.0),
                bg_secondary: Color::new(0.12, 0.12, 0.18, 1.0),
                bg_panel: Color::new(0.15, 0.15, 0.22, 0.95),
                bg_overlay: Color::new(0.0, 0.0, 0.0, 0.7),
                text_primary: Color::new(0.95, 0.95, 0.97, 1.0),
                text_secondary: Color::new(0.7, 0.7, 0.75, 1.0),
                text_disabled: Color::new(0.4, 0.4, 0.45, 1.0),
                accent: Color::new(0.85, 0.65, 0.3, 1.0),
                accent_hover: Color::new(0.95, 0.75, 0.4, 1.0),
                accent_pressed: Color::new(0.7, 0.55, 0.25, 1.0),
                button_bg: Color::new(0.2, 0.2, 0.28, 1.0),
                button_hover: Color::new(0.28, 0.28, 0.38, 1.0),
                button_pressed: Color::new(0.15, 0.15, 0.22, 1.0),
                button_disabled: Color::new(0.15, 0.15, 0.18, 1.0),
                danger: Color::new(0.85, 0.3, 0.3, 1.0),
                success: Color::new(0.3, 0.75, 0.4, 1.0),
                warning: Color::new(0.9, 0.7, 0.2, 1.0),
            },
            typography: TypographyTokens {
                title: 48.0,
                large: 28.0,
                normal: 22.0,
                small: 16.0,
            },
            spacing: SpacingTokens {
                small: 8.0,
                medium: 16.0,
                large: 32.0,
                padding: 20.0,
            },
            radius: RadiusTokens {
                small: 4.0,
                medium: 8.0,
            },
            elevation: ElevationTokens {
                panel_border_thickness: 2.0,
                separator_alpha: 0.3,
                overlay_alpha: 0.7,
            },
            control: ControlTokens {
                button_height: 50.0,
                button_min_width: 200.0,
                tab_height: 40.0,
                slider_track_height: 8.0,
                slider_knob_radius: 10.0,
                toggle_height: 35.0,
                list_item_height: 70.0,
                scroll_bar_width: 4.0,
                toast_width: 300.0,
                toast_height: 50.0,
                modal_width: 400.0,
                modal_height: 200.0,
            },
        })
    }

    /// 浅色主题
    pub fn light() -> Self {
        Self::from_tokens(ThemeTokens {
            palette: PaletteTokens {
                bg_primary: Color::new(0.95, 0.95, 0.97, 1.0),
                bg_secondary: Color::new(0.9, 0.9, 0.92, 1.0),
                bg_panel: Color::new(1.0, 1.0, 1.0, 0.95),
                bg_overlay: Color::new(0.0, 0.0, 0.0, 0.5),
                text_primary: Color::new(0.1, 0.1, 0.15, 1.0),
                text_secondary: Color::new(0.4, 0.4, 0.45, 1.0),
                text_disabled: Color::new(0.6, 0.6, 0.65, 1.0),
                accent: Color::new(0.2, 0.5, 0.85, 1.0),
                accent_hover: Color::new(0.3, 0.6, 0.95, 1.0),
                accent_pressed: Color::new(0.15, 0.4, 0.7, 1.0),
                button_bg: Color::new(0.85, 0.85, 0.88, 1.0),
                button_hover: Color::new(0.8, 0.8, 0.85, 1.0),
                button_pressed: Color::new(0.75, 0.75, 0.8, 1.0),
                button_disabled: Color::new(0.9, 0.9, 0.92, 1.0),
                danger: Color::new(0.85, 0.25, 0.25, 1.0),
                success: Color::new(0.2, 0.7, 0.35, 1.0),
                warning: Color::new(0.9, 0.65, 0.1, 1.0),
            },
            typography: TypographyTokens {
                title: 48.0,
                large: 28.0,
                normal: 22.0,
                small: 16.0,
            },
            spacing: SpacingTokens {
                small: 8.0,
                medium: 16.0,
                large: 32.0,
                padding: 20.0,
            },
            radius: RadiusTokens {
                small: 4.0,
                medium: 8.0,
            },
            elevation: ElevationTokens {
                panel_border_thickness: 2.0,
                separator_alpha: 0.3,
                overlay_alpha: 0.5,
            },
            control: ControlTokens {
                button_height: 50.0,
                button_min_width: 200.0,
                tab_height: 40.0,
                slider_track_height: 8.0,
                slider_knob_radius: 10.0,
                toggle_height: 35.0,
                list_item_height: 70.0,
                scroll_bar_width: 4.0,
                toast_width: 300.0,
                toast_height: 50.0,
                modal_width: 400.0,
                modal_height: 200.0,
            },
        })
    }
}
