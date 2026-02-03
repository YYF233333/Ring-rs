//! # 角色相关命令执行
//!
//! 处理 ShowCharacter 和 HideCharacter 命令。

use crate::renderer::RenderState;
use vn_runtime::command::{Position, Transition};

use super::CommandExecutor;
use super::types::{CharacterAnimationCommand, ExecuteResult};

impl CommandExecutor {
    /// 执行 ShowCharacter
    pub(super) fn execute_show_character(
        &mut self,
        path: &str,
        alias: &str,
        position: Position,
        transition: &Option<Transition>,
        render_state: &mut RenderState,
    ) -> ExecuteResult {
        // 解析过渡效果持续时间
        // 如果 transition 存在且是 dissolve/fade，使用指定的 duration 或默认 0.3 秒
        let duration = transition
            .as_ref()
            .and_then(|t| {
                let name_lower = t.name.to_lowercase();
                if name_lower == "dissolve" || name_lower == "fade" {
                    Some(t.get_duration().map(|d| d as f32).unwrap_or(0.3))
                } else {
                    None
                }
            })
            .unwrap_or(0.0); // 无过渡效果时立即显示

        // 在 RenderState 中创建角色数据
        render_state.show_character(alias.to_string(), path.to_string(), position);

        // 如果有过渡效果，记录动画命令（由 main.rs 处理）
        if duration > 0.0 {
            self.last_output.character_animation = Some(CharacterAnimationCommand::Show {
                alias: alias.to_string(),
                duration,
            });
        } else {
            // 无过渡效果：直接设置角色为完全可见
            if let Some(anim) = render_state.get_character_anim(alias) {
                anim.set_alpha(1.0);
            }
        }

        ExecuteResult::Ok
    }

    /// 执行 HideCharacter
    pub(super) fn execute_hide_character(
        &mut self,
        alias: &str,
        transition: &Option<Transition>,
        render_state: &mut RenderState,
    ) -> ExecuteResult {
        // 解析过渡效果持续时间
        let duration = transition
            .as_ref()
            .and_then(|t| {
                let name_lower = t.name.to_lowercase();
                if name_lower == "dissolve" || name_lower == "fade" {
                    Some(t.get_duration().map(|d| d as f32).unwrap_or(0.3))
                } else {
                    None
                }
            })
            .unwrap_or(0.0);

        if duration > 0.0 {
            // 有过渡效果：标记为淡出，由 AnimationSystem 处理
            render_state.mark_character_fading_out(alias);
            self.last_output.character_animation = Some(CharacterAnimationCommand::Hide {
                alias: alias.to_string(),
                duration,
            });
        } else {
            // 无过渡效果：立即移除
            render_state.hide_character(alias);
        }

        ExecuteResult::Ok
    }
}
