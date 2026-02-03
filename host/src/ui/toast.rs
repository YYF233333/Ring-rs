//! # Toast 提示组件

use super::{UiContext, draw_rounded_rect};
use macroquad::prelude::*;

/// Toast 类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToastType {
    /// 普通信息
    Info,
    /// 成功
    Success,
    /// 警告
    Warning,
    /// 错误
    Error,
}

/// 单个 Toast 消息
#[derive(Debug, Clone)]
pub struct Toast {
    /// 消息内容
    pub message: String,
    /// 类型
    pub toast_type: ToastType,
    /// 剩余显示时间
    pub remaining_time: f32,
    /// 淡出进度 (0.0 - 1.0)
    pub fade_progress: f32,
}

impl Toast {
    pub fn new(message: impl Into<String>, toast_type: ToastType, duration: f32) -> Self {
        Self {
            message: message.into(),
            toast_type,
            remaining_time: duration,
            fade_progress: 0.0,
        }
    }

    /// 更新状态，返回是否应该移除
    pub fn update(&mut self, dt: f32) -> bool {
        self.remaining_time -= dt;

        // 最后 0.3 秒开始淡出
        if self.remaining_time <= 0.3 {
            self.fade_progress = 1.0 - (self.remaining_time / 0.3).max(0.0);
        }

        self.remaining_time <= 0.0
    }
}

/// Toast 管理器
pub struct ToastManager {
    /// 活跃的 Toast 列表
    toasts: Vec<Toast>,
    /// 默认显示时间
    default_duration: f32,
}

impl Default for ToastManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ToastManager {
    pub fn new() -> Self {
        Self {
            toasts: Vec::new(),
            default_duration: 2.5,
        }
    }

    /// 显示普通消息
    pub fn info(&mut self, message: impl Into<String>) {
        self.show(message, ToastType::Info);
    }

    /// 显示成功消息
    pub fn success(&mut self, message: impl Into<String>) {
        self.show(message, ToastType::Success);
    }

    /// 显示警告消息
    pub fn warning(&mut self, message: impl Into<String>) {
        self.show(message, ToastType::Warning);
    }

    /// 显示错误消息
    pub fn error(&mut self, message: impl Into<String>) {
        self.show(message, ToastType::Error);
    }

    /// 显示自定义类型消息
    pub fn show(&mut self, message: impl Into<String>, toast_type: ToastType) {
        self.toasts
            .push(Toast::new(message, toast_type, self.default_duration));
    }

    /// 更新所有 Toast
    pub fn update(&mut self, dt: f32) {
        self.toasts.retain_mut(|toast| !toast.update(dt));
    }

    /// 绘制所有 Toast
    pub fn draw(&self, ctx: &UiContext, text_renderer: &crate::renderer::TextRenderer) {
        let theme = &ctx.theme;
        let toast_height = 50.0;
        let toast_width = 300.0;
        let margin = theme.spacing;
        let start_y = theme.spacing_large;

        for (i, toast) in self.toasts.iter().enumerate() {
            let y = start_y + i as f32 * (toast_height + margin);
            let x = ctx.screen_width - toast_width - margin;

            // 根据类型选择颜色
            let base_color = match toast.toast_type {
                ToastType::Info => theme.bg_secondary,
                ToastType::Success => theme.success,
                ToastType::Warning => theme.warning,
                ToastType::Error => theme.danger,
            };

            // 应用淡出透明度
            let alpha = 1.0 - toast.fade_progress;
            let bg_color = Color::new(base_color.r, base_color.g, base_color.b, 0.9 * alpha);
            let text_color = Color::new(
                theme.text_primary.r,
                theme.text_primary.g,
                theme.text_primary.b,
                alpha,
            );

            // 绘制背景
            draw_rounded_rect(
                x,
                y,
                toast_width,
                toast_height,
                theme.corner_radius,
                bg_color,
            );

            // 绘制图标（简化为文字）
            let icon = match toast.toast_type {
                ToastType::Info => "ℹ",
                ToastType::Success => "✓",
                ToastType::Warning => "⚠",
                ToastType::Error => "✗",
            };
            text_renderer.draw_ui_text(
                icon,
                x + theme.spacing,
                y + toast_height / 2.0 + theme.font_size_normal * 0.3,
                theme.font_size_normal,
                text_color,
            );

            // 绘制消息
            text_renderer.draw_ui_text(
                &toast.message,
                x + theme.spacing * 2.5,
                y + toast_height / 2.0 + theme.font_size_small * 0.3,
                theme.font_size_small,
                text_color,
            );
        }
    }

    /// 是否有活跃的 Toast
    pub fn has_toasts(&self) -> bool {
        !self.toasts.is_empty()
    }

    /// 清空所有 Toast
    pub fn clear(&mut self) {
        self.toasts.clear();
    }
}
