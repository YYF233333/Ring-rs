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
        // 阶段 24：不再隐式清除 ChapterMark
        // ChapterMark 有独立的定时生命周期，不受对话影响

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
        // 清除对话框（选择分支替代对话）
        render_state.clear_dialogue();
        // 阶段 24：不再隐式清除 ChapterMark

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
    /// 阶段 24 重构：
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
    pub(super) fn execute_text_box_hide(&mut self, render_state: &mut RenderState) -> ExecuteResult {
        render_state.ui_visible = false;
        ExecuteResult::Ok
    }

    /// 执行 TextBoxShow（显示对话框）
    pub(super) fn execute_text_box_show(&mut self, render_state: &mut RenderState) -> ExecuteResult {
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
}
