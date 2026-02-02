//! # 游戏内系统菜单

use macroquad::prelude::*;
use crate::ui::{UiContext, Button, ButtonStyle, Panel, Modal, ModalResult, draw_rounded_rect};
use crate::renderer::TextRenderer;

/// 菜单操作
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InGameMenuAction {
    None,
    Resume,
    Save,
    Load,
    Settings,
    History,
    ReturnToTitle,
    Exit,
}

/// 游戏内系统菜单
pub struct InGameMenuScreen {
    /// 按钮列表
    buttons: Vec<(InGameMenuAction, Button)>,
    /// 确认对话框（返回标题/退出时）
    confirm_modal: Option<(InGameMenuAction, Modal)>,
    /// 是否需要重新初始化
    needs_init: bool,
}

impl InGameMenuScreen {
    pub fn new() -> Self {
        Self {
            buttons: Vec::new(),
            confirm_modal: None,
            needs_init: true,
        }
    }

    /// 初始化界面
    pub fn init(&mut self, ctx: &UiContext) {
        let theme = &ctx.theme;
        
        self.buttons.clear();
        self.confirm_modal = None;

        // 面板尺寸
        let panel_width = 300.0;
        let panel_height = 420.0;
        let panel_x = (ctx.screen_width - panel_width) / 2.0;
        let panel_y = (ctx.screen_height - panel_height) / 2.0;

        let button_width = panel_width - theme.padding * 2.0;
        let button_height = theme.button_height;
        let spacing = theme.spacing_small;
        
        let mut y = panel_y + theme.padding + theme.font_size_large + theme.spacing;

        // 继续游戏
        let mut resume_btn = Button::new("继续游戏", panel_x + theme.padding, y, button_width, button_height);
        resume_btn.style = ButtonStyle::Primary;
        self.buttons.push((InGameMenuAction::Resume, resume_btn));
        y += button_height + spacing;

        // 存档
        self.buttons.push((InGameMenuAction::Save, Button::new("存档", panel_x + theme.padding, y, button_width, button_height)));
        y += button_height + spacing;

        // 读档
        self.buttons.push((InGameMenuAction::Load, Button::new("读档", panel_x + theme.padding, y, button_width, button_height)));
        y += button_height + spacing;

        // 设置
        self.buttons.push((InGameMenuAction::Settings, Button::new("设置", panel_x + theme.padding, y, button_width, button_height)));
        y += button_height + spacing;

        // 历史
        self.buttons.push((InGameMenuAction::History, Button::new("历史记录", panel_x + theme.padding, y, button_width, button_height)));
        y += button_height + spacing * 2.0;

        // 返回标题
        self.buttons.push((InGameMenuAction::ReturnToTitle, Button::new("返回标题", panel_x + theme.padding, y, button_width, button_height)));
        y += button_height + spacing;

        // 退出
        let mut exit_btn = Button::new("退出游戏", panel_x + theme.padding, y, button_width, button_height);
        exit_btn.style = ButtonStyle::Danger;
        self.buttons.push((InGameMenuAction::Exit, exit_btn));

        self.needs_init = false;
    }

    /// 更新界面
    pub fn update(&mut self, ctx: &UiContext) -> InGameMenuAction {
        // 处理确认对话框
        if let Some((action, ref mut modal)) = self.confirm_modal {
            match modal.update(ctx) {
                ModalResult::Confirm => {
                    self.confirm_modal = None;
                    return action;
                }
                ModalResult::Cancel => {
                    self.confirm_modal = None;
                    return InGameMenuAction::None;
                }
                ModalResult::None => {
                    return InGameMenuAction::None;
                }
            }
        }

        // ESC 返回游戏
        if is_key_pressed(KeyCode::Escape) {
            return InGameMenuAction::Resume;
        }

        // 处理按钮
        for (action, button) in &mut self.buttons {
            if button.update(ctx) {
                match action {
                    InGameMenuAction::ReturnToTitle => {
                        // 显示确认对话框
                        let modal = Modal::confirm("返回标题", "确定要返回标题界面吗？\n当前进度将不会自动保存。")
                            .with_danger(true);
                        self.confirm_modal = Some((*action, modal));
                        return InGameMenuAction::None;
                    }
                    InGameMenuAction::Exit => {
                        let modal = Modal::confirm("退出游戏", "确定要退出游戏吗？\n当前进度将不会自动保存。")
                            .with_danger(true);
                        self.confirm_modal = Some((*action, modal));
                        return InGameMenuAction::None;
                    }
                    _ => return *action,
                }
            }
        }

        InGameMenuAction::None
    }

    /// 绘制界面
    pub fn draw(&self, ctx: &UiContext, text_renderer: &TextRenderer) {
        let theme = &ctx.theme;

        // 绘制半透明覆盖层
        draw_rectangle(0.0, 0.0, ctx.screen_width, ctx.screen_height, theme.bg_overlay);

        // 面板
        let panel_width = 300.0;
        let panel_height = 420.0;
        let panel = Panel::centered(panel_width, panel_height, ctx.screen_width, ctx.screen_height)
            .with_title("系统菜单");
        panel.draw(ctx, text_renderer);

        // 绘制按钮
        for (_, button) in &self.buttons {
            button.draw(ctx, text_renderer);
        }

        // 绘制确认对话框
        if let Some((_, ref modal)) = self.confirm_modal {
            modal.draw(ctx, text_renderer);
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

impl Default for InGameMenuScreen {
    fn default() -> Self {
        Self::new()
    }
}
