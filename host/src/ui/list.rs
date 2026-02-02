//! # 列表组件

use macroquad::prelude::*;
use super::{UiContext, draw_rounded_rect};

/// 列表项
#[derive(Debug, Clone)]
pub struct ListItem {
    /// 主文本
    pub text: String,
    /// 次要文本（可选）
    pub secondary_text: Option<String>,
    /// 附加数据（用于识别）
    pub data: String,
    /// 是否禁用
    pub disabled: bool,
}

impl ListItem {
    pub fn new(text: impl Into<String>, data: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            secondary_text: None,
            data: data.into(),
            disabled: false,
        }
    }

    pub fn with_secondary(mut self, text: impl Into<String>) -> Self {
        self.secondary_text = Some(text.into());
        self
    }

    pub fn with_disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

/// 列表视图
pub struct ListView {
    /// 列表区域
    pub rect: Rect,
    /// 列表项
    pub items: Vec<ListItem>,
    /// 当前选中索引
    pub selected_index: Option<usize>,
    /// 当前悬停索引
    pub hovered_index: Option<usize>,
    /// 滚动偏移
    pub scroll_offset: f32,
    /// 每项高度
    pub item_height: f32,
}

impl ListView {
    pub fn new(rect: Rect, item_height: f32) -> Self {
        Self {
            rect,
            items: Vec::new(),
            selected_index: None,
            hovered_index: None,
            scroll_offset: 0.0,
            item_height,
        }
    }

    /// 设置列表项
    pub fn set_items(&mut self, items: Vec<ListItem>) {
        self.items = items;
        self.scroll_offset = 0.0;
        // 保持选中索引有效
        if let Some(idx) = self.selected_index {
            if idx >= self.items.len() {
                self.selected_index = if self.items.is_empty() { None } else { Some(self.items.len() - 1) };
            }
        }
    }

    /// 可见项的数量
    pub fn visible_count(&self) -> usize {
        (self.rect.h / self.item_height).floor() as usize
    }

    /// 总高度
    pub fn total_height(&self) -> f32 {
        self.items.len() as f32 * self.item_height
    }

    /// 是否可滚动
    pub fn can_scroll(&self) -> bool {
        self.total_height() > self.rect.h
    }

    /// 选择上一项
    pub fn select_prev(&mut self) {
        if self.items.is_empty() {
            return;
        }
        match self.selected_index {
            Some(idx) if idx > 0 => {
                self.selected_index = Some(idx - 1);
            }
            None => {
                self.selected_index = Some(self.items.len() - 1);
            }
            _ => {}
        }
        self.ensure_selected_visible();
    }

    /// 选择下一项
    pub fn select_next(&mut self) {
        if self.items.is_empty() {
            return;
        }
        match self.selected_index {
            Some(idx) if idx < self.items.len() - 1 => {
                self.selected_index = Some(idx + 1);
            }
            None => {
                self.selected_index = Some(0);
            }
            _ => {}
        }
        self.ensure_selected_visible();
    }

    /// 确保选中项可见
    fn ensure_selected_visible(&mut self) {
        if let Some(idx) = self.selected_index {
            let item_top = idx as f32 * self.item_height;
            let item_bottom = item_top + self.item_height;

            if item_top < self.scroll_offset {
                self.scroll_offset = item_top;
            } else if item_bottom > self.scroll_offset + self.rect.h {
                self.scroll_offset = item_bottom - self.rect.h;
            }
        }
    }

