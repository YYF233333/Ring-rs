use dioxus::prelude::*;

use crate::render_state::{RenderState, SceneTransitionKind, SceneTransitionPhaseState};

/// 场景过渡覆盖层：处理 Fade / FadeWhite 类型。
///
/// Rule 类型由 RuleTransitionCanvas 单独处理。
/// 过渡 phase 由后端驱动（FadeIn→Hold→FadeOut→Completed）。
#[component]
pub fn TransitionOverlay(render_state: Signal<RenderState>) -> Element {
    let rs = render_state.read();

    let transition = match &rs.scene_transition {
        Some(t) => t,
        None => return rsx! {},
    };

    // Rule 类型不在此组件处理
    if matches!(transition.transition_type, SceneTransitionKind::Rule { .. }) {
        return rsx! {};
    }

    // Completed 阶段不渲染
    if transition.phase == SceneTransitionPhaseState::Completed {
        return rsx! {};
    }

    let bg_color = match transition.transition_type {
        SceneTransitionKind::FadeWhite => "rgba(255,255,255,1)",
        _ => "rgba(0,0,0,1)",
    };

    let duration = transition.duration;

    // FadeIn: 0→1, Hold: 1, FadeOut: 1→0
    let opacity = match transition.phase {
        SceneTransitionPhaseState::FadeIn => 1.0,
        SceneTransitionPhaseState::Hold => 1.0,
        SceneTransitionPhaseState::FadeOut => 0.0,
        SceneTransitionPhaseState::Completed => 0.0,
    };

    // FadeIn 和 FadeOut 需要过渡动画，Hold 不需要
    let td = match transition.phase {
        SceneTransitionPhaseState::Hold => 0.0,
        _ => duration,
    };

    rsx! {
        div {
            class: "vn-transition-overlay",
            style: "background: {bg_color}; opacity: {opacity}; transition: opacity {td}s ease;",
        }
    }
}
