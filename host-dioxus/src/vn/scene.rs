use dioxus::prelude::*;

use crate::render_state::{PlaybackMode, RenderState};
use crate::state::AppState;

use super::audio_bridge::AudioBridge;
use super::background::BackgroundLayer;
use super::chapter_mark::ChapterMark;
use super::character::CharacterLayer;
use super::choice::ChoicePanel;
use super::dialogue::DialogueBox;
use super::map_overlay::MapOverlay;
use super::minigame_overlay::MinigameOverlay;
use super::nvl::NvlPanel;
use super::quick_menu::QuickMenu;
use super::rule_transition::RuleTransitionCanvas;
use super::title_card::TitleCard;
use super::transition::TransitionOverlay;
use super::video::VideoOverlay;

/// VN 场景容器：组合背景、立绘、对话框等子层。
///
/// 处理场景级效果（shake/blur/dim）和 skip-mode 切换。
/// 点击事件统一在此处理，调用 `process_click()`。
#[component]
pub fn VNScene(render_state: Signal<RenderState>) -> Element {
    let app_state = use_context::<AppState>();

    let rs = render_state.read();

    // 场景效果：shake 偏移 + blur + dim
    let se = &rs.scene_effect;
    let scene_transform = if se.shake_offset_x != 0.0 || se.shake_offset_y != 0.0 {
        format!(
            "transform: translate({}px, {}px);",
            se.shake_offset_x, se.shake_offset_y
        )
    } else {
        String::new()
    };
    let scene_filter = if se.blur_amount > 0.0 {
        format!("filter: blur({}px);", se.blur_amount)
    } else {
        String::new()
    };
    let scene_style = format!("{scene_transform} {scene_filter}");

    // dim 覆盖层
    let dim_level = se.dim_level;

    // skip-mode class
    let skip_class = if rs.playback_mode == PlaybackMode::Skip {
        " skip-mode"
    } else {
        ""
    };

    rsx! {
        div {
            class: "vn-scene{skip_class}",
            onclick: move |_| {
                if let Ok(mut inner) = app_state.inner.lock() {
                    inner.process_click();
                }
            },

            // 场景效果包装层
            div {
                class: "vn-scene__layers",
                style: "{scene_style}",

                BackgroundLayer { render_state }
                CharacterLayer { render_state }
                TransitionOverlay { render_state }
                RuleTransitionCanvas { render_state }
            }

            // dim 覆盖层
            if dim_level > 0.0 {
                div {
                    class: "vn-scene__dim",
                    style: "opacity: {dim_level};",
                }
            }

            DialogueBox { render_state }
            NvlPanel { render_state }
            ChoicePanel { render_state }
            ChapterMark { render_state }
            TitleCard { render_state }
            VideoOverlay { render_state }
            MapOverlay { render_state }
            MinigameOverlay { render_state }
            QuickMenu { render_state }
            AudioBridge { render_state }
        }
    }
}
