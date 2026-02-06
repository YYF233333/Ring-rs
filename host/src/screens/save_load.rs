//! # 存档/读档界面

use crate::app_mode::SaveLoadTab;
use crate::renderer::TextRenderer;
use crate::save_manager::{MAX_SAVE_SLOTS, SaveInfo, SaveManager};
use crate::ui::{Button, ButtonStyle, ListItem, ListView, Modal, ModalResult, Panel, UiContext};
use macroquad::prelude::*;
use std::path::PathBuf;

/// 存档界面操作
#[derive(Debug, Clone)]
pub enum SaveLoadAction {
    None,
    Back,
    Save(u32),
    Load(u32),
    Delete(u32),
}

/// 存档/读档界面
pub struct SaveLoadScreen {
    /// 当前标签页
    pub tab: SaveLoadTab,
    /// 标签按钮
    tab_buttons: Vec<(SaveLoadTab, Button)>,
    /// 存档列表
    save_list: ListView,
    /// 操作按钮（保存/读取/删除）
    action_buttons: Vec<(String, Button)>,
    /// 返回按钮
    back_button: Option<Button>,
    /// 确认对话框
    confirm_modal: Option<(SaveLoadAction, Modal)>,
    /// 存档数据缓存 (slot, path)
    saves_cache: Vec<(u32, PathBuf)>,
    /// 是否需要重新初始化
    needs_init: bool,
    /// 是否需要刷新存档列表
    needs_refresh: bool,
}

impl SaveLoadScreen {
    pub fn new() -> Self {
        Self {
            tab: SaveLoadTab::Load,
            tab_buttons: Vec::new(),
            save_list: ListView::new(Rect::new(0.0, 0.0, 0.0, 0.0), 70.0),
            action_buttons: Vec::new(),
            back_button: None,
            confirm_modal: None,
            saves_cache: Vec::new(),
            needs_init: true,
            needs_refresh: true,
        }
    }

    /// 设置初始标签页
    pub fn with_tab(mut self, tab: SaveLoadTab) -> Self {
        self.tab = tab;
        self
    }

    /// 初始化界面
    pub fn init(&mut self, ctx: &UiContext, save_manager: &SaveManager) {
        let theme = &ctx.theme;

        // 面板布局
        let panel_width = 600.0;
        let panel_height = 500.0;
        let panel_x = (ctx.screen_width - panel_width) / 2.0;
        let panel_y = (ctx.screen_height - panel_height) / 2.0;

        // 标签按钮
        self.tab_buttons.clear();
        let tab_width = 100.0;
        let tab_height = 40.0;
        let tab_y = panel_y + theme.padding;

        let mut save_tab = Button::new(
            "存档",
            panel_x + theme.padding,
            tab_y,
            tab_width,
            tab_height,
        );
        save_tab.style = if self.tab == SaveLoadTab::Save {
            ButtonStyle::Primary
        } else {
            ButtonStyle::Secondary
        };
        self.tab_buttons.push((SaveLoadTab::Save, save_tab));

        let mut load_tab = Button::new(
            "读档",
            panel_x + theme.padding + tab_width + theme.spacing_small,
            tab_y,
            tab_width,
            tab_height,
        );
        load_tab.style = if self.tab == SaveLoadTab::Load {
            ButtonStyle::Primary
        } else {
            ButtonStyle::Secondary
        };
        self.tab_buttons.push((SaveLoadTab::Load, load_tab));

        // 列表区域
        let list_x = panel_x + theme.padding;
        let list_y = tab_y + tab_height + theme.spacing;
        let list_width = panel_width - theme.padding * 2.0 - 120.0; // 留出右侧按钮空间
        let list_height =
            panel_height - (list_y - panel_y) - theme.padding - theme.button_height - theme.spacing;
        self.save_list = ListView::new(Rect::new(list_x, list_y, list_width, list_height), 70.0);

        // 操作按钮（右侧）
        self.action_buttons.clear();
        let btn_x = list_x + list_width + theme.spacing_small;
        let btn_width = 100.0;
        let btn_height = theme.button_height;

        let action_text = match self.tab {
            SaveLoadTab::Save => "保存",
            SaveLoadTab::Load => "读取",
        };
        self.action_buttons.push((
            action_text.to_string(),
            Button::new(action_text, btn_x, list_y, btn_width, btn_height),
        ));

        let mut del_btn = Button::new(
            "删除",
            btn_x,
            list_y + btn_height + theme.spacing_small,
            btn_width,
            btn_height,
        );
        del_btn.style = ButtonStyle::Danger;
        self.action_buttons.push(("delete".to_string(), del_btn));

        // 返回按钮
        self.back_button = Some(Button::new(
            "返回",
            panel_x + theme.padding,
            panel_y + panel_height - theme.padding - theme.button_height,
            100.0,
            theme.button_height,
        ));

        // 刷新存档列表
        self.refresh_saves(save_manager);

        self.needs_init = false;
    }

