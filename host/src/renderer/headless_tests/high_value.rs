//! 高价值测试：组合绘制链路、state -> DrawCommand 契约、dim/blur 阈值行为。

use super::*;
use std::sync::Arc;

#[test]
fn test_build_draw_commands_empty_state() {
    let renderer = Renderer::new(1920.0, 1080.0);
    let state = RenderState::default();
    let resource_manager = make_test_resource_manager();
    let manifest = Manifest::with_defaults();

    let cmds = renderer.build_draw_commands(&state, &resource_manager, &manifest);
    assert!(cmds.is_empty());
}

#[test]
fn test_build_draw_commands_with_background() {
    let renderer = Renderer::new(1920.0, 1080.0);
    let state = RenderState {
        current_background: Some("bg/sky.png".to_string()),
        ..Default::default()
    };

    let mut resource_manager = make_test_resource_manager();
    let tex: Arc<dyn crate::rendering_types::Texture> = Arc::new(NullTexture::new(1920, 1080));
    resource_manager
        .texture_cache_mut()
        .insert("bg/sky.png".to_string(), tex);

    let manifest = Manifest::with_defaults();
    let cmds = renderer.build_draw_commands(&state, &resource_manager, &manifest);
    assert!(cmds.iter().any(|c| matches!(c, DrawCommand::Sprite { .. })));
}

#[test]
fn test_build_draw_commands_with_character() {
    let mut renderer = Renderer::new(1920.0, 1080.0);
    renderer.set_screen_size(1920.0, 1080.0);

    let mut state = RenderState::default();
    state.show_character(
        "hero".to_string(),
        "characters/hero/normal.png".to_string(),
        vn_runtime::command::Position::Center,
    );

    let mut resource_manager = make_test_resource_manager();
    let tex: Arc<dyn crate::rendering_types::Texture> = Arc::new(NullTexture::new(512, 1024));
    resource_manager
        .texture_cache_mut()
        .insert("characters/hero/normal.png".to_string(), tex);

    let manifest = Manifest::with_defaults();
    let cmds = renderer.build_draw_commands(&state, &resource_manager, &manifest);

    assert!(cmds.iter().any(|c| matches!(c, DrawCommand::Sprite { .. })));
}

#[test]
fn test_build_draw_commands_with_dim_adds_rect() {
    let (renderer, resource_manager, manifest) = build_draw_deps();
    let mut state = RenderState::default();
    state.scene_effect.dim_level = 0.5;

    let cmds = renderer.build_draw_commands(&state, &resource_manager, &manifest);
    let has_dim_rect = cmds.iter().any(|c| {
        matches!(c, DrawCommand::Rect { color, .. } if (color[3] - 0.5).abs() < 0.01 && color[0] == 0.0)
    });
    assert!(has_dim_rect);
}

#[test]
fn test_build_draw_commands_with_blur_adds_rect() {
    let (renderer, resource_manager, manifest) = build_draw_deps();
    let mut state = RenderState::default();
    state.scene_effect.blur_amount = 1.0;

    let cmds = renderer.build_draw_commands(&state, &resource_manager, &manifest);

    let has_blur_rect = cmds.iter().any(|c| {
        matches!(c, DrawCommand::Rect { color, .. } if color[0] == 1.0 && color[1] == 1.0 && color[2] == 1.0 && color[3] > 0.0)
    });
    assert!(has_blur_rect);
}

#[test]
fn test_build_draw_commands_below_threshold_no_overlay() {
    let (renderer, resource_manager, manifest) = build_draw_deps();
    let mut state = RenderState::default();
    state.scene_effect.dim_level = 0.001;
    state.scene_effect.blur_amount = 0.001;

    let cmds = renderer.build_draw_commands(&state, &resource_manager, &manifest);
    assert!(!cmds.iter().any(|c| matches!(c, DrawCommand::Rect { .. })));
}
