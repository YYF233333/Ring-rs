use super::*;
use std::io::Write;

// ============ infer_group_id 测试 ============

#[test]
fn test_infer_group_id() {
    let manifest = Manifest::with_defaults();

    // 从文件名推导
    assert_eq!(
        manifest.infer_group_id("characters/北风-日常服.png"),
        "北风"
    );
    assert_eq!(manifest.infer_group_id("characters/路汐_笑颜.png"), "路汐");
    assert_eq!(manifest.infer_group_id("characters/测试.png"), "测试");

    // 从目录推导
    assert_eq!(manifest.infer_group_id("北风/日常服.png"), "北风");
}

#[test]
fn test_get_group_config_explicit() {
    let mut manifest = Manifest::with_defaults();

    // 添加显式配置
    manifest.characters.groups.insert(
        "北风".to_string(),
        GroupConfig {
            anchor: Point2D { x: 0.5, y: 0.9 },
            pre_scale: 0.8,
        },
    );
    manifest
        .characters
        .sprites
        .insert("characters/北风-日常服.png".to_string(), "北风".to_string());

    let config = manifest.get_group_config("characters/北风-日常服.png");
    assert!((config.anchor.y - 0.9).abs() < 0.01);
    assert!((config.pre_scale - 0.8).abs() < 0.01);
}

#[test]
fn test_get_group_config_inferred() {
    let mut manifest = Manifest::with_defaults();

    // 只添加组配置，不添加显式映射
    manifest.characters.groups.insert(
        "北风".to_string(),
        GroupConfig {
            anchor: Point2D { x: 0.5, y: 0.85 },
            pre_scale: 0.75,
        },
    );

    // 通过文件名推导应该能找到
    let config = manifest.get_group_config("characters/北风-惊讶.png");
    assert!((config.pre_scale - 0.75).abs() < 0.01);
}

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
fn test_get_group_config_explicit_with_relative_segments() {
    let mut manifest = Manifest::with_defaults();

    manifest.characters.groups.insert(
        "红叶".to_string(),
        GroupConfig {
            anchor: Point2D { x: 0.5, y: 0.35 },
            pre_scale: 0.1,
        },
    );
    manifest.characters.sprites.insert(
        "characters/立绘红叶/夏装/角色夏收手4.webp".to_string(),
        "红叶".to_string(),
    );

    let config = manifest.get_group_config(
        "scripts/remake/ring/summer/../../../../characters/立绘红叶/夏装/角色夏收手4.webp",
    );
    assert!((config.anchor.y - 0.35).abs() < 0.01);
    assert!((config.pre_scale - 0.1).abs() < 0.01);
}

#[test]
fn test_infer_group_id_no_separator() {
    let manifest = Manifest::with_defaults();
    assert_eq!(manifest.infer_group_id("characters/Alice.png"), "Alice");
}

#[test]
fn test_infer_group_id_from_parent_directory() {
    let manifest = Manifest::with_defaults();
    assert_eq!(
        manifest.infer_group_id("sprites/角色甲/normal.png"),
        "角色甲"
    );
}

// ============ with_defaults 预设验证 ============

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

// ============ load_from_bytes 测试 ============

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
fn test_load_from_bytes_invalid_json_returns_error() {
    let result = Manifest::load_from_bytes(b"not valid json");
    assert!(result.is_err());
    let msg = result.unwrap_err();
    assert!(msg.contains("解析") || msg.contains("JSON") || msg.contains("parse"));
}

#[test]
fn test_load_from_bytes_invalid_utf8_returns_error() {
    let invalid_utf8 = vec![0xFF, 0xFE, 0x00];
    let result = Manifest::load_from_bytes(&invalid_utf8);
    assert!(result.is_err());
}

// ============ load（文件）测试 ============

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

#[test]
fn test_load_from_file_not_found_returns_error() {
    let result = Manifest::load("/nonexistent/path/to/manifest.json");
    assert!(result.is_err());
    let msg = result.unwrap_err();
    assert!(msg.contains("无法读取"));
}