    /// 刷新存档列表
    pub fn refresh_saves(&mut self, save_manager: &SaveManager) {
        self.saves_cache = save_manager.list_saves();

        // 预加载所有存档信息（用于显示元数据）
        let save_infos: std::collections::HashMap<u32, SaveInfo> = self
            .saves_cache
            .iter()
            .filter_map(|(slot, _)| save_manager.get_save_info(*slot).map(|info| (*slot, info)))
            .collect();

        // 转换为列表项（支持 1-99 槽位）
        let items: Vec<ListItem> = (1..=MAX_SAVE_SLOTS)
            .map(|slot| {
                if let Some(info) = save_infos.get(&slot) {
                    // 有存档：显示详细信息
                    let chapter = info.chapter_title.as_deref().unwrap_or("未知章节");
                    let timestamp = info.formatted_timestamp();
                    let play_time = info.formatted_play_time();

                    ListItem::new(format!("槽位 {:02} - {}", slot, chapter), slot.to_string())
                        .with_secondary(format!("{} | 游玩: {}", timestamp, play_time))
                } else {
                    // 空槽位
                    let mut item =
                        ListItem::new(format!("槽位 {:02} - 空", slot), slot.to_string());
                    // 读档模式下，空槽位禁用
                    if self.tab == SaveLoadTab::Load {
                        item = item.with_disabled(true);
                    }
                    item
                }
            })
            .collect();

        self.save_list.set_items(items);
        self.needs_refresh = false;
    }

    /// 切换标签页
    fn switch_tab(&mut self, tab: SaveLoadTab) {
        if self.tab != tab {
            self.tab = tab;
            self.needs_init = true;
        }
    }

    /// 更新界面
    pub fn update(&mut self, ctx: &UiContext) -> SaveLoadAction {
        // 处理确认对话框
        if let Some((ref action, ref mut modal)) = self.confirm_modal {
            match modal.update(ctx) {
                ModalResult::Confirm => {
                    let result = action.clone();
                    self.confirm_modal = None;
                    self.needs_refresh = true;
                    return result;
                }
                ModalResult::Cancel => {
                    self.confirm_modal = None;
                    return SaveLoadAction::None;
                }
                ModalResult::None => {
                    return SaveLoadAction::None;
                }
            }
        }

        // ESC 返回
        if is_key_pressed(KeyCode::Escape) {
            return SaveLoadAction::Back;
        }

        // 标签按钮
        let mut new_tab = None;
        for (tab, button) in &mut self.tab_buttons {
            if button.update(ctx) {
                new_tab = Some(*tab);
            }
        }
        if let Some(tab) = new_tab {
            self.switch_tab(tab);
            return SaveLoadAction::None;
        }

        // 列表点击
        if let Some(idx) = self.save_list.update(ctx) {
            // 列表选中已在 update 中处理
            let _ = idx;
        }

        // 键盘导航
        if is_key_pressed(KeyCode::Up) {
            self.save_list.select_prev();
        }
        if is_key_pressed(KeyCode::Down) {
            self.save_list.select_next();
        }

        // 操作按钮
        if let Some(selected_idx) = self.save_list.selected_index {
            let slot = selected_idx as u32 + 1;
            let has_save = self.saves_cache.iter().any(|(s, _)| *s == slot);

            for (action_id, button) in &mut self.action_buttons {
                // 更新按钮禁用状态
                if action_id == "delete" || self.tab == SaveLoadTab::Load {
                    button.disabled = !has_save;
                }

                if button.update(ctx) {
                    match action_id.as_str() {
                        "delete" => {
                            let modal = Modal::confirm(
                                "删除存档",
                                format!("确定要删除槽位 {} 的存档吗？", slot),
                            )
                            .with_danger(true);
                            self.confirm_modal = Some((SaveLoadAction::Delete(slot), modal));
                        }
                        _ => {
                            match self.tab {
                                SaveLoadTab::Save if has_save => {
                                    // 覆盖确认
                                    let modal = Modal::confirm(
                                        "覆盖存档",
                                        format!("槽位 {} 已有存档，确定要覆盖吗？", slot),
                                    );
                                    self.confirm_modal = Some((SaveLoadAction::Save(slot), modal));
                                }
                                SaveLoadTab::Save => {
                                    return SaveLoadAction::Save(slot);
                                }
                                SaveLoadTab::Load => {
                                    return SaveLoadAction::Load(slot);
                                }
                            }
                        }
                    }
                }
            }

            // Enter 确认
            if is_key_pressed(KeyCode::Enter) {
                match self.tab {
                    SaveLoadTab::Save if has_save => {
                        let modal = Modal::confirm(
                            "覆盖存档",
                            format!("槽位 {} 已有存档，确定要覆盖吗？", slot),
                        );
                        self.confirm_modal = Some((SaveLoadAction::Save(slot), modal));
                    }
                    SaveLoadTab::Save => {
                        return SaveLoadAction::Save(slot);
                    }
                    SaveLoadTab::Load if has_save => {
                        return SaveLoadAction::Load(slot);
                    }
                    _ => {}
                }
            }
        }

        // 返回按钮
        if let Some(ref mut btn) = self.back_button
            && btn.update(ctx)
        {
            return SaveLoadAction::Back;
        }

        SaveLoadAction::None
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
        let panel_width = 600.0;
        let panel_height = 500.0;
        let panel = Panel::centered(
            panel_width,
            panel_height,
            ctx.screen_width,
            ctx.screen_height,
        )
        .with_title(match self.tab {
            SaveLoadTab::Save => "存档",
            SaveLoadTab::Load => "读档",
        });
        panel.draw(ctx, text_renderer);

        // 标签按钮
        for (_, button) in &self.tab_buttons {
            button.draw(ctx, text_renderer);
        }

        // 列表
        self.save_list.draw(ctx, text_renderer);

        // 操作按钮
        for (_, button) in &self.action_buttons {
            button.draw(ctx, text_renderer);
        }

        // 返回按钮
        if let Some(ref btn) = self.back_button {
            btn.draw(ctx, text_renderer);
        }

        // 确认对话框
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

    /// 是否需要刷新
    pub fn needs_refresh(&self) -> bool {
        self.needs_refresh
    }
}

impl Default for SaveLoadScreen {
    fn default() -> Self {
        Self::new()
    }
}
