//! # 滚动条组件

use super::{UiContext, draw_rounded_rect};

/// 纵向滚动条
#[derive(Debug, Clone)]
pub struct ScrollBar {
    pub track_x: f32,
    pub track_y: f32,
    pub track_h: f32,
    pub content_h: f32,
    pub viewport_h: f32,
    pub scroll_offset: f32,
}

impl ScrollBar {
    pub fn new(
        track_x: f32,
        track_y: f32,
        track_h: f32,
        content_h: f32,
        viewport_h: f32,
        scroll_offset: f32,
    ) -> Self {
        Self {
            track_x,
            track_y,
            track_h,
            content_h,
            viewport_h,
            scroll_offset,
        }
    }

    pub fn should_draw(&self) -> bool {
        self.content_h > self.viewport_h
    }

    pub fn draw(&self, ctx: &UiContext) {
        if !self.should_draw() {
            return;
        }

        let theme = &ctx.theme;
        let bar_w = theme.tokens.control.scroll_bar_width;
        let bar_h = (self.viewport_h / self.content_h) * self.track_h;
        let max_scroll = (self.content_h - self.viewport_h).max(1.0);
        let progress = (self.scroll_offset / max_scroll).clamp(0.0, 1.0);
        let y = self.track_y + progress * (self.track_h - bar_h);

        draw_rounded_rect(
            self.track_x,
            y,
            bar_w,
            bar_h,
            theme.tokens.radius.small,
            macroquad::prelude::Color::new(
                theme.text_secondary.r,
                theme.text_secondary.g,
                theme.text_secondary.b,
                0.5,
            ),
        );
    }
}
