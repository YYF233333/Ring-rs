//! changeScene 相关：多阶段场景过渡的驱动更新

use crate::Renderer;
use crate::renderer::RenderState;

/// 更新场景过渡状态（基于 AnimationSystem）
///
/// 多阶段流程由 SceneTransitionManager 管理：
/// - Fade/FadeWhite: FadeIn → FadeOut → UIFadeIn → Completed
/// - Rule: FadeIn → Blackout → FadeOut → UIFadeIn → Completed
///
/// 阶段 24 重构：changeScene 不再隐式管理 UI 可见性，
/// UI 显示/隐藏由编剧通过 textBoxHide/textBoxShow 显式控制。
/// 此函数只负责驱动过渡动画和在中间点切换背景。
pub fn update_scene_transition(renderer: &mut Renderer, render_state: &mut RenderState, dt: f32) {
    // 记录过渡开始前的状态
    let was_active = renderer.is_scene_transition_active();

    if !was_active {
        return;
    }

    // 更新场景过渡
    renderer.update_scene_transition(dt);

    // 在中间点时切换背景
    if renderer.is_scene_transition_at_midpoint()
        && let Some(path) = renderer.take_pending_background()
    {
        render_state.set_background(path);
    }
}
