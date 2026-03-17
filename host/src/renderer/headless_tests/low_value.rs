//! 低价值测试：几何计算、getter/setter、枚举到字符串映射。

use super::*;

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
