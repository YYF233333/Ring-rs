use super::*;
use crate::rendering_types::{NullTexture, NullTextureFactory, TextureContext};
use crate::resources::ResourceManager;
use std::sync::Arc;

fn make_test_resource_manager() -> ResourceManager {
    let mut manager = ResourceManager::new("assets", 256);
    manager.set_texture_context(TextureContext::new(Arc::new(NullTextureFactory)));
    manager
}

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

// ============ draw_commands.rs 纯计算测试 ============

#[test]
fn test_get_choice_rects_correct_count() {
    let renderer = Renderer::new(1280.0, 720.0);
    let rects = renderer.get_choice_rects(3);
    assert_eq!(rects.len(), 3);
}

#[test]
fn test_get_choice_rects_centered_and_consistent_size() {
    let renderer = Renderer::new(1280.0, 720.0);
    let rects = renderer.get_choice_rects(2);
    let (_, _, w0, h0) = rects[0];
    let (_, _, w1, h1) = rects[1];
    assert!((w0 - w1).abs() < 0.01 && (h0 - h1).abs() < 0.01);
    assert!((w0 - 1280.0 * 0.6).abs() < 0.01);
}

#[test]
fn test_get_choice_rects_zero_count() {
    let renderer = Renderer::new(1280.0, 720.0);
    let rects = renderer.get_choice_rects(0);
    assert!(rects.is_empty());
}

#[test]
fn test_get_scale_factor_square_screen() {
    let renderer = Renderer::new(1000.0, 500.0);
    assert!((renderer.get_scale_factor() - 1.0).abs() < 0.01);
}

#[test]
fn test_get_scale_factor_scaled_down() {
    let mut renderer = Renderer::new(1920.0, 1080.0);
    renderer.set_screen_size(960.0, 540.0);
    assert!((renderer.get_scale_factor() - 0.5).abs() < 0.01);
}

#[test]
fn test_calculate_draw_rect_cover_wider_texture() {
    let renderer = Renderer::new(1280.0, 720.0);
    let texture = NullTexture::new(1280, 720);
    let (dw, dh, x, y) = renderer.calculate_draw_rect_for(&texture, DrawMode::Cover);
    assert!((dw - 1280.0).abs() < 0.01);
    assert!((dh - 720.0).abs() < 0.01);
    assert!((x - 0.0).abs() < 0.01);
    assert!((y - 0.0).abs() < 0.01);
}

#[test]
fn test_calculate_draw_rect_cover_tall_texture() {
    let renderer = Renderer::new(1280.0, 720.0);
    let texture = NullTexture::new(100, 200);
    let (dw, dh, x, _y) = renderer.calculate_draw_rect_for(&texture, DrawMode::Cover);
    assert!((dw - 1280.0).abs() < 0.01);
    assert!((dh - 2560.0).abs() < 0.01);
    assert!((x - 0.0).abs() < 0.01);
}

#[test]
fn test_calculate_draw_rect_contain() {
    let renderer = Renderer::new(1280.0, 720.0);
    let texture = NullTexture::new(640, 360);
    let (dw, dh, x, y) = renderer.calculate_draw_rect_for(&texture, DrawMode::Contain);
    assert!((dw - 1280.0).abs() < 0.01);
    assert!((dh - 720.0).abs() < 0.01);
    assert!((x - 0.0).abs() < 0.01);
    assert!((y - 0.0).abs() < 0.01);
}

#[test]
fn test_calculate_draw_rect_stretch() {
    let renderer = Renderer::new(1280.0, 720.0);
    let texture = NullTexture::new(100, 100); // irrelevant size
    let (dw, dh, x, y) = renderer.calculate_draw_rect_for(&texture, DrawMode::Stretch);
    assert!((dw - 1280.0).abs() < 0.01);
    assert!((dh - 720.0).abs() < 0.01);
    assert!((x - 0.0).abs() < 0.01);
    assert!((y - 0.0).abs() < 0.01);
}

// ============ Renderer 公共方法边界测试 ============

/// build_draw_commands 测试用：1280×720 的 renderer、默认 resource_manager 与 manifest。
fn build_draw_deps() -> (Renderer, ResourceManager, Manifest) {
    (
        Renderer::new(1280.0, 720.0),
        make_test_resource_manager(),
        Manifest::with_defaults(),
    )
}

#[test]
fn test_screen_size_defaults_to_design_size() {
    let renderer = Renderer::new(1920.0, 1080.0);
    assert!((renderer.screen_width() - 1920.0).abs() < 0.01);
    assert!((renderer.screen_height() - 1080.0).abs() < 0.01);
}

#[test]
fn test_set_screen_size_updates_getters() {
    let mut renderer = Renderer::new(1920.0, 1080.0);
    renderer.set_screen_size(1280.0, 720.0);
    assert!((renderer.screen_width() - 1280.0).abs() < 0.01);
    assert!((renderer.screen_height() - 720.0).abs() < 0.01);
}

#[test]
fn test_position_to_preset_name_all_variants() {
    use vn_runtime::command::Position;
    assert_eq!(position_to_preset_name(Position::Left), "left");
    assert_eq!(position_to_preset_name(Position::NearLeft), "nearleft");
    assert_eq!(position_to_preset_name(Position::FarLeft), "farleft");
    assert_eq!(position_to_preset_name(Position::Center), "center");
    assert_eq!(position_to_preset_name(Position::NearMiddle), "nearmiddle");
    assert_eq!(position_to_preset_name(Position::FarMiddle), "farmiddle");
    assert_eq!(position_to_preset_name(Position::Right), "right");
    assert_eq!(position_to_preset_name(Position::NearRight), "nearright");
    assert_eq!(position_to_preset_name(Position::FarRight), "farright");
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
