//! # UI 相关命令执行
//!
//! 处理 ShowText、PresentChoices、ChapterMark 命令。

use crate::renderer::{ChoiceItem, RenderState};
use vn_runtime::command::Choice;

use super::CommandExecutor;
use super::types::ExecuteResult;

impl CommandExecutor {
    /// 执行 ShowText
    pub(super) fn execute_show_text(
        &mut self,
        speaker: Option<String>,
        content: &str,
        render_state: &mut RenderState,
    ) -> ExecuteResult {
        // 清除章节标记（避免遮挡对话）
        render_state.clear_chapter_mark();

        // 开始打字机效果
        render_state.start_typewriter(speaker, content.to_string());

        // ShowText 通常需要等待用户点击
        ExecuteResult::WaitForClick
    }

    /// 执行 PresentChoices
    pub(super) fn execute_present_choices(
        &mut self,
        style: Option<String>,
        choices: &[Choice],
        render_state: &mut RenderState,
    ) -> ExecuteResult {
        // 清除对话框和章节标记
        render_state.clear_dialogue();
        render_state.clear_chapter_mark();

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

    /// 执行 ChapterMark
    pub(super) fn execute_chapter_mark(
        &mut self,
        title: &str,
        level: u8,
        render_state: &mut RenderState,
    ) -> ExecuteResult {
        // 清除其他 UI 元素
        render_state.clear_dialogue();
        render_state.clear_choices();

        // 显示章节标记
        render_state.set_chapter_mark(title.to_string(), level);

        // 章节标记通常需要等待用户点击
        ExecuteResult::WaitForClick
    }
}
