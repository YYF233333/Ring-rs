//! # 面板组件

use macroquad::prelude::*;
use super::{UiContext, draw_rounded_rect, draw_rounded_rect_lines};

/// 面板组件
pub struct Panel {
    /// 面板矩形区域
    pub rect: Rect,
    /// 标题（可选）
    pub title: Option<String>,
    /// 是否显示边框
    pub show_border: bool,
    /// 是否显示标题栏
    pub show_title_bar: bool,
}

impl Panel {
    pub fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        Self {
            rect: Rect::new(x, y, w, h),
            title: None,
            show_border: true,
            show_title_bar: false,
        }
    }

    /// 创建居中面板
    pub fn centered(w: f32, h: f32, screen_width: f32, screen_height: f32) -> Self {
        let x = (screen_width - w) / 2.0;
        let y = (screen_height - h) / 2.0;
        Self::new(x, y, w, h)
    }

    /// 设置标题
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self.show_title_bar = true;
        self
    }

    /// 设置是否显示边框
    pub fn with_border(mut self, show: bool) -> Self {
        self.show_border = show;
        self
    }

    /// 获取内容区域（排除标题栏）
    pub fn content_rect(&self, theme: &super::Theme) -> Rect {
        if self.show_title_bar {
            let title_height = theme.font_size_large + theme.padding;
            Rect::new(
                self.rect.x + theme.padding,
                self.rect.y + title_height + theme.spacing_small,
                self.rect.w - theme.padding * 2.0,
                self.rect.h - title_height - theme.padding - theme.spacing_small,
            )
        } else {
            Rect::new(
                self.rect.x + theme.padding,
                self.rect.y + theme.padding,
                self.rect.w - theme.padding * 2.0,
                self.rect.h - theme.padding * 2.0,
            )
        }
    }

    /// 绘制面板
    pub fn draw(&self, ctx: &UiContext, text_renderer: &crate::renderer::TextRenderer) {
        let theme = &ctx.theme;

        // 绘制背景
        draw_rounded_rect(
            self.rect.x, self.rect.y,
            self.rect.w, self.rect.h,
            theme.corner_radius,
            theme.bg_panel
        );

        // 绘制边框
        if self.show_border {
            draw_rounded_rect_lines(
                self.rect.x, self.rect.y,
                self.rect.w, self.rect.h,
                theme.corner_radius,
                2.0,
                theme.accent
            );
        }

        // 绘制标题栏
        if self.show_title_bar {
            let title_height = theme.font_size_large + theme.padding;
            
            // 标题栏背景
            draw_rounded_rect(
                self.rect.x + 2.0, self.rect.y + 2.0,
                self.rect.w - 4.0, title_height,
                theme.corner_radius - 2.0,
                theme.bg_secondary
            );

            // 分隔线
            draw_line(
                self.rect.x + theme.padding,
                self.rect.y + title_height,
                self.rect.x + self.rect.w - theme.padding,
                self.rect.y + title_height,
                1.0,
                Color::new(theme.accent.r, theme.accent.g, theme.accent.b, 0.3)
            );

            // 标题文字
            if let Some(ref title) = self.title {
                text_renderer.draw_ui_text(
                    title,
                    self.rect.x + theme.padding,
                    self.rect.y + theme.padding + theme.font_size_large * 0.8,
                    theme.font_size_large,
                    theme.text_primary
                );
            }
        }
    }
}

/// 全屏覆盖层（半透明背景）
pub fn draw_overlay(ctx: &UiContext) {
    draw_rectangle(
        0.0, 0.0,
        ctx.screen_width, ctx.screen_height,
        ctx.theme.bg_overlay
    );
}
