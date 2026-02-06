//! # 模态对话框组件

use super::{Button, ButtonStyle, Panel, UiContext};
use macroquad::prelude::*;

/// 模态对话框结果
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModalResult {
    /// 无操作
    None,
    /// 确认
    Confirm,
    /// 取消
    Cancel,
}

/// 模态对话框
pub struct Modal {
    /// 标题
    pub title: String,
    /// 消息内容
    pub message: String,
    /// 确认按钮文本
    pub confirm_text: String,
    /// 取消按钮文本
    pub cancel_text: String,
    /// 是否显示取消按钮
    pub show_cancel: bool,
    /// 确认按钮是否危险操作
    pub is_danger: bool,
    /// 内部按钮状态
    confirm_button: Option<Button>,
    cancel_button: Option<Button>,
}

impl Modal {
    /// 创建确认对话框
    pub fn confirm(title: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            message: message.into(),
            confirm_text: "确定".to_string(),
            cancel_text: "取消".to_string(),
            show_cancel: true,
            is_danger: false,
            confirm_button: None,
            cancel_button: None,
        }
    }

    /// 创建警告对话框（只有确定按钮）
    pub fn alert(title: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            message: message.into(),
            confirm_text: "确定".to_string(),
            cancel_text: "取消".to_string(),
            show_cancel: false,
            is_danger: false,
            confirm_button: None,
            cancel_button: None,
        }
    }

    /// 设置确认按钮文本
    pub fn with_confirm_text(mut self, text: impl Into<String>) -> Self {
        self.confirm_text = text.into();
        self
    }

    /// 设置取消按钮文本
    pub fn with_cancel_text(mut self, text: impl Into<String>) -> Self {
        self.cancel_text = text.into();
        self
    }

    /// 设置为危险操作
    pub fn with_danger(mut self, is_danger: bool) -> Self {
        self.is_danger = is_danger;
        self
    }

    /// 更新并返回结果
    pub fn update(&mut self, ctx: &UiContext) -> ModalResult {
        let theme = &ctx.theme;

        // 计算对话框尺寸和位置
        let modal_width = 400.0;
        let modal_height = 200.0;
        let modal_x = (ctx.screen_width - modal_width) / 2.0;
        let modal_y = (ctx.screen_height - modal_height) / 2.0;

        let button_width = 120.0;
        let button_height = theme.button_height;
        let button_y = modal_y + modal_height - button_height - theme.padding;

        // 初始化或更新按钮位置
        if self.show_cancel {
            let confirm_x = modal_x + modal_width / 2.0 - button_width - theme.spacing_small;
            let cancel_x = modal_x + modal_width / 2.0 + theme.spacing_small;

            if self.confirm_button.is_none() {
                let mut btn = Button::new(
                    &self.confirm_text,
                    confirm_x,
                    button_y,
                    button_width,
                    button_height,
                );
                btn.style = if self.is_danger {
                    ButtonStyle::Danger
                } else {
                    ButtonStyle::Primary
                };
                self.confirm_button = Some(btn);
            }
            if self.cancel_button.is_none() {
                self.cancel_button = Some(Button::new(
                    &self.cancel_text,
                    cancel_x,
                    button_y,
                    button_width,
                    button_height,
                ));
            }
        } else {
            let confirm_x = modal_x + (modal_width - button_width) / 2.0;
            if self.confirm_button.is_none() {
                let mut btn = Button::new(
                    &self.confirm_text,
                    confirm_x,
                    button_y,
                    button_width,
                    button_height,
                );
                btn.style = ButtonStyle::Primary;
                self.confirm_button = Some(btn);
            }
        }

        // 更新按钮
        if let Some(ref mut btn) = self.confirm_button
            && btn.update(ctx)
        {
            return ModalResult::Confirm;
        }
        if let Some(ref mut btn) = self.cancel_button
            && btn.update(ctx)
        {
            return ModalResult::Cancel;
        }

        // ESC 关闭
        if is_key_pressed(KeyCode::Escape) && self.show_cancel {
            return ModalResult::Cancel;
        }

        // Enter 确认
        if is_key_pressed(KeyCode::Enter) {
            return ModalResult::Confirm;
        }

        ModalResult::None
    }

    /// 绘制对话框
    pub fn draw(&self, ctx: &UiContext, text_renderer: &crate::renderer::TextRenderer) {
        let theme = &ctx.theme;

        // 绘制覆盖层
        draw_rectangle(
            0.0,
            0.0,
            ctx.screen_width,
            ctx.screen_height,
            theme.bg_overlay,
        );

        // 计算对话框尺寸
        let modal_width = 400.0;
        let modal_height = 200.0;
        let modal_x = (ctx.screen_width - modal_width) / 2.0;
        let modal_y = (ctx.screen_height - modal_height) / 2.0;

        // 绘制面板
        let panel = Panel::new(modal_x, modal_y, modal_width, modal_height).with_title(&self.title);
        panel.draw(ctx, text_renderer);

        // 绘制消息
        let content_rect = panel.content_rect(theme);
        text_renderer.draw_ui_text(
            &self.message,
            content_rect.x,
            content_rect.y + theme.font_size_normal,
            theme.font_size_normal,
            theme.text_primary,
        );

        // 绘制按钮
        if let Some(ref btn) = self.confirm_button {
            btn.draw(ctx, text_renderer);
        }
        if let Some(ref btn) = self.cancel_button {
            btn.draw(ctx, text_renderer);
        }
    }
}
