//! # 主标题界面

use macroquad::prelude::*;
use crate::ui::{UiContext, Button, ButtonStyle, Theme, draw_rounded_rect};
use crate::app_mode::{AppMode, NavigationStack, SaveLoadTab};
use crate::save_manager::SaveManager;
use crate::renderer::TextRenderer;

/// 主菜单项
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TitleAction {
    None,
    StartGame,
    Continue,
    LoadGame,
    Settings,
    Exit,
}

/// 主标题界面
pub struct TitleScreen {
    /// 按钮列表
    buttons: Vec<(TitleAction, Button)>,
    /// 是否有存档可继续
    has_save: bool,
    /// 最新存档槽位
    latest_slot: Option<u32>,
    /// 是否需要重新初始化
    needs_init: bool,
}

impl TitleScreen {
    pub fn new() -> Self {
        Self {
            buttons: Vec::new(),
            has_save: false,
            latest_slot: None,
            needs_init: true,
        }
    }

    /// 初始化界面（检查存档状态等）
    pub fn init(&mut self, save_manager: &SaveManager, theme: &Theme, screen_width: f32, screen_height: f32) {
        // 检查是否有存档
        let saves = save_manager.list_saves();
        self.has_save = !saves.is_empty();
        
        // 找到最新的存档（按槽位号，实际应该按时间戳）
        self.latest_slot = saves.iter().map(|(slot, _)| *slot).max();

        // 创建按钮
        self.buttons.clear();
        
        let button_width = theme.button_min_width;
        let button_height = theme.button_height;
        let spacing = theme.spacing;
        let start_y = screen_height * 0.45;
        let center_x = (screen_width - button_width) / 2.0;

        let mut y = start_y;
        
        // 开始游戏
        let mut start_btn = Button::new("开始游戏", center_x, y, button_width, button_height);
        start_btn.style = ButtonStyle::Primary;
        self.buttons.push((TitleAction::StartGame, start_btn));
        y += button_height + spacing;

        // 继续（如果有存档）
        let mut continue_btn = Button::new("继续", center_x, y, button_width, button_height);
        continue_btn.disabled = !self.has_save;
        self.buttons.push((TitleAction::Continue, continue_btn));
        y += button_height + spacing;

        // 读取存档
        self.buttons.push((TitleAction::LoadGame, Button::new("读取存档", center_x, y, button_width, button_height)));
        y += button_height + spacing;

        // 设置
        self.buttons.push((TitleAction::Settings, Button::new("设置", center_x, y, button_width, button_height)));
        y += button_height + spacing;

        // 退出
        self.buttons.push((TitleAction::Exit, Button::new("退出游戏", center_x, y, button_width, button_height)));

        self.needs_init = false;
    }

    /// 更新界面，返回用户操作
    pub fn update(&mut self, ctx: &UiContext) -> TitleAction {
        for (action, button) in &mut self.buttons {
            if button.update(ctx) {
                return *action;
            }
        }
        TitleAction::None
    }

    /// 绘制界面
    pub fn draw(&self, ctx: &UiContext, text_renderer: &TextRenderer) {
        let theme = &ctx.theme;

        // 绘制背景
        clear_background(theme.bg_primary);

        // 绘制装饰性背景（渐变效果）
        for i in 0..10 {
            let alpha = 0.02 * (10 - i) as f32;
            draw_rectangle(
                0.0, 
                ctx.screen_height * 0.3 + i as f32 * 20.0,
                ctx.screen_width,
                ctx.screen_height * 0.7 - i as f32 * 20.0,
                Color::new(theme.accent.r, theme.accent.g, theme.accent.b, alpha)
            );
        }

        // 绘制标题
        let title = "Visual Novel Engine";
        let title_size = theme.font_size_title;
        // 简单估算居中位置
        let title_width = title.len() as f32 * title_size * 0.5;
        let title_x = (ctx.screen_width - title_width) / 2.0;
        let title_y = ctx.screen_height * 0.25;

        // 标题阴影
        text_renderer.draw_ui_text(
            title,
            title_x + 3.0,
            title_y + 3.0,
            title_size,
            Color::new(0.0, 0.0, 0.0, 0.5)
        );
        // 标题
        text_renderer.draw_ui_text(
            title,
            title_x,
            title_y,
            title_size,
            theme.accent
        );

        // 副标题
        let subtitle = "Rust + macroquad";
        let subtitle_size = theme.font_size_normal;
        let subtitle_width = subtitle.len() as f32 * subtitle_size * 0.5;
        text_renderer.draw_ui_text(
            subtitle,
            (ctx.screen_width - subtitle_width) / 2.0,
            title_y + title_size + theme.spacing_small,
            subtitle_size,
            theme.text_secondary
        );

        // 绘制按钮
        for (_, button) in &self.buttons {
            button.draw(ctx, text_renderer);
        }

        // 底部版本信息
        let version = "v0.1.0";
        text_renderer.draw_ui_text(
            version,
            theme.spacing,
            ctx.screen_height - theme.spacing,
            theme.font_size_small,
            theme.text_disabled
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

    /// 获取最新存档槽位
    pub fn latest_slot(&self) -> Option<u32> {
        self.latest_slot
    }
}

impl Default for TitleScreen {
    fn default() -> Self {
        Self::new()
    }
}
