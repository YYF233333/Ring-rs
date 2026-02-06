//! # 角色相关命令执行
//!
//! 处理 ShowCharacter 和 HideCharacter 命令。

use crate::renderer::RenderState;
use crate::renderer::effects::{self, EffectKind, EffectRequest, EffectTarget, defaults};
use vn_runtime::command::{Position, Transition};

use super::CommandExecutor;
use super::types::ExecuteResult;

impl CommandExecutor {
    /// 执行 ShowCharacter
    ///
    /// 阶段 24 扩展：如果角色已存在且位置变更，生成 Move 动画命令。
    /// 阶段 25 重构：使用统一 `effects::resolve()` 解析过渡效果。
    /// 阶段 25 续：产出 `EffectRequest` 替代 `CharacterAnimationCommand`。
    pub(super) fn execute_show_character(
        &mut self,
        path: &str,
        alias: &str,
        position: Position,
        transition: &Option<Transition>,
        render_state: &mut RenderState,
    ) -> ExecuteResult {
        // 统一解析过渡效果
        let effect = transition.as_ref().map(effects::resolve);

        // 检查角色是否已经存在（位置变更场景）
        let existing_character = render_state
            .visible_characters
            .get(alias)
            .map(|c| (c.position, c.texture_path.clone(), c.anim.alpha()));

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

                // 仅 move/slide 效果触发移动动画
                if let Some(ref effect) = effect
                    && effect.is_move_effect()
                {
                    let move_duration = effect.duration_or(defaults::MOVE_DURATION);
                    self.last_output.effect_requests.push(EffectRequest {
                        target: EffectTarget::CharacterMove {
                            alias: alias.to_string(),
                            old_position,
                            new_position: position,
                        },
                        effect: effects::ResolvedEffect {
                            kind: effect.kind.clone(),
                            duration: Some(move_duration),
                            easing: effect.easing,
                        },
                    });
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

        // 计算 alpha 动画持续时间
        // dissolve / fade 在立绘上下文中均为 alpha 淡入
        let alpha_duration = effect
            .as_ref()
            .filter(|e| matches!(e.kind, EffectKind::Dissolve | EffectKind::Fade))
            .map(|e| e.duration_or(defaults::CHARACTER_ALPHA_DURATION))
            .unwrap_or(0.0);

        if alpha_duration > 0.0 {
            let resolved = effect.unwrap(); // safe: alpha_duration > 0 implies effect is Some
            self.last_output.effect_requests.push(EffectRequest {
                target: EffectTarget::CharacterShow {
                    alias: alias.to_string(),
                },
                effect: effects::ResolvedEffect {
                    kind: resolved.kind,
                    duration: Some(alpha_duration),
                    easing: resolved.easing,
                },
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
    ///
    /// 阶段 25 重构：使用统一 `effects::resolve()` 解析过渡效果。
    /// 阶段 25 续：产出 `EffectRequest` 替代 `CharacterAnimationCommand`。
    pub(super) fn execute_hide_character(
        &mut self,
        alias: &str,
        transition: &Option<Transition>,
        render_state: &mut RenderState,
    ) -> ExecuteResult {
        // 统一解析过渡效果
        let effect = transition.as_ref().map(effects::resolve);

        // dissolve / fade 在立绘上下文中均为 alpha 淡出
        let alpha_duration = effect
            .as_ref()
            .filter(|e| matches!(e.kind, EffectKind::Dissolve | EffectKind::Fade))
            .map(|e| e.duration_or(defaults::CHARACTER_ALPHA_DURATION))
            .unwrap_or(0.0);

        if alpha_duration > 0.0 {
            // 有过渡效果：标记为淡出，由 AnimationSystem 处理
            render_state.mark_character_fading_out(alias);
            let resolved = effect.unwrap(); // safe: alpha_duration > 0 implies effect is Some
            self.last_output.effect_requests.push(EffectRequest {
                target: EffectTarget::CharacterHide {
                    alias: alias.to_string(),
                },
                effect: effects::ResolvedEffect {
                    kind: resolved.kind,
                    duration: Some(alpha_duration),
                    easing: resolved.easing,
                },
            });
        } else {
            // 无过渡效果：立即移除
            render_state.hide_character(alias);
        }

        ExecuteResult::Ok
    }
}
