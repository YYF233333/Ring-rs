//! # 背景相关命令执行
//!
//! 处理 ShowBackground 和 ChangeScene 命令。

use crate::renderer::RenderState;
use crate::renderer::effects::{self, EffectKind, defaults};
use crate::resources::ResourceManager;
use tracing::debug;

use super::CommandExecutor;
use super::types::{ExecuteResult, SceneTransitionCommand, TransitionInfo};

impl CommandExecutor {
    /// 执行 ShowBackground
    ///
    /// 阶段 25 重构：使用统一 `effects::resolve()` 解析过渡效果。
    pub(super) fn execute_show_background(
        &mut self,
        path: &str,
        transition: Option<vn_runtime::command::Transition>,
        render_state: &mut RenderState,
    ) -> ExecuteResult {
        // 保存旧背景用于过渡效果
        let old_background = render_state.current_background.clone();

        // 设置新背景路径
        render_state.set_background(path.to_string());

        // 统一解析过渡效果
        let effect = transition.as_ref().map(effects::resolve);

        // 记录过渡信息（使用 ResolvedEffect）
        self.last_output.transition_info = TransitionInfo {
            has_background_transition: true,
            old_background,
            effect: effect.clone(),
        };

        // 处理过渡效果
        if let Some(ref effect) = effect {
            let duration = effect.duration_or(defaults::BACKGROUND_DISSOLVE_DURATION);
            self.start_transition(duration);
        }

        ExecuteResult::Ok
    }

    /// 执行 ChangeScene（场景切换 — 遮罩过渡 + 切换背景）
    ///
    /// 阶段 24 重构后，changeScene **只负责**：
    /// - 拉遮罩/蒙版过渡 + 切换背景
    /// - **不再隐式隐藏 UI**（由编剧通过 textBoxHide 显式控制）
    /// - **不再隐式清除立绘**（由编剧通过 clearCharacters / hide 显式控制）
    ///
    /// 阶段 25 重构：使用统一 `effects::resolve()` 解析过渡效果。
    pub(super) fn execute_change_scene(
        &mut self,
        path: &str,
        transition: Option<vn_runtime::command::Transition>,
        render_state: &mut RenderState,
        resource_manager: &ResourceManager,
    ) -> ExecuteResult {
        // 保存旧背景用于过渡效果
        let old_background = render_state.current_background.clone();

        if let Some(ref trans) = transition {
            let effect = effects::resolve(trans);

            match &effect.kind {
                EffectKind::Fade => {
                    let duration = effect.duration_or(defaults::FADE_DURATION);
                    self.last_output.scene_transition = Some(SceneTransitionCommand::Fade {
                        duration,
                        pending_background: path.to_string(),
                    });
                    debug!(duration = duration, "changeScene: Fade 黑屏过渡");
                }
                EffectKind::FadeWhite => {
                    let duration = effect.duration_or(defaults::FADE_WHITE_DURATION);
                    self.last_output.scene_transition = Some(SceneTransitionCommand::FadeWhite {
                        duration,
                        pending_background: path.to_string(),
                    });
                    debug!(duration = duration, "changeScene: FadeWhite 白屏过渡");
                }
                EffectKind::Rule {
                    mask_path,
                    reversed,
                } => {
                    let duration = effect.duration_or(defaults::RULE_DURATION);
                    // 规范化路径
                    let normalized_mask_path = resource_manager.resolve_path(mask_path);

                    self.last_output.scene_transition = Some(SceneTransitionCommand::Rule {
                        duration,
                        pending_background: path.to_string(),
                        mask_path: normalized_mask_path.clone(),
                        reversed: *reversed,
                    });
                    debug!(
                        mask = %normalized_mask_path,
                        duration = duration,
                        reversed = reversed,
                        "changeScene: Rule 遮罩过渡"
                    );
                }
                EffectKind::Dissolve => {
                    // Dissolve 使用 TransitionManager 处理背景过渡
                    self.last_output.transition_info = TransitionInfo {
                        has_background_transition: true,
                        old_background: old_background.clone(),
                        effect: Some(effect),
                    };
                    render_state.set_background(path.to_string());
                    debug!("changeScene: Dissolve 过渡");
                }
                _ => {
                    // 未知/None/Move 等：使用默认 dissolve
                    let fallback_effect = effects::ResolvedEffect {
                        kind: EffectKind::Dissolve,
                        duration: effect.duration,
                        easing: effect.easing,
                    };
                    self.last_output.transition_info = TransitionInfo {
                        has_background_transition: true,
                        old_background: old_background.clone(),
                        effect: Some(fallback_effect),
                    };
                    render_state.set_background(path.to_string());
                    debug!(kind = ?effect.kind, "changeScene: 降级为 dissolve");
                }
            }
        } else {
            // 无过渡效果，立即切换背景
            render_state.set_background(path.to_string());
        }

        ExecuteResult::Ok
    }
}
