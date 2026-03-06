//! # 历史回看界面

use crate::renderer::TextRenderer;
use crate::ui::{Button, ListItem, ListView, Panel, UiContext};
use macroquad::prelude::*;
use vn_runtime::history::{History, HistoryEvent};

/// 历史界面操作
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HistoryAction {
    None,
    Back,
}

/// 历史回看界面
pub struct HistoryScreen {
    /// 历史列表
    history_list: ListView,
    /// 返回按钮
    back_button: Option<Button>,
    /// 是否需要重新初始化
    needs_init: bool,
}

impl HistoryScreen {
    pub fn new() -> Self {
        Self {
            history_list: ListView::new(Rect::new(0.0, 0.0, 0.0, 0.0), 80.0),
            back_button: None,
            needs_init: true,
        }
    }

    /// 初始化界面
    pub fn init(&mut self, ctx: &UiContext, history: &History) {
        let theme = &ctx.theme;

        // 面板布局
        let panel_width = 700.0;
        let panel_height = 550.0;
        let panel_x = (ctx.screen_width - panel_width) / 2.0;
        let panel_y = (ctx.screen_height - panel_height) / 2.0;

        // 列表区域
        let list_x = panel_x + theme.padding;
        let list_y = panel_y + theme.font_size_large + theme.padding * 2.0;
        let list_width = panel_width - theme.padding * 2.0;
        let list_height =
            panel_height - (list_y - panel_y) - theme.padding - theme.button_height - theme.spacing;
        self.history_list = ListView::new(
            Rect::new(list_x, list_y, list_width, list_height),
            theme.tokens.control.list_item_height + 10.0,
        );

        // 转换历史事件为列表项
        let items: Vec<ListItem> = history
            .events()
            .iter()
            .rev()
            .enumerate()
            .map(|(i, event)| {
                let (text, secondary) = match event {
                    HistoryEvent::Dialogue {
                        speaker, content, ..
                    } => {
                        let speaker_text = speaker.as_deref().unwrap_or("旁白");
                        (format!("【{}】", speaker_text), content.clone())
                    }
                    HistoryEvent::ChapterMark { title, .. } => {
                        (format!("📖 {}", title), String::new())
                    }
                    HistoryEvent::ChoiceMade {
                        options,
                        selected_index,
                        ..
                    } => {
                        let selected = options
                            .get(*selected_index)
                            .map(|s| s.as_str())
                            .unwrap_or("???");
                        (
                            format!("🔀 选择：{}", selected),
                            format!("从 {} 个选项中选择", options.len()),
                        )
                    }
                    HistoryEvent::Jump { label, .. } => {
                        ("➡️ 跳转".to_string(), format!("→ {}", label))
                    }
                    HistoryEvent::BackgroundChange { path, .. } => {
                        ("🖼️ 背景切换".to_string(), path.clone())
                    }
                    HistoryEvent::BgmChange { path, .. } => {
                        let path_text = path.as_deref().unwrap_or("停止");
                        ("🎵 BGM".to_string(), path_text.to_string())
                    }
                };

                let mut item = ListItem::new(text, i.to_string());
                if !secondary.is_empty() {
                    item = item.with_secondary(secondary);
                }
                item
            })
            .collect();

        self.history_list.set_items(items);

        // 返回按钮
        let btn_y = panel_y + panel_height - theme.padding - theme.button_height;
        self.back_button = Some(Button::new(
            "返回",
            panel_x + theme.padding,
            btn_y,
            100.0,
            theme.button_height,
        ));

        self.needs_init = false;
    }

    /// 更新界面
    pub fn update(&mut self, ctx: &UiContext) -> HistoryAction {
        // ESC 返回
        if is_key_pressed(KeyCode::Escape) {
            return HistoryAction::Back;
        }

        // 列表更新（滚动等）
        self.history_list.update(ctx);

        // 键盘导航
        if is_key_pressed(KeyCode::Up) {
            self.history_list.select_prev();
        }
        if is_key_pressed(KeyCode::Down) {
            self.history_list.select_next();
        }

        // 返回按钮
        if let Some(ref mut btn) = self.back_button
            && btn.update(ctx)
        {
            return HistoryAction::Back;
        }

        HistoryAction::None
    }

    /// 绘制界面（覆盖在游戏画面上）
    pub fn draw(&self, ctx: &UiContext, text_renderer: &TextRenderer) {
        let theme = &ctx.theme;

        // 半透明覆盖层
        draw_rectangle(
            0.0,
            0.0,
            ctx.screen_width,
            ctx.screen_height,
            theme.bg_overlay,
        );

        // 面板
        let panel_width = 700.0;
        let panel_height = 550.0;
        let panel = Panel::centered(
            panel_width,
            panel_height,
            ctx.screen_width,
            ctx.screen_height,
        )
        .with_title("历史记录");
        panel.draw(ctx, text_renderer);

        // 列表
        self.history_list.draw(ctx, text_renderer);

        // 返回按钮
        if let Some(ref btn) = self.back_button {
            btn.draw(ctx, text_renderer);
        }

        // 提示
        text_renderer.draw_ui_text(
            "使用 ↑↓ 或鼠标滚轮浏览",
            (ctx.screen_width - 200.0) / 2.0,
            ctx.screen_height - theme.spacing * 2.0,
            theme.font_size_small,
            theme.text_disabled,
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

impl Default for HistoryScreen {
    fn default() -> Self {
        Self::new()
    }
}
