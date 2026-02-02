//! # UI 组件模块
//!
//! 提供统一的 UI 组件库，用于主菜单、存档界面、设置界面等。

pub mod theme;
pub mod button;
pub mod panel;
pub mod list;
pub mod modal;
pub mod toast;

pub use theme::Theme;
pub use button::{Button, ButtonState, ButtonStyle};
pub use panel::Panel;
pub use list::{ListItem, ListView};
pub use modal::{Modal, ModalResult};
pub use toast::{Toast, ToastManager, ToastType};

use macroquad::prelude::*;

/// UI 上下文，存储 UI 渲染所需的共享状态
pub struct UiContext {
    /// 当前主题
    pub theme: Theme,
    /// 屏幕宽度
    pub screen_width: f32,
    /// 屏幕高度
    pub screen_height: f32,
    /// 鼠标位置
    pub mouse_pos: Vec2,
    /// 鼠标是否按下
    pub mouse_pressed: bool,
    /// 鼠标是否刚按下（本帧）
    pub mouse_just_pressed: bool,
    /// 鼠标是否刚释放（本帧）
    pub mouse_just_released: bool,
}

impl UiContext {
    pub fn new(theme: Theme) -> Self {
        Self {
            theme,
            screen_width: screen_width(),
            screen_height: screen_height(),
            mouse_pos: Vec2::ZERO,
            mouse_pressed: false,
            mouse_just_pressed: false,
            mouse_just_released: false,
        }
    }

    /// 每帧更新状态
    pub fn update(&mut self) {
        self.screen_width = screen_width();
        self.screen_height = screen_height();
        self.mouse_pos = Vec2::new(mouse_position().0, mouse_position().1);
        self.mouse_just_pressed = is_mouse_button_pressed(MouseButton::Left);
        self.mouse_just_released = is_mouse_button_released(MouseButton::Left);
        self.mouse_pressed = is_mouse_button_down(MouseButton::Left);
    }

    /// 检查点是否在矩形内
    pub fn point_in_rect(&self, point: Vec2, rect: Rect) -> bool {
        point.x >= rect.x && point.x <= rect.x + rect.w &&
        point.y >= rect.y && point.y <= rect.y + rect.h
    }

    /// 检查鼠标是否在矩形内
    pub fn mouse_in_rect(&self, rect: Rect) -> bool {
        self.point_in_rect(self.mouse_pos, rect)
    }
}

/// 绘制圆角矩形（简化版，用四个圆角近似）
pub fn draw_rounded_rect(x: f32, y: f32, w: f32, h: f32, radius: f32, color: Color) {
    let r = radius.min(w / 2.0).min(h / 2.0);
    
    // 中心矩形
    draw_rectangle(x + r, y, w - 2.0 * r, h, color);
    // 左右矩形
    draw_rectangle(x, y + r, r, h - 2.0 * r, color);
    draw_rectangle(x + w - r, y + r, r, h - 2.0 * r, color);
    
    // 四个角（用圆形近似）
    draw_circle(x + r, y + r, r, color);
    draw_circle(x + w - r, y + r, r, color);
    draw_circle(x + r, y + h - r, r, color);
    draw_circle(x + w - r, y + h - r, r, color);
}

/// 绘制圆角矩形边框
pub fn draw_rounded_rect_lines(x: f32, y: f32, w: f32, h: f32, radius: f32, thickness: f32, color: Color) {
    let r = radius.min(w / 2.0).min(h / 2.0);
    
    // 上下边
    draw_line(x + r, y, x + w - r, y, thickness, color);
    draw_line(x + r, y + h, x + w - r, y + h, thickness, color);
    // 左右边
    draw_line(x, y + r, x, y + h - r, thickness, color);
    draw_line(x + w, y + r, x + w, y + h - r, thickness, color);
    
    // 四个角（用弧线，这里简化为直角连接点）
    // macroquad 没有直接的 arc 函数，用短线段近似
    let steps = 8;
    for i in 0..steps {
        let a1 = std::f32::consts::PI * 1.5 + (i as f32 / steps as f32) * std::f32::consts::FRAC_PI_2;
        let a2 = std::f32::consts::PI * 1.5 + ((i + 1) as f32 / steps as f32) * std::f32::consts::FRAC_PI_2;
        
        // 左上角
        draw_line(
            x + r + r * a1.cos(), y + r + r * a1.sin(),
            x + r + r * a2.cos(), y + r + r * a2.sin(),
            thickness, color
        );
        // 右上角
        draw_line(
            x + w - r + r * (a1 + std::f32::consts::FRAC_PI_2).cos(),
            y + r + r * (a1 + std::f32::consts::FRAC_PI_2).sin(),
            x + w - r + r * (a2 + std::f32::consts::FRAC_PI_2).cos(),
            y + r + r * (a2 + std::f32::consts::FRAC_PI_2).sin(),
            thickness, color
        );
        // 右下角
        draw_line(
            x + w - r + r * (a1 + std::f32::consts::PI).cos(),
            y + h - r + r * (a1 + std::f32::consts::PI).sin(),
            x + w - r + r * (a2 + std::f32::consts::PI).cos(),
            y + h - r + r * (a2 + std::f32::consts::PI).sin(),
            thickness, color
        );
        // 左下角
        draw_line(
            x + r + r * (a1 - std::f32::consts::FRAC_PI_2).cos(),
            y + h - r + r * (a1 - std::f32::consts::FRAC_PI_2).sin(),
            x + r + r * (a2 - std::f32::consts::FRAC_PI_2).cos(),
            y + h - r + r * (a2 - std::f32::consts::FRAC_PI_2).sin(),
            thickness, color
        );
    }
}