#[test]
fn test_load_from_file_invalid_json_returns_error() {
    let mut tmp = tempfile::NamedTempFile::new().expect("创建临时文件失败");
    write!(tmp, "this is not json").unwrap();
    let path = tmp.path().to_str().unwrap().to_string();

    let result = Manifest::load(&path);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("解析"));
}

// ============ validate 测试 ============

#[test]
fn test_validate_clean_manifest_no_warnings() {
    let manifest = Manifest::with_defaults();
    let warnings = manifest.validate();
    assert!(
        warnings.is_empty(),
        "默认 manifest 不应有警告: {:?}",
        warnings
    );
}

#[test]
fn test_validate_invalid_anchor() {
    let mut manifest = Manifest::with_defaults();
    manifest.defaults.anchor = Point2D { x: 1.5, y: 0.5 }; // x > 1.0 无效
    let warnings = manifest.validate();
    assert!(
        warnings
            .iter()
            .any(|w| matches!(w, ManifestWarning::InvalidAnchor { .. })),
        "应该产生 InvalidAnchor 警告"
    );
}

#[test]
fn test_validate_invalid_pre_scale_zero() {
    let mut manifest = Manifest::with_defaults();
    manifest.defaults.pre_scale = 0.0;
    let warnings = manifest.validate();
    assert!(
        warnings
            .iter()
            .any(|w| matches!(w, ManifestWarning::InvalidPreScale { .. })),
        "pre_scale=0 应该产生警告"
    );
}

#[test]
fn test_validate_invalid_pre_scale_negative() {
    let mut manifest = Manifest::with_defaults();
    manifest.defaults.pre_scale = -1.0;
    let warnings = manifest.validate();
    assert!(
        warnings
            .iter()
            .any(|w| matches!(w, ManifestWarning::InvalidPreScale { .. })),
        "负的 pre_scale 应该产生警告"
    );
}

#[test]
fn test_validate_invalid_preset_position() {
    let mut manifest = Manifest::with_defaults();
    manifest.presets.insert(
        "bad".to_string(),
        PositionPreset {
            x: 1.5,
            y: 0.5,
            scale: 1.0,
        },
    );
    let warnings = manifest.validate();
    assert!(
        warnings
            .iter()
            .any(|w| matches!(w, ManifestWarning::InvalidPresetPosition { .. })),
        "预设 x > 1.0 应该产生警告"
    );
}

#[test]
fn test_validate_invalid_preset_scale() {
    let mut manifest = Manifest::with_defaults();
    manifest.presets.insert(
        "zero_scale".to_string(),
        PositionPreset {
            x: 0.5,
            y: 0.5,
            scale: 0.0,
        },
    );
    let warnings = manifest.validate();
    assert!(
        warnings
            .iter()
            .any(|w| matches!(w, ManifestWarning::InvalidPreScale { .. })),
        "预设 scale=0 应该产生警告"
    );
}

#[test]
fn test_validate_unknown_group_in_sprite() {
    let mut manifest = Manifest::with_defaults();
    manifest.characters.sprites.insert(
        "characters/some.png".to_string(),
        "nonexistent_group".to_string(),
    );
    let warnings = manifest.validate();
    assert!(
        warnings
            .iter()
            .any(|w| matches!(w, ManifestWarning::UnknownGroup { .. })),
        "引用不存在的组应该产生警告"
    );
}

#[test]
fn test_validate_group_with_invalid_anchor() {
    let mut manifest = Manifest::with_defaults();
    manifest.characters.groups.insert(
        "bad_group".to_string(),
        GroupConfig {
            anchor: Point2D { x: -0.1, y: 0.5 }, // x < 0 无效
            pre_scale: 1.0,
        },
    );
    let warnings = manifest.validate();
    assert!(
        warnings
            .iter()
            .any(|w| matches!(w, ManifestWarning::InvalidAnchor { .. })),
        "组锚点 x < 0 应该产生警告"
    );
}

// ============ ManifestWarning Display 测试 ============

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
