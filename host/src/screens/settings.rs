//! # 设置界面

use crate::app_mode::UserSettings;
use crate::renderer::TextRenderer;
use crate::ui::{Button, ButtonStyle, Panel, Slider, Toggle, UiContext};
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
    bgm_slider: Option<Slider>,
    sfx_slider: Option<Slider>,
    text_speed_slider: Option<Slider>,
    muted_toggle: Option<Toggle>,
    fullscreen_toggle: Option<Toggle>,
}

impl SettingsScreen {
    pub fn new() -> Self {
        Self {
            settings: UserSettings::default(),
            original_settings: UserSettings::default(),
            back_button: None,
            apply_button: None,
            needs_init: true,
            bgm_slider: None,
            sfx_slider: None,
            text_speed_slider: None,
            muted_toggle: None,
            fullscreen_toggle: None,
        }
    }

    fn panel_layout(ctx: &UiContext) -> (f32, f32, f32, f32) {
        let panel_width = 500.0;
        let panel_height = 400.0;
        let panel_x = (ctx.screen_width - panel_width) / 2.0;
        let panel_y = (ctx.screen_height - panel_height) / 2.0;
        (panel_x, panel_y, panel_width, panel_height)
    }

    fn sync_layout(&mut self, ctx: &UiContext) {
        let theme = &ctx.theme;
        let (panel_x, panel_y, panel_width, panel_height) = Self::panel_layout(ctx);
        let content_x = panel_x + theme.padding;
        let content_y = panel_y + theme.font_size_large + theme.padding * 2.0;
        let slider_width = panel_width - theme.padding * 2.0 - 150.0;

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

        let slider_h = theme.tokens.control.toggle_height;
        let toggle_h = theme.tokens.control.toggle_height;
        let toggle_w = 100.0;

        if let Some(s) = &mut self.bgm_slider {
            s.set_rect(Rect::new(
                content_x + 150.0,
                content_y,
                slider_width,
                slider_h,
            ));
        }
        if let Some(s) = &mut self.sfx_slider {
            s.set_rect(Rect::new(
                content_x + 150.0,
                content_y + 50.0,
                slider_width,
                slider_h,
            ));
        }
        if let Some(s) = &mut self.text_speed_slider {
            s.set_rect(Rect::new(
                content_x + 150.0,
                content_y + 100.0,
                slider_width,
                slider_h,
            ));
        }
        if let Some(t) = &mut self.muted_toggle {
            t.set_rect(Rect::new(
                content_x + 150.0,
                content_y + 150.0,
                toggle_w,
                toggle_h,
            ));
        }
        if let Some(t) = &mut self.fullscreen_toggle {
            t.set_rect(Rect::new(
                content_x + 150.0,
                content_y + 200.0,
                toggle_w,
                toggle_h,
            ));
        }
    }

    /// 初始化界面
    pub fn init(&mut self, ctx: &UiContext, settings: &UserSettings) {
        self.settings = settings.clone();
        self.original_settings = settings.clone();

        self.bgm_slider = Some(Slider::new(
            Rect::new(0.0, 0.0, 0.0, 0.0),
            0.0,
            1.0,
            settings.bgm_volume,
        ));
        self.sfx_slider = Some(Slider::new(
            Rect::new(0.0, 0.0, 0.0, 0.0),
            0.0,
            1.0,
            settings.sfx_volume,
        ));
        self.text_speed_slider = Some(Slider::new(
            Rect::new(0.0, 0.0, 0.0, 0.0),
            10.0,
            100.0,
            settings.text_speed,
        ));
        self.muted_toggle = Some(Toggle::new(Rect::new(0.0, 0.0, 0.0, 0.0), settings.muted));
        self.fullscreen_toggle = Some(Toggle::new(
            Rect::new(0.0, 0.0, 0.0, 0.0),
            settings.fullscreen,
        ));

        self.sync_layout(ctx);

        self.needs_init = false;
    }

    /// 获取当前设置
    pub fn settings(&self) -> &UserSettings {
        &self.settings
    }

    /// 更新界面
    pub fn update(&mut self, ctx: &UiContext) -> SettingsAction {
        self.sync_layout(ctx);

        // ESC 返回
        if is_key_pressed(KeyCode::Escape) {
            self.settings = self.original_settings.clone();
            return SettingsAction::Back;
        }

        if let Some(s) = &mut self.bgm_slider
            && s.update(ctx)
        {
            self.settings.bgm_volume = s.value;
        }
        if let Some(s) = &mut self.sfx_slider
            && s.update(ctx)
        {
            self.settings.sfx_volume = s.value;
        }
        if let Some(s) = &mut self.text_speed_slider
            && s.update(ctx)
        {
            self.settings.text_speed = s.value;
        }
        if let Some(t) = &mut self.muted_toggle
            && t.update(ctx)
        {
            self.settings.muted = t.value;
        }
        if let Some(t) = &mut self.fullscreen_toggle
            && t.update(ctx)
        {
            self.settings.fullscreen = t.value;
        }

        // 返回按钮
        if let Some(ref mut btn) = self.back_button
            && btn.update(ctx)
        {
            self.settings = self.original_settings.clone();
            return SettingsAction::Back;
        }

        // 应用按钮
        if let Some(ref mut btn) = self.apply_button
            && btn.update(ctx)
        {
            return SettingsAction::Apply;
        }

        SettingsAction::None
    }

    /// 绘制界面
    pub fn draw(&self, ctx: &UiContext, text_renderer: &TextRenderer) {
        let theme = &ctx.theme;
        let (panel_x, panel_y, panel_width, panel_height) = Self::panel_layout(ctx);

        // 背景
        draw_rectangle(
            0.0,
            0.0,
            ctx.screen_width,
            ctx.screen_height,
            theme.bg_primary,
        );

        // 面板
        let panel = Panel::centered(
            panel_width,
            panel_height,
            ctx.screen_width,
            ctx.screen_height,
        )
        .with_title("设置");
        panel.draw(ctx, text_renderer);

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
        if let Some(s) = &self.bgm_slider {
            s.draw(ctx);
        }

        // SFX 音量
        text_renderer.draw_ui_text(
            "音效音量",
            content_x,
            content_y + 70.0,
            theme.font_size_normal,
            theme.text_primary,
        );
        if let Some(s) = &self.sfx_slider {
            s.draw(ctx);
        }

        // 文字速度
        text_renderer.draw_ui_text(
            "文字速度",
            content_x,
            content_y + 120.0,
            theme.font_size_normal,
            theme.text_primary,
        );
        if let Some(s) = &self.text_speed_slider {
            s.draw(ctx);
        }
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
        if let Some(t) = &self.muted_toggle {
            t.draw(ctx);
        }

        // 全屏
        text_renderer.draw_ui_text(
            "全屏",
            content_x,
            content_y + 220.0,
            theme.font_size_normal,
            theme.text_primary,
        );
        if let Some(t) = &self.fullscreen_toggle {
            t.draw(ctx);
        }

        // 按钮
        if let Some(ref btn) = self.back_button {
            btn.draw(ctx, text_renderer);
        }
        if let Some(ref btn) = self.apply_button {
            btn.draw(ctx, text_renderer);
        }
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
