//! changeScene 相关：多阶段场景过渡的驱动更新

use crate::Renderer;
use crate::renderer::RenderState;

/// 更新场景过渡状态（基于 AnimationSystem）
///
/// 多阶段流程由 SceneTransitionManager 管理：
/// - Fade/FadeWhite: FadeIn → FadeOut → UIFadeIn → Completed
/// - Rule: FadeIn → Blackout → FadeOut → UIFadeIn → Completed
pub fn update_scene_transition(renderer: &mut Renderer, render_state: &mut RenderState, dt: f32) {
    // 记录过渡开始前的状态
    let was_active = renderer.is_scene_transition_active();

    if !was_active {
        return;
    }

    // 更新场景过渡
    renderer.update_scene_transition(dt);

    // 在中间点时切换背景
    if renderer.is_scene_transition_at_midpoint() {
        if let Some(path) = renderer.take_pending_background() {
            render_state.set_background(path);
        }
    }

    // 当进入 UI 淡入阶段时，恢复 UI 可见性
    if renderer.is_scene_transition_ui_fading_in() && !render_state.ui_visible {
        render_state.ui_visible = true;
    }

    // 过渡完成时恢复 UI（包括被跳过的情况）
    if !renderer.is_scene_transition_active() {
        render_state.ui_visible = true;
    }
}
