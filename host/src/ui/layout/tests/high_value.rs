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
