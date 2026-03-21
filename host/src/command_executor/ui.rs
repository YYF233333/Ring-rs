//! # UI 相关命令执行
//!
//! 处理 ShowText、PresentChoices、ChapterMark、SetTextMode 命令。

use crate::renderer::{ChoiceItem, NvlEntry, RenderState};
use vn_runtime::command::{Choice, InlineEffect, TextMode};

use super::CommandExecutor;
use super::types::ExecuteResult;

impl CommandExecutor {
    /// 执行 ShowText
    pub(super) fn execute_show_text(
        &mut self,
        speaker: Option<String>,
        content: &str,
        inline_effects: Vec<InlineEffect>,
        no_wait: bool,
        render_state: &mut RenderState,
    ) -> ExecuteResult {
        if render_state.text_mode == TextMode::NVL {
            for entry in render_state.nvl_entries.iter_mut() {
                entry.is_complete = true;
                entry.visible_chars = entry.content.chars().count();
            }
            render_state.nvl_entries.push(NvlEntry {
                speaker: speaker.clone(),
                content: content.to_string(),
                visible_chars: 0,
                is_complete: false,
            });
        }
        render_state.start_typewriter(speaker, content.to_string(), inline_effects, no_wait);
        ExecuteResult::WaitForClick
    }

    /// 执行 ExtendText（台词续接）
    pub(super) fn execute_extend_text(
        &mut self,
        content: &str,
        inline_effects: Vec<InlineEffect>,
        no_wait: bool,
        render_state: &mut RenderState,
    ) -> ExecuteResult {
        render_state.extend_dialogue(content, inline_effects, no_wait);
        ExecuteResult::WaitForClick
    }

    /// 执行 PresentChoices
    pub(super) fn execute_present_choices(
        &mut self,
        style: Option<String>,
        choices: &[Choice],
        render_state: &mut RenderState,
    ) -> ExecuteResult {
        // 清除对话框（选择分支替代对话）
        render_state.clear_dialogue();
        // 不再隐式清除 ChapterMark

        // 转换选项格式
        let choice_items: Vec<ChoiceItem> = choices
            .iter()
            .map(|c| ChoiceItem {
                text: c.text.clone(),
                target_label: c.target_label.clone(),
            })
            .collect();

        let choice_count = choice_items.len();

        // 设置选择界面
        render_state.set_choices(choice_items, style);

        ExecuteResult::WaitForChoice { choice_count }
    }

    /// 执行 ChapterMark（非阻塞，异步显示）
    ///
    /// - 章节标记是**非阻塞**的，不产生 WaitForClick
    /// - 固定持续时间后自动消失（由 Host 更新循环驱动）
    /// - 不受用户快进/点击影响
    /// - 新 chapter mark 直接覆盖旧的（避免重叠）
    pub(super) fn execute_chapter_mark(
        &mut self,
        title: &str,
        level: u8,
        render_state: &mut RenderState,
    ) -> ExecuteResult {
        // 显示章节标记（覆盖旧的）
        render_state.set_chapter_mark(title.to_string(), level);

        // 非阻塞：不等待用户点击
        ExecuteResult::Ok
    }

    /// 执行 TextBoxHide（隐藏对话框，不影响背景/立绘）
    pub(super) fn execute_text_box_hide(
        &mut self,
        render_state: &mut RenderState,
    ) -> ExecuteResult {
        render_state.ui_visible = false;
        ExecuteResult::Ok
    }

    /// 执行 TextBoxShow（显示对话框）
    pub(super) fn execute_text_box_show(
        &mut self,
        render_state: &mut RenderState,
    ) -> ExecuteResult {
        render_state.ui_visible = true;
        ExecuteResult::Ok
    }

    /// 执行 TextBoxClear（清理对话框内容）
    pub(super) fn execute_text_box_clear(
        &mut self,
        render_state: &mut RenderState,
    ) -> ExecuteResult {
        render_state.clear_dialogue();
        render_state.clear_choices();
        render_state.nvl_entries.clear();
        ExecuteResult::Ok
    }

    /// 执行 ClearCharacters（清除所有角色立绘）
    pub(super) fn execute_clear_characters(
        &mut self,
        render_state: &mut RenderState,
    ) -> ExecuteResult {
        render_state.hide_all_characters();
        ExecuteResult::Ok
    }

    /// 执行 SetTextMode（切换文本模式）
    pub(super) fn execute_set_text_mode(
        &mut self,
        mode: TextMode,
        render_state: &mut RenderState,
    ) -> ExecuteResult {
        if render_state.text_mode != mode {
            render_state.text_mode = mode;
            if mode == TextMode::ADV {
                render_state.nvl_entries.clear();
            }
        }
        ExecuteResult::Ok
    }
}
