//! # 设置界面

use crate::app_mode::UserSettings;
use crate::renderer::TextRenderer;
use crate::ui::{Button, ButtonStyle, Panel, UiContext, draw_rounded_rect};
use macroquad::prelude::*;

/// 设置界面操作
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsAction {
    None,
    Back,
    Apply,
}

/// 设置界面
pub struct SettingsScreen {
    /// 当前编辑的设置
    settings: UserSettings,
    /// 原始设置（用于取消时恢复）
    original_settings: UserSettings,
    /// 返回按钮
    back_button: Option<Button>,
    /// 应用按钮
    apply_button: Option<Button>,
    /// 是否需要重新初始化
    needs_init: bool,
    /// 滑块状态
    dragging_slider: Option<&'static str>,
}

impl SettingsScreen {
    pub fn new() -> Self {
        Self {
            settings: UserSettings::default(),
            original_settings: UserSettings::default(),
            back_button: None,
            apply_button: None,
            needs_init: true,
            dragging_slider: None,
        }
    }

    /// 初始化界面
    pub fn init(&mut self, ctx: &UiContext, settings: &UserSettings) {
        let theme = &ctx.theme;

        self.settings = settings.clone();
        self.original_settings = settings.clone();

        // 面板布局
        let panel_width = 500.0;
        let panel_height = 400.0;
        let panel_x = (ctx.screen_width - panel_width) / 2.0;
        let panel_y = (ctx.screen_height - panel_height) / 2.0;

        // 按钮
        let btn_y = panel_y + panel_height - theme.padding - theme.button_height;
        self.back_button = Some(Button::new(
            "返回",
            panel_x + theme.padding,
            btn_y,
            100.0,
            theme.button_height,
        ));

        let mut apply_btn = Button::new(
            "应用",
            panel_x + panel_width - theme.padding - 100.0,
            btn_y,
            100.0,
            theme.button_height,
        );
        apply_btn.style = ButtonStyle::Primary;
        self.apply_button = Some(apply_btn);

        self.needs_init = false;
    }

    /// 获取当前设置
    pub fn settings(&self) -> &UserSettings {
        &self.settings
    }

    /// 更新界面
    pub fn update(&mut self, ctx: &UiContext) -> SettingsAction {
        let theme = &ctx.theme;

        // ESC 返回
        if is_key_pressed(KeyCode::Escape) {
            self.settings = self.original_settings.clone();
            return SettingsAction::Back;
        }

        // 面板布局
        let panel_width = 500.0;
        let panel_height = 400.0;
        let panel_x = (ctx.screen_width - panel_width) / 2.0;
        let panel_y = (ctx.screen_height - panel_height) / 2.0;

        let content_x = panel_x + theme.padding;
        let content_y = panel_y + theme.font_size_large + theme.padding * 2.0;
        let slider_width = panel_width - theme.padding * 2.0 - 150.0;

        // 处理滑块拖动
        if ctx.mouse_just_released {
            self.dragging_slider = None;
        }

        // BGM 音量滑块
        let bgm_slider_rect = Rect::new(content_x + 150.0, content_y, slider_width, 30.0);
        if ctx.mouse_in_rect(bgm_slider_rect) && ctx.mouse_just_pressed {
            self.dragging_slider = Some("bgm");
        }
        if self.dragging_slider == Some("bgm") {
            let relative_x = (ctx.mouse_pos.x - bgm_slider_rect.x) / bgm_slider_rect.w;
            self.settings.bgm_volume = relative_x.clamp(0.0, 1.0);
        }

        // SFX 音量滑块
        let sfx_slider_rect = Rect::new(content_x + 150.0, content_y + 50.0, slider_width, 30.0);
        if ctx.mouse_in_rect(sfx_slider_rect) && ctx.mouse_just_pressed {
            self.dragging_slider = Some("sfx");
        }
        if self.dragging_slider == Some("sfx") {
            let relative_x = (ctx.mouse_pos.x - sfx_slider_rect.x) / sfx_slider_rect.w;
            self.settings.sfx_volume = relative_x.clamp(0.0, 1.0);
        }

        // 文字速度滑块
        let text_speed_rect = Rect::new(content_x + 150.0, content_y + 100.0, slider_width, 30.0);
        if ctx.mouse_in_rect(text_speed_rect) && ctx.mouse_just_pressed {
            self.dragging_slider = Some("text_speed");
        }
        if self.dragging_slider == Some("text_speed") {
            let relative_x = (ctx.mouse_pos.x - text_speed_rect.x) / text_speed_rect.w;
            // 文字速度范围：10 - 100 字/秒
            self.settings.text_speed = 10.0 + relative_x.clamp(0.0, 1.0) * 90.0;
        }

        // 静音按钮
        let mute_btn_rect = Rect::new(content_x + 150.0, content_y + 150.0, 100.0, 35.0);
        if ctx.mouse_in_rect(mute_btn_rect) && ctx.mouse_just_released {
            self.settings.muted = !self.settings.muted;
        }

        // 全屏按钮
        let fullscreen_btn_rect = Rect::new(content_x + 150.0, content_y + 200.0, 100.0, 35.0);
        if ctx.mouse_in_rect(fullscreen_btn_rect) && ctx.mouse_just_released {
            self.settings.fullscreen = !self.settings.fullscreen;
        }

        // 返回按钮
        if let Some(ref mut btn) = self.back_button {
            if btn.update(ctx) {
                self.settings = self.original_settings.clone();
                return SettingsAction::Back;
            }
        }

        // 应用按钮
        if let Some(ref mut btn) = self.apply_button {
            if btn.update(ctx) {
                return SettingsAction::Apply;
            }
        }

        SettingsAction::None
    }

