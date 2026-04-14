use dioxus::prelude::*;

use crate::render_state::{RenderState, SceneTransitionKind, SceneTransitionPhaseState};

/// 场景过渡覆盖层：处理 Fade / FadeWhite 类型。
///
/// Rule 类型由 RuleTransitionCanvas 单独处理。
/// 过渡 phase 由后端驱动（FadeIn→Hold→FadeOut→Completed）。
///
/// 为确保 CSS transition 正常工作，overlay 始终存在于 DOM 中。
/// 未激活时 opacity=0 + pointer-events: none，激活时通过 CSS transition 动画。
#[component]
pub fn TransitionOverlay(render_state: Signal<RenderState>) -> Element {
    let scene_transition = use_memo(move || render_state.read().scene_transition.clone());

    let st_ref = scene_transition.read();
    let transition = st_ref.as_ref().filter(|t| {
        !matches!(t.transition_type, SceneTransitionKind::Rule { .. })
            && t.phase != SceneTransitionPhaseState::Completed
    });

    let (bg_color, opacity, td) = match transition {
        Some(t) => {
            let bg = match t.transition_type {
                SceneTransitionKind::FadeWhite => "rgba(255,255,255,1)",
                _ => "rgba(0,0,0,1)",
            };
            let duration = t.duration;
            // FadeIn: 0→1, Hold: 1, FadeOut: 1→0
            let opacity = match t.phase {
                SceneTransitionPhaseState::FadeIn => 1.0,
                SceneTransitionPhaseState::Hold => 1.0,
                SceneTransitionPhaseState::FadeOut => 0.0,
                SceneTransitionPhaseState::Completed => 0.0,
            };
            // FadeIn 和 FadeOut 需要过渡动画，Hold 不需要
            let td = match t.phase {
                SceneTransitionPhaseState::Hold => 0.0,
                _ => duration,
            };
            (bg, opacity, td)
        }
        None => ("rgba(0,0,0,1)", 0.0_f32, 0.0_f32),
    };

    rsx! {
        div {
            class: "vn-transition-overlay",
            style: "background: {bg_color}; opacity: {opacity}; transition: opacity {td}s ease;",
        }
    }
}
