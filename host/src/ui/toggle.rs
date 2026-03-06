//! # 开关组件

use super::{UiContext, draw_rounded_rect};
use macroquad::prelude::*;

/// 布尔开关
#[derive(Debug, Clone)]
pub struct Toggle {
    pub rect: Rect,
    pub value: bool,
}

impl Toggle {
    pub fn new(rect: Rect, value: bool) -> Self {
        Self { rect, value }
    }

    pub fn set_rect(&mut self, rect: Rect) {
        self.rect = rect;
    }

    pub fn set_value(&mut self, value: bool) {
        self.value = value;
    }

    pub fn update(&mut self, ctx: &UiContext) -> bool {
        if ctx.mouse_in_rect(self.rect) && ctx.mouse_just_released {
            self.value = !self.value;
            return true;
        }
        false
    }

    pub fn draw(&self, ctx: &UiContext) {
        let theme = &ctx.theme;
        let bg = if self.value {
            theme.accent
        } else {
            theme.bg_secondary
        };
        draw_rounded_rect(
            self.rect.x,
            self.rect.y,
            self.rect.w,
            self.rect.h,
            self.rect.h / 2.0,
            bg,
        );

        let knob_margin = 4.0;
        let knob_d = self.rect.h - knob_margin * 2.0;
        let knob_x = if self.value {
            self.rect.x + self.rect.w - knob_d - knob_margin
        } else {
            self.rect.x + knob_margin
        };

        draw_circle(
            knob_x + knob_d / 2.0,
            self.rect.y + self.rect.h / 2.0,
            knob_d / 2.0,
            theme.text_primary,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_value_should_update_state() {
        let mut t = Toggle::new(Rect::new(0.0, 0.0, 100.0, 40.0), false);
        assert!(!t.value);
        t.set_value(true);
        assert!(t.value);
    }
}
