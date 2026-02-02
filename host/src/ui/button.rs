//! # 按钮组件

use macroquad::prelude::*;
use super::{UiContext, draw_rounded_rect};

/// 按钮状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonState {
    Normal,
    Hovered,
    Pressed,
    Disabled,
}

/// 按钮样式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonStyle {
    /// 主要按钮（强调色）
    Primary,
    /// 次要按钮（普通）
    Secondary,
    /// 危险按钮（红色）
    Danger,
    /// 文字按钮（无背景）
    Text,
}

/// 按钮组件
pub struct Button {
    /// 按钮文本
    pub text: String,
    /// 按钮矩形区域
    pub rect: Rect,
    /// 按钮样式
    pub style: ButtonStyle,
    /// 是否禁用
    pub disabled: bool,
    /// 当前状态
    state: ButtonState,
}

impl Button {
    pub fn new(text: impl Into<String>, x: f32, y: f32, w: f32, h: f32) -> Self {
        Self {
            text: text.into(),
            rect: Rect::new(x, y, w, h),
            style: ButtonStyle::Secondary,
            disabled: false,
            state: ButtonState::Normal,
        }
    }

    /// 设置样式
    pub fn with_style(mut self, style: ButtonStyle) -> Self {
        self.style = style;
        self
    }

    /// 设置禁用状态
    pub fn with_disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// 更新按钮状态并返回是否被点击
    pub fn update(&mut self, ctx: &UiContext) -> bool {
        if self.disabled {
            self.state = ButtonState::Disabled;
            return false;
        }

        let hovered = ctx.mouse_in_rect(self.rect);

        if hovered {
            if ctx.mouse_pressed {
                self.state = ButtonState::Pressed;
            } else {
                self.state = ButtonState::Hovered;
            }

            // 检查点击（鼠标释放时）
            if ctx.mouse_just_released {
                return true;
            }
        } else {
            self.state = ButtonState::Normal;
        }

        false
    }

    /// 绘制按钮
    pub fn draw(&self, ctx: &UiContext, text_renderer: &crate::renderer::TextRenderer) {
        let theme = &ctx.theme;
        
        // 根据样式和状态选择颜色
        let (bg_color, text_color) = match (self.style, self.state) {
            (_, ButtonState::Disabled) => (theme.button_disabled, theme.text_disabled),
            (ButtonStyle::Primary, ButtonState::Normal) => (theme.accent, theme.bg_primary),
            (ButtonStyle::Primary, ButtonState::Hovered) => (theme.accent_hover, theme.bg_primary),
            (ButtonStyle::Primary, ButtonState::Pressed) => (theme.accent_pressed, theme.bg_primary),
            (ButtonStyle::Secondary, ButtonState::Normal) => (theme.button_bg, theme.text_primary),
            (ButtonStyle::Secondary, ButtonState::Hovered) => (theme.button_hover, theme.text_primary),
            (ButtonStyle::Secondary, ButtonState::Pressed) => (theme.button_pressed, theme.text_primary),
            (ButtonStyle::Danger, ButtonState::Normal) => (theme.danger, theme.text_primary),
            (ButtonStyle::Danger, ButtonState::Hovered) => (
                Color::new(theme.danger.r * 1.2, theme.danger.g * 1.2, theme.danger.b * 1.2, 1.0),
                theme.text_primary
            ),
            (ButtonStyle::Danger, ButtonState::Pressed) => (
                Color::new(theme.danger.r * 0.8, theme.danger.g * 0.8, theme.danger.b * 0.8, 1.0),
                theme.text_primary
            ),
            (ButtonStyle::Text, ButtonState::Normal) => (Color::new(0.0, 0.0, 0.0, 0.0), theme.text_primary),
            (ButtonStyle::Text, ButtonState::Hovered) => (Color::new(1.0, 1.0, 1.0, 0.1), theme.accent),
            (ButtonStyle::Text, ButtonState::Pressed) => (Color::new(0.0, 0.0, 0.0, 0.1), theme.accent_pressed),
        };

        // 绘制背景
        if self.style != ButtonStyle::Text || self.state != ButtonState::Normal {
            draw_rounded_rect(
                self.rect.x, self.rect.y,
                self.rect.w, self.rect.h,
                theme.corner_radius,
                bg_color
            );
        }

        // 绘制文字（居中）
        let font_size = theme.font_size_normal;
        // 简单估算文字宽度（每个字符约 font_size * 0.6）
        let text_width = self.text.chars().count() as f32 * font_size * 0.55;
        let text_x = self.rect.x + (self.rect.w - text_width) / 2.0;
        let text_y = self.rect.y + (self.rect.h + font_size * 0.7) / 2.0;

        text_renderer.draw_ui_text(&self.text, text_x, text_y, font_size, text_color);
    }

    /// 获取当前状态
    pub fn state(&self) -> ButtonState {
        self.state
    }
}

/// 创建居中的按钮
pub fn centered_button(text: impl Into<String>, y: f32, screen_width: f32, theme: &super::Theme) -> Button {
    let w = theme.button_min_width;
    let h = theme.button_height;
    let x = (screen_width - w) / 2.0;
    Button::new(text, x, y, w, h)
}

/// 创建菜单按钮列表的布局
pub fn menu_button_layout(
    labels: &[&str],
    start_y: f32,
    screen_width: f32,
    theme: &super::Theme
) -> Vec<Button> {
    let spacing = theme.spacing;
    let mut buttons = Vec::new();
    let mut y = start_y;

    for label in labels {
        buttons.push(centered_button(*label, y, screen_width, theme));
        y += theme.button_height + spacing;
    }

    buttons
}
