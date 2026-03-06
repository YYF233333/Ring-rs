//! # 标签页组件

use super::{Button, ButtonStyle, UiContext};

/// 标签项
#[derive(Debug, Clone)]
pub struct TabItem {
    pub id: String,
    pub label: String,
}

/// 标签栏
#[derive(Debug, Clone)]
pub struct TabBar {
    pub items: Vec<TabItem>,
    pub selected: usize,
    pub x: f32,
    pub y: f32,
    pub tab_width: f32,
}

impl TabBar {
    pub fn new(items: Vec<TabItem>, selected: usize, x: f32, y: f32, tab_width: f32) -> Self {
        Self {
            items,
            selected,
            x,
            y,
            tab_width,
        }
    }

    pub fn update(&mut self, ctx: &UiContext) -> Option<usize> {
        let mut changed = None;
        let tab_h = ctx.theme.tokens.control.tab_height;
        for (idx, item) in self.items.iter().enumerate() {
            let mut btn = Button::new(
                &item.label,
                self.x + idx as f32 * (self.tab_width + ctx.theme.spacing_small),
                self.y,
                self.tab_width,
                tab_h,
            );
            btn.style = if idx == self.selected {
                ButtonStyle::Primary
            } else {
                ButtonStyle::Secondary
            };
            if btn.update(ctx) {
                changed = Some(idx);
            }
        }
        if let Some(idx) = changed {
            self.selected = idx;
        }
        changed
    }

    pub fn draw(&self, ctx: &UiContext, text_renderer: &crate::renderer::TextRenderer) {
        let tab_h = ctx.theme.tokens.control.tab_height;
        for (idx, item) in self.items.iter().enumerate() {
            let mut btn = Button::new(
                &item.label,
                self.x + idx as f32 * (self.tab_width + ctx.theme.spacing_small),
                self.y,
                self.tab_width,
                tab_h,
            );
            btn.style = if idx == self.selected {
                ButtonStyle::Primary
            } else {
                ButtonStyle::Secondary
            };
            btn.draw(ctx, text_renderer);
        }
    }
}
