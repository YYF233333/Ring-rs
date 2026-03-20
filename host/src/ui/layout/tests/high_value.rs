use super::*;

#[test]
fn json_invalid_returns_error() {
    assert!(
        serde_json::from_str::<UiLayoutConfig>(r#"{ "fonts": { "text_size": 40.0 } }"#).is_err()
    );
    let font_json = r#"{ "text_size": 33.0, "name_text_size": 45.0, "interface_text_size": 33.0, "label_text_size": 36.0, "notify_text_size": 24.0, "title_text_size": 75.0, "bogus": true }"#;
    assert!(serde_json::from_str::<FontConfig>(font_json).is_err());
}

#[test]
fn hex_color_invalid_len_returns_white() {
    assert_eq!(HexColor("#fff".into()).to_egui(), egui::Color32::WHITE);
}

#[test]
fn scale_context() {
    // 等比 1:1
    let sc = ScaleContext::new(1920.0, 1080.0, 1920.0, 1080.0);
    assert!((sc.x(100.0) - 100.0).abs() < 0.001);
    assert!((sc.y(100.0) - 100.0).abs() < 0.001);
    assert!((sc.uniform(100.0) - 100.0).abs() < 0.001);
    // 等比 0.5
    let sc = ScaleContext::new(1920.0, 1080.0, 960.0, 540.0);
    assert!((sc.x(100.0) - 50.0).abs() < 0.001);
    assert!((sc.y(100.0) - 50.0).abs() < 0.001);
    // 非等比 + rect
    let sc = ScaleContext::new(1920.0, 1080.0, 1280.0, 720.0);
    let rect = sc.rect(10.0, 20.0, 100.0, 200.0);
    assert!((rect.min.x - 10.0 * 1280.0 / 1920.0).abs() < 0.1);
    assert!((rect.min.y - 20.0 * 720.0 / 1080.0).abs() < 0.1);
}

#[test]
fn hex_color_parsing() {
    assert_eq!(
        HexColor("#ff9900".into()).to_egui(),
        egui::Color32::from_rgb(255, 153, 0)
    );
    assert_eq!(
        HexColor("#7878787f".into()).to_egui(),
        egui::Color32::from_rgba_unmultiplied(120, 120, 120, 127)
    );
    assert_eq!(
        HexColor("#000000".into()).to_egui(),
        egui::Color32::from_rgb(0, 0, 0)
    );
}

#[test]
fn scale_context_vec2() {
    let sc = ScaleContext::new(1920.0, 1080.0, 960.0, 540.0);
    let v = sc.vec2(100.0, 200.0);
    assert!((v.x - 50.0).abs() < 0.001);
    assert!((v.y - 100.0).abs() < 0.001);
}

#[test]
fn scale_context_uniform_takes_min() {
    let sc = ScaleContext::new(1000.0, 1000.0, 2000.0, 1000.0);
    assert!((sc.uniform(100.0) - 100.0).abs() < 0.001);
}

#[test]
fn hex_color_uppercase() {
    let color = HexColor("#FF9900".into()).to_egui();
    assert_eq!(color, egui::Color32::from_rgb(255, 153, 0));
}

#[test]
fn hex_color_alpha_zero() {
    let color = HexColor("#00000000".into()).to_egui();
    assert_eq!(color, egui::Color32::from_rgba_unmultiplied(0, 0, 0, 0));
}
