//! # Command Executor 模块
//!
//! Command 执行器，负责将 Runtime 发出的 Command 转换为实际操作。
//!
//! ## 设计说明
//!
//! - `CommandExecutor` 接收 `Command`，更新 `RenderState` 和控制音频
//! - 执行器不直接渲染，只更新状态，渲染由 `Renderer` 负责
//! - 动画/过渡效果通过 `EffectRequest` 统一传递给 `EffectApplier`
//!
//! ## 模块结构
//!
//! - `audio`: 音频命令执行
//! - `background`: 背景命令执行
//! - `character`: 角色命令执行
//! - `ui`: UI 命令执行
//! - `types`: 类型定义

mod audio;
mod background;
mod character;
mod types;
mod ui;

pub use types::*;

use crate::renderer::RenderState;
use crate::resources::ResourceManager;
use vn_runtime::command::Command;

/// Command 执行器
///
/// 负责将 Runtime 发出的 Command 转换为实际的渲染状态更新。
/// 动画/过渡效果通过 `last_output.effect_requests` 传递给 `EffectApplier`。
#[derive(Debug)]
pub struct CommandExecutor {
    /// 最近一次执行的输出
    pub last_output: CommandOutput,
}

impl CommandExecutor {
    /// 创建新的 Command 执行器
    pub fn new() -> Self {
        Self {
            last_output: CommandOutput::default(),
        }
    }

    /// 执行单个 Command
    ///
    /// 根据 Command 类型更新 RenderState。
    /// 返回执行结果，同时更新 `last_output` 以获取过渡和音频信息。
    pub fn execute(
        &mut self,
        command: &Command,
        render_state: &mut RenderState,
        resource_manager: &ResourceManager,
    ) -> ExecuteResult {
        // 重置输出
        self.last_output = CommandOutput::default();

        let result = match command {
            Command::ShowBackground { path, transition } => {
                self.execute_show_background(path, transition.clone(), render_state)
            }
            Command::ChangeScene { path, transition } => {
                // ChangeScene：遮罩过渡 + 切换背景（不再隐式清理 UI/立绘）
                self.execute_change_scene(path, transition.clone(), render_state, resource_manager)
            }
            Command::ShowCharacter {
                path,
                alias,
                position,
                transition,
            } => self.execute_show_character(path, alias, *position, transition, render_state),
            Command::HideCharacter { alias, transition } => {
                self.execute_hide_character(alias, transition, render_state)
            }
            Command::ShowText { speaker, content } => {
                self.execute_show_text(speaker.clone(), content, render_state)
            }
            Command::PresentChoices { style, choices } => {
                self.execute_present_choices(style.clone(), choices, render_state)
            }
            Command::ChapterMark { title, level } => {
                self.execute_chapter_mark(title, *level, render_state)
            }
            Command::PlayBgm { path, looping } => self.execute_play_bgm(path, *looping),
            Command::StopBgm { fade_out } => self.execute_stop_bgm(*fade_out),
            Command::PlaySfx { path } => self.execute_play_sfx(path),
            Command::TextBoxHide => self.execute_text_box_hide(render_state),
            Command::TextBoxShow => self.execute_text_box_show(render_state),
            Command::TextBoxClear => self.execute_text_box_clear(render_state),
            Command::ClearCharacters => self.execute_clear_characters(render_state),
        };

        self.last_output.result = result.clone();
        result
    }

    /// 批量执行 Commands
    ///
    /// 执行一组 Commands，返回最后一个需要等待的结果。
    pub fn execute_batch(
        &mut self,
        commands: &[Command],
        render_state: &mut RenderState,
        resource_manager: &ResourceManager,
    ) -> ExecuteResult {
        let mut last_result = ExecuteResult::Ok;

        for command in commands {
            let result = self.execute(command, render_state, resource_manager);

            // 记录需要等待的结果
            match &result {
                ExecuteResult::WaitForClick
                | ExecuteResult::WaitForChoice { .. }
                | ExecuteResult::WaitForTime(_) => {
                    last_result = result;
                }
                ExecuteResult::Error(_) => {
                    return result; // 遇到错误立即返回
                }
                _ => {}
            }
        }

        last_result
    }
}

impl Default for CommandExecutor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests;
