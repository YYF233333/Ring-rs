//! # 背景相关命令执行
//!
//! 处理 ShowBackground 和 ChangeScene 命令。

use crate::renderer::RenderState;
use crate::renderer::effects::{self, EffectKind, EffectRequest, EffectTarget};
use crate::resources::ResourceManager;
use tracing::debug;

use super::CommandExecutor;
use super::types::ExecuteResult;

impl CommandExecutor {
    /// 执行 ShowBackground
    ///
    /// 阶段 25 续：产出 `EffectRequest` 替代 `TransitionInfo`。
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

        // 产出背景过渡效果请求
        if let Some(effect) = effect {
            self.last_output.effect_requests.push(EffectRequest {
                target: EffectTarget::BackgroundTransition { old_background },
                effect,
            });
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
    /// 阶段 25 续：产出 `EffectRequest` 替代 `SceneTransitionCommand` / `TransitionInfo`。
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
            let mut effect = effects::resolve(trans);

            match &effect.kind {
                EffectKind::Fade => {
                    debug!(
                        duration = ?effect.duration,
                        "changeScene: Fade 黑屏过渡"
                    );
                    self.last_output.effect_requests.push(EffectRequest {
                        target: EffectTarget::SceneTransition {
                            pending_background: path.to_string(),
                        },
                        effect,
                    });
                }
                EffectKind::FadeWhite => {
                    debug!(
                        duration = ?effect.duration,
                        "changeScene: FadeWhite 白屏过渡"
                    );
                    self.last_output.effect_requests.push(EffectRequest {
                        target: EffectTarget::SceneTransition {
                            pending_background: path.to_string(),
                        },
                        effect,
                    });
                }
                EffectKind::Rule {
                    mask_path,
                    reversed,
                } => {
                    // 规范化路径
                    let normalized_mask_path = resource_manager.resolve_path(mask_path);
                    debug!(
                        mask = %normalized_mask_path,
                        duration = ?effect.duration,
                        reversed = reversed,
                        "changeScene: Rule 遮罩过渡"
                    );
                    // 用规范化后的路径重建 EffectKind::Rule
                    let resolved_effect = effects::ResolvedEffect {
                        kind: EffectKind::Rule {
                            mask_path: normalized_mask_path,
                            reversed: *reversed,
                        },
                        duration: effect.duration,
                        easing: effect.easing,
                    };
                    self.last_output.effect_requests.push(EffectRequest {
                        target: EffectTarget::SceneTransition {
                            pending_background: path.to_string(),
                        },
                        effect: resolved_effect,
                    });
                }
                EffectKind::Dissolve => {
                    // Dissolve 使用 TransitionManager 处理背景过渡
                    render_state.set_background(path.to_string());
                    debug!("changeScene: Dissolve 过渡");
                    self.last_output.effect_requests.push(EffectRequest {
                        target: EffectTarget::BackgroundTransition {
                            old_background: old_background.clone(),
                        },
                        effect,
                    });
                }
                _ => {
                    // 未知/None/Move 等：使用默认 dissolve
                    render_state.set_background(path.to_string());
                    debug!(kind = ?effect.kind, "changeScene: 降级为 dissolve");
                    effect.kind = EffectKind::Dissolve;
                    self.last_output.effect_requests.push(EffectRequest {
                        target: EffectTarget::BackgroundTransition {
                            old_background: old_background.clone(),
                        },
                        effect,
                    });
                }
            }
        } else {
            // 无过渡效果，立即切换背景
            render_state.set_background(path.to_string());
        }

        ExecuteResult::Ok
    }
}
