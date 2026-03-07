//! # 滑块组件

use super::{UiContext, draw_rounded_rect};
use macroquad::prelude::*;

/// 数值滑块
#[derive(Debug, Clone)]
pub struct Slider {
    pub rect: Rect,
    pub min: f32,
    pub max: f32,
    pub value: f32,
    dragging: bool,
}

impl Slider {
    pub fn new(rect: Rect, min: f32, max: f32, value: f32) -> Self {
        Self {
            rect,
            min,
            max,
            value: value.clamp(min, max),
            dragging: false,
        }
    }

    pub fn set_rect(&mut self, rect: Rect) {
        self.rect = rect;
    }

    pub fn set_value(&mut self, value: f32) {
        self.value = value.clamp(self.min, self.max);
    }

    pub fn normalized_value(&self) -> f32 {
        let span = (self.max - self.min).max(f32::EPSILON);
        (self.value - self.min) / span
    }

    pub fn update(&mut self, ctx: &UiContext) -> bool {
        if ctx.mouse_just_released {
            self.dragging = false;
        }

        if ctx.mouse_in_rect(self.rect) && ctx.mouse_just_pressed {
            self.dragging = true;
        }

        if !self.dragging {
            return false;
        }

        let old = self.value;
        let ratio = ((ctx.mouse_pos.x - self.rect.x) / self.rect.w).clamp(0.0, 1.0);
        self.value = self.min + ratio * (self.max - self.min);
        (old - self.value).abs() > f32::EPSILON
    }

    pub fn draw(&self, ctx: &UiContext) {
        let theme = &ctx.theme;
        let track_h = theme.tokens.control.slider_track_height;
        let radius = theme.tokens.radius.small;
        let knob_radius = theme.tokens.control.slider_knob_radius;
        let fill_w = self.rect.w * self.normalized_value();
        let y = self.rect.y + self.rect.h / 2.0 - track_h / 2.0;

        draw_rounded_rect(
            self.rect.x,
            y,
            self.rect.w,
            track_h,
            radius,
            theme.bg_secondary,
        );
        draw_rounded_rect(self.rect.x, y, fill_w, track_h, radius, theme.accent);

        let knob_x = (self.rect.x + fill_w).clamp(self.rect.x, self.rect.x + self.rect.w);
        draw_circle(
            knob_x,
            self.rect.y + self.rect.h / 2.0,
            knob_radius,
            theme.text_primary,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalized_value_should_be_clamped_to_range() {
        let mut s = Slider::new(Rect::new(0.0, 0.0, 100.0, 20.0), 10.0, 100.0, 30.0);
        assert!((s.normalized_value() - (20.0 / 90.0)).abs() < 0.0001);

        s.set_value(120.0);
        assert!((s.value - 100.0).abs() < 0.0001);
        assert!((s.normalized_value() - 1.0).abs() < 0.0001);
    }
}
