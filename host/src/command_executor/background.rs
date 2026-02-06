//! # 背景相关命令执行
//!
//! 处理 ShowBackground 和 ChangeScene 命令。

use crate::renderer::RenderState;
use crate::resources::ResourceManager;
use tracing::debug;
use vn_runtime::command::{Transition, TransitionArg};

use super::CommandExecutor;
use super::types::{ExecuteResult, SceneTransitionCommand, TransitionInfo};

impl CommandExecutor {
    /// 执行 ShowBackground
    pub(super) fn execute_show_background(
        &mut self,
        path: &str,
        transition: Option<Transition>,
        render_state: &mut RenderState,
    ) -> ExecuteResult {
        // 保存旧背景用于过渡效果
        let old_background = render_state.current_background.clone();

        // 设置新背景路径
        render_state.set_background(path.to_string());

        // 记录过渡信息
        self.last_output.transition_info = TransitionInfo {
            has_background_transition: true,
            old_background,
            transition: transition.clone(),
        };

        // 处理过渡效果
        if let Some(ref trans) = transition {
            self.start_transition(trans);
        }

        ExecuteResult::Ok
    }

    /// 执行 ChangeScene（场景切换 — 遮罩过渡 + 切换背景）
    ///
    /// 阶段 24 重构后，changeScene **只负责**：
    /// - 拉遮罩/蒙版过渡 + 切换背景
    /// - **不再隐式隐藏 UI**（由编剧通过 textBoxHide 显式控制）
    /// - **不再隐式清除立绘**（由编剧通过 clearCharacters / hide 显式控制）
    pub(super) fn execute_change_scene(
        &mut self,
        path: &str,
        transition: Option<Transition>,
        render_state: &mut RenderState,
        resource_manager: &ResourceManager,
    ) -> ExecuteResult {
        // 保存旧背景用于过渡效果
        let old_background = render_state.current_background.clone();

        // 根据 transition 类型发出场景切换命令
        if let Some(ref trans) = transition {
            let name_lower = trans.name.to_lowercase();
            let duration = trans.get_duration().unwrap_or(0.5) as f32;

            match name_lower.as_str() {
                "fade" => {
                    // 黑屏遮罩 - 发出 Fade 命令
                    self.last_output.scene_transition = Some(SceneTransitionCommand::Fade {
                        duration,
                        pending_background: path.to_string(),
                    });
                    debug!(duration = duration, "changeScene: Fade 黑屏过渡");
                }
                "fadewhite" => {
                    // 白屏遮罩 - 发出 FadeWhite 命令
                    self.last_output.scene_transition = Some(SceneTransitionCommand::FadeWhite {
                        duration,
                        pending_background: path.to_string(),
                    });
                    debug!(duration = duration, "changeScene: FadeWhite 白屏过渡");
                }
                "rule" => {
                    // 图片遮罩 - 使用 resource_manager 规范化路径
                    let raw_mask_path = trans
                        .get_named("mask")
                        .and_then(|arg| {
                            if let TransitionArg::String(s) = arg {
                                Some(s.clone())
                            } else {
                                None
                            }
                        })
                        .unwrap_or_default();

                    // 规范化路径
                    let normalized_mask_path = resource_manager.resolve_path(&raw_mask_path);
                    let reversed = trans.get_reversed().unwrap_or(false);

                    // 发出 Rule 命令
                    self.last_output.scene_transition = Some(SceneTransitionCommand::Rule {
                        duration,
                        pending_background: path.to_string(),
                        mask_path: normalized_mask_path.clone(),
                        reversed,
                    });
                    debug!(
                        mask = %normalized_mask_path,
                        duration = duration,
                        reversed = reversed,
                        "changeScene: Rule 遮罩过渡"
                    );
                }
                "dissolve" => {
                    // Dissolve 使用 TransitionManager 处理背景过渡
                    self.last_output.transition_info = TransitionInfo {
                        has_background_transition: true,
                        old_background: old_background.clone(),
                        transition: transition.clone(),
                    };
                    // 立即切换背景（交叉溶解依赖 old_background）
                    render_state.set_background(path.to_string());
                    debug!(duration = duration, "changeScene: Dissolve 过渡");
                }
                _ => {
                    // 未知效果，使用默认 dissolve
                    self.last_output.transition_info = TransitionInfo {
                        has_background_transition: true,
                        old_background: old_background.clone(),
                        transition: transition.clone(),
                    };
                    render_state.set_background(path.to_string());
                    debug!(name = %trans.name, "changeScene: 未知效果，使用 dissolve");
                }
            }
        } else {
            // 无过渡效果，立即切换背景
            render_state.set_background(path.to_string());
        }

        ExecuteResult::Ok
    }
}
