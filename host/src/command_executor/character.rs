//! # 角色相关命令执行
//!
//! 处理 ShowCharacter 和 HideCharacter 命令。

use crate::renderer::RenderState;
use vn_runtime::command::{Position, Transition};

use super::CommandExecutor;
use super::types::{CharacterAnimationCommand, ExecuteResult};

impl CommandExecutor {
    /// 执行 ShowCharacter
    ///
    /// 阶段 24 扩展：如果角色已存在且位置变更，生成 Move 动画命令。
    pub(super) fn execute_show_character(
        &mut self,
        path: &str,
        alias: &str,
        position: Position,
        transition: &Option<Transition>,
        render_state: &mut RenderState,
    ) -> ExecuteResult {
        // 解析过渡效果持续时间（show/hide 的淡入淡出效果）
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

        // 检查角色是否已经存在（位置变更场景）
        let existing_character = render_state.visible_characters.get(alias).map(|c| {
            (c.position, c.texture_path.clone(), c.anim.alpha())
        });

        if let Some((old_position, existing_path, current_alpha)) = existing_character {
            // 角色已存在：这是一个位置变更（或立绘更换）
            let is_position_change = old_position != position;
            let is_same_texture = existing_path == path;

            if is_same_texture && is_position_change {
                // 同一立绘、不同位置：默认瞬移；只有显式 with effect 才做移动动画
                // 直接更新 position 预设（不重建角色、不重置动画状态）
                if let Some(character) = render_state.visible_characters.get_mut(alias) {
                    character.position = position;
                }

                if let Some(transition) = transition {
                    // 位置变更的“移动动画”需要显式指定效果名（避免 `dissolve/fade` 语义混淆）
                    // - `with move(...)` / `with slide(...)`：做平滑移动
                    // - 其他效果（如 dissolve/fade）：仅作为“show/hide 的淡入淡出”，位置变更仍瞬移
                    let name_lower = transition.name.to_lowercase();
                    if name_lower == "move" || name_lower == "slide" {
                        let move_duration = transition
                            .get_duration()
                            .map(|d| d as f32)
                            .unwrap_or(0.3);
                        self.last_output.character_animation = Some(CharacterAnimationCommand::Move {
                            alias: alias.to_string(),
                            old_position,
                            new_position: position,
                            duration: move_duration,
                        });
                    }
                }

                return ExecuteResult::Ok;
            }

            // 不同立绘或无位置变更 → 重建角色（走正常逻辑）
            // 如果角色已完全可见且无位置变更，只需更新纹理
            if !is_position_change && current_alpha >= 0.99 {
                if let Some(character) = render_state.visible_characters.get_mut(alias) {
                    character.texture_path = path.to_string();
                }
                return ExecuteResult::Ok;
            }
        }

        // 新角色或需要重建的角色：创建角色数据
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
