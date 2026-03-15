use super::*;

#[test]
fn default_layout_has_ref_project_values() {
    let config = UiLayoutConfig::default();
    assert_eq!(config.base_width, 1920.0);
    assert_eq!(config.base_height, 1080.0);
    assert_eq!(config.fonts.text_size, 33.0);
    assert_eq!(config.fonts.name_text_size, 45.0);
    assert_eq!(config.dialogue.textbox_height, 278.0);
    assert_eq!(config.dialogue.dialogue_xpos, 402.0);
    assert_eq!(config.dialogue.dialogue_width, 1116.0);
    assert_eq!(config.choice.button_width, 1185.0);
    assert_eq!(config.save_load.cols, 3);
    assert_eq!(config.save_load.rows, 2);
    assert_eq!(config.history.entry_height, 210.0);
}

#[test]
fn scale_context_identity() {
    let sc = ScaleContext::new(1920.0, 1080.0, 1920.0, 1080.0);
    assert!((sc.x(100.0) - 100.0).abs() < 0.001);
    assert!((sc.y(100.0) - 100.0).abs() < 0.001);
    assert!((sc.uniform(100.0) - 100.0).abs() < 0.001);
}

#[test]
fn scale_context_half() {
    let sc = ScaleContext::new(1920.0, 1080.0, 960.0, 540.0);
    assert!((sc.x(100.0) - 50.0).abs() < 0.001);
    assert!((sc.y(100.0) - 50.0).abs() < 0.001);
    assert!((sc.uniform(100.0) - 50.0).abs() < 0.001);
}

#[test]
fn scale_context_non_uniform() {
    let sc = ScaleContext::new(1920.0, 1080.0, 1280.0, 720.0);
    let rect = sc.rect(10.0, 20.0, 100.0, 200.0);
    assert!((rect.min.x - 10.0 * 1280.0 / 1920.0).abs() < 0.1);
    assert!((rect.min.y - 20.0 * 720.0 / 1080.0).abs() < 0.1);
}

#[test]
fn hex_color_parsing() {
    let c = HexColor("#ff9900".into()).to_egui();
    assert_eq!(c, egui::Color32::from_rgb(255, 153, 0));

    let c2 = HexColor("#7878787f".into()).to_egui();
    assert_eq!(
        c2,
        egui::Color32::from_rgba_unmultiplied(120, 120, 120, 127)
    );

    let c3 = HexColor("#000000".into()).to_egui();
    assert_eq!(c3, egui::Color32::from_rgb(0, 0, 0));
}

#[test]
fn json_missing_field_returns_error() {
    let json = r#"{ "fonts": { "text_size": 40.0 } }"#;
    let result = serde_json::from_str::<UiLayoutConfig>(json);
    assert!(
        result.is_err(),
        "partial JSON should fail without serde(default)"
    );
}

#[test]
fn json_unknown_field_returns_error() {
    let json = r#"{ "text_size": 33.0, "name_text_size": 45.0, "interface_text_size": 33.0, "label_text_size": 36.0, "notify_text_size": 24.0, "title_text_size": 75.0, "bogus": true }"#;
    let result = serde_json::from_str::<FontConfig>(json);
    assert!(
        result.is_err(),
        "unknown field should fail with deny_unknown_fields"
    );
}

#[test]
fn asset_paths_all_entries_count() {
    let paths = UiAssetPaths::default();
    assert_eq!(paths.all_entries().len(), 23);
}
