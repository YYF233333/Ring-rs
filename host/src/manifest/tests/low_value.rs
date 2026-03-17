use super::*;
use std::io::Write;

// ============ 默认值 / 预设 / getter ============

#[test]
fn test_get_group_config_default() {
    let manifest = Manifest::with_defaults();

    // 未配置的立绘应返回默认值
    let config = manifest.get_group_config("characters/未知角色.png");
    assert!((config.anchor.x - 0.5).abs() < 0.01);
    assert!((config.anchor.y - 1.0).abs() < 0.01);
    assert!((config.pre_scale - 1.0).abs() < 0.01);
}

#[test]
fn test_with_defaults_has_nine_presets() {
    let manifest = Manifest::with_defaults();
    assert_eq!(manifest.presets.len(), 9);
    for name in [
        "left",
        "nearleft",
        "farleft",
        "center",
        "nearmiddle",
        "farmiddle",
        "right",
        "nearright",
        "farright",
    ] {
        assert!(manifest.presets.contains_key(name), "缺少预设: {name}");
    }
}

#[test]
fn test_get_preset_known() {
    let manifest = Manifest::with_defaults();
    let preset = manifest.get_preset("center");
    assert!((preset.x - 0.5).abs() < 0.01);
    assert!((preset.y - 0.95).abs() < 0.01);
}

#[test]
fn test_get_preset_unknown_returns_default() {
    let manifest = Manifest::with_defaults();
    let preset = manifest.get_preset("nonexistent");
    assert!((preset.x - 0.5).abs() < 0.01);
    assert!((preset.scale - 1.0).abs() < 0.01);
}

// ============ load schema / 成功路径 ============

#[test]
fn test_load_from_bytes_valid_json() {
    let json = r#"{
        "presets": {
            "custom": { "x": 0.3, "y": 0.8, "scale": 1.2 }
        }
    }"#;
    let manifest = Manifest::load_from_bytes(json.as_bytes()).expect("应该成功解析");
    let preset = manifest.presets.get("custom").expect("应该有 custom 预设");
    assert!((preset.x - 0.3).abs() < 0.01);
    assert!((preset.scale - 1.2).abs() < 0.01);
}

#[test]
fn test_load_from_bytes_empty_object() {
    let json = r#"{}"#;
    let manifest = Manifest::load_from_bytes(json.as_bytes()).expect("空 JSON 应该成功");
    assert!(manifest.presets.is_empty());
}

#[test]
fn test_load_from_file_success() {
    let mut tmp = tempfile::NamedTempFile::new().expect("创建临时文件失败");
    write!(
        tmp,
        r#"{{"presets": {{"stage": {{"x": 0.5, "y": 0.9, "scale": 0.9}}}}}}"#
    )
    .unwrap();
    let path = tmp.path().to_str().unwrap().to_string();

    let manifest = Manifest::load(&path).expect("应该成功加载");
    assert!(manifest.presets.contains_key("stage"));
}

// ============ ManifestWarning Display ============

#[test]
fn test_manifest_warning_display_invalid_anchor() {
    let w = ManifestWarning::InvalidAnchor {
        context: "test.anchor".to_string(),
        x: 1.5,
        y: 0.5,
    };
    let s = format!("{w}");
    assert!(s.contains("test.anchor"));
    assert!(s.contains("1.5"));
}

#[test]
fn test_manifest_warning_display_invalid_pre_scale() {
    let w = ManifestWarning::InvalidPreScale {
        context: "test.pre_scale".to_string(),
        value: 0.0,
    };
    let s = format!("{w}");
    assert!(s.contains("test.pre_scale"));
    assert!(s.contains("0"));
}

#[test]
fn test_manifest_warning_display_invalid_preset_position() {
    let w = ManifestWarning::InvalidPresetPosition {
        context: "presets.bad.x".to_string(),
        value: 1.5,
    };
    let s = format!("{w}");
    assert!(s.contains("presets.bad.x"));
}

#[test]
fn test_manifest_warning_display_unknown_group() {
    let w = ManifestWarning::UnknownGroup {
        sprite_path: "characters/a.png".to_string(),
        group_id: "missing".to_string(),
    };
    let s = format!("{w}");
    assert!(s.contains("characters/a.png"));
    assert!(s.contains("missing"));
}