    /// 更新列表状态，返回被点击的项索引
    pub fn update(&mut self, ctx: &UiContext) -> Option<usize> {
        // 检查鼠标是否在列表区域内
        if !ctx.mouse_in_rect(self.rect) {
            self.hovered_index = None;
            return None;
        }

        // 处理滚动
        let scroll = mouse_wheel().1;
        if scroll != 0.0 {
            self.scroll_offset -= scroll * self.item_height * 0.5;
            self.scroll_offset = self.scroll_offset
                .max(0.0)
                .min((self.total_height() - self.rect.h).max(0.0));
        }

        // 计算悬停项
        let relative_y = ctx.mouse_pos.y - self.rect.y + self.scroll_offset;
        let hover_idx = (relative_y / self.item_height).floor() as usize;
        
        if hover_idx < self.items.len() && !self.items[hover_idx].disabled {
            self.hovered_index = Some(hover_idx);
            
            // 检查点击
            if ctx.mouse_just_released {
                self.selected_index = Some(hover_idx);
                return Some(hover_idx);
            }
        } else {
            self.hovered_index = None;
        }

        None
    }

    /// 绘制列表
    pub fn draw(&self, ctx: &UiContext, text_renderer: &crate::renderer::TextRenderer) {
        let theme = &ctx.theme;

        // 裁剪区域（防止绘制到列表外）
        // macroquad 没有直接的裁剪，我们手动处理可见项

        let start_idx = (self.scroll_offset / self.item_height).floor() as usize;
        let end_idx = ((self.scroll_offset + self.rect.h) / self.item_height).ceil() as usize;
        let end_idx = end_idx.min(self.items.len());

        for idx in start_idx..end_idx {
            let item = &self.items[idx];
            let item_y = self.rect.y + idx as f32 * self.item_height - self.scroll_offset;

            // 跳过完全不可见的项
            if item_y + self.item_height < self.rect.y || item_y > self.rect.y + self.rect.h {
                continue;
            }

            // 选中/悬停背景
            let is_selected = self.selected_index == Some(idx);
            let is_hovered = self.hovered_index == Some(idx);
            
            if is_selected {
                draw_rounded_rect(
                    self.rect.x, item_y,
                    self.rect.w, self.item_height - 2.0,
                    theme.corner_radius / 2.0,
                    theme.accent
                );
            } else if is_hovered && !item.disabled {
                draw_rounded_rect(
                    self.rect.x, item_y,
                    self.rect.w, self.item_height - 2.0,
                    theme.corner_radius / 2.0,
                    theme.button_hover
                );
            }

            // 文字颜色
            let text_color = if item.disabled {
                theme.text_disabled
            } else if is_selected {
                theme.bg_primary
            } else {
                theme.text_primary
            };

            let secondary_color = if item.disabled {
                theme.text_disabled
            } else if is_selected {
                Color::new(theme.bg_primary.r, theme.bg_primary.g, theme.bg_primary.b, 0.8)
            } else {
                theme.text_secondary
            };

            // 绘制主文本
            let text_y = if item.secondary_text.is_some() {
                item_y + self.item_height * 0.35
            } else {
                item_y + (self.item_height + theme.font_size_normal * 0.7) / 2.0
            };

            text_renderer.draw_ui_text(
                &item.text,
                self.rect.x + theme.spacing,
                text_y,
                theme.font_size_normal,
                text_color
            );

            // 绘制次要文本
            if let Some(ref secondary) = item.secondary_text {
                text_renderer.draw_ui_text(
                    secondary,
                    self.rect.x + theme.spacing,
                    item_y + self.item_height * 0.7,
                    theme.font_size_small,
                    secondary_color
                );
            }
        }

        // 绘制滚动条（如果需要）
        if self.can_scroll() {
            let scrollbar_height = (self.rect.h / self.total_height()) * self.rect.h;
            let scrollbar_y = self.rect.y + 
                (self.scroll_offset / (self.total_height() - self.rect.h)) * (self.rect.h - scrollbar_height);

            draw_rounded_rect(
                self.rect.x + self.rect.w - 6.0,
                scrollbar_y,
                4.0,
                scrollbar_height,
                2.0,
                Color::new(theme.text_secondary.r, theme.text_secondary.g, theme.text_secondary.b, 0.5)
            );
        }
    }
}