    /// 绘制界面
    pub fn draw(&self, ctx: &UiContext, text_renderer: &TextRenderer) {
        let theme = &ctx.theme;

        // 背景
        draw_rectangle(
            0.0,
            0.0,
            ctx.screen_width,
            ctx.screen_height,
            theme.bg_primary,
        );

        // 面板
        let panel_width = 500.0;
        let panel_height = 400.0;
        let panel = Panel::centered(
            panel_width,
            panel_height,
            ctx.screen_width,
            ctx.screen_height,
        )
        .with_title("设置");
        panel.draw(ctx, text_renderer);

        let panel_x = (ctx.screen_width - panel_width) / 2.0;
        let panel_y = (ctx.screen_height - panel_height) / 2.0;
        let content_x = panel_x + theme.padding;
        let content_y = panel_y + theme.font_size_large + theme.padding * 2.0;
        let slider_width = panel_width - theme.padding * 2.0 - 150.0;

        // BGM 音量
        text_renderer.draw_ui_text(
            "BGM 音量",
            content_x,
            content_y + 20.0,
            theme.font_size_normal,
            theme.text_primary,
        );
        self.draw_slider(
            content_x + 150.0,
            content_y,
            slider_width,
            30.0,
            self.settings.bgm_volume,
            theme,
        );

        // SFX 音量
        text_renderer.draw_ui_text(
            "音效音量",
            content_x,
            content_y + 70.0,
            theme.font_size_normal,
            theme.text_primary,
        );
        self.draw_slider(
            content_x + 150.0,
            content_y + 50.0,
            slider_width,
            30.0,
            self.settings.sfx_volume,
            theme,
        );

        // 文字速度
        text_renderer.draw_ui_text(
            "文字速度",
            content_x,
            content_y + 120.0,
            theme.font_size_normal,
            theme.text_primary,
        );
        let text_speed_value = (self.settings.text_speed - 10.0) / 90.0;
        self.draw_slider(
            content_x + 150.0,
            content_y + 100.0,
            slider_width,
            30.0,
            text_speed_value,
            theme,
        );
        text_renderer.draw_ui_text(
            &format!("{:.0} 字/秒", self.settings.text_speed),
            content_x + 150.0 + slider_width + 10.0,
            content_y + 120.0,
            theme.font_size_small,
            theme.text_secondary,
        );

        // 静音
        text_renderer.draw_ui_text(
            "静音",
            content_x,
            content_y + 170.0,
            theme.font_size_normal,
            theme.text_primary,
        );
        self.draw_toggle(
            content_x + 150.0,
            content_y + 150.0,
            100.0,
            35.0,
            self.settings.muted,
            theme,
        );

        // 全屏
        text_renderer.draw_ui_text(
            "全屏",
            content_x,
            content_y + 220.0,
            theme.font_size_normal,
            theme.text_primary,
        );
        self.draw_toggle(
            content_x + 150.0,
            content_y + 200.0,
            100.0,
            35.0,
            self.settings.fullscreen,
            theme,
        );

        // 按钮
        if let Some(ref btn) = self.back_button {
            btn.draw(ctx, text_renderer);
        }
        if let Some(ref btn) = self.apply_button {
            btn.draw(ctx, text_renderer);
        }
    }

    /// 绘制滑块
    fn draw_slider(&self, x: f32, y: f32, w: f32, h: f32, value: f32, theme: &crate::ui::Theme) {
        // 背景
        draw_rounded_rect(x, y + h / 2.0 - 4.0, w, 8.0, 4.0, theme.bg_secondary);
        // 填充
        let fill_w = w * value;
        draw_rounded_rect(x, y + h / 2.0 - 4.0, fill_w, 8.0, 4.0, theme.accent);
        // 滑块
        let knob_x = x + fill_w - 8.0;
        draw_circle(knob_x.max(x) + 8.0, y + h / 2.0, 10.0, theme.text_primary);
    }

    /// 绘制开关
    fn draw_toggle(&self, x: f32, y: f32, w: f32, h: f32, value: bool, theme: &crate::ui::Theme) {
        let bg_color = if value {
            theme.accent
        } else {
            theme.bg_secondary
        };
        draw_rounded_rect(x, y, w, h, h / 2.0, bg_color);

        let knob_x = if value { x + w - h + 4.0 } else { x + 4.0 };
        draw_circle(
            knob_x + (h - 8.0) / 2.0,
            y + h / 2.0,
            (h - 8.0) / 2.0,
            theme.text_primary,
        );
    }

    /// 标记需要重新初始化
    pub fn mark_needs_init(&mut self) {
        self.needs_init = true;
    }

    /// 是否需要初始化
    pub fn needs_init(&self) -> bool {
        self.needs_init
    }
}

impl Default for SettingsScreen {
    fn default() -> Self {
        Self::new()
    }
}
