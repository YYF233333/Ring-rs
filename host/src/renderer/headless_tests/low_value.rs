//! 低价值测试：几何计算、getter/setter、枚举到字符串映射。

use super::*;

#[test]
fn test_get_choice_rects_correct_count() {
    let renderer = Renderer::new(1280.0, 720.0);
    let rects = renderer.get_choice_rects(3);
    assert_eq!(rects.len(), 3);
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
