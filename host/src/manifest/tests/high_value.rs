use super::*;

// ============ infer_group_id 契约与路径推导 ============

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

// ============ load 错误分类与边界 ============

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
    std::io::Write::write_all(&mut tmp, b"this is not json").unwrap();
    let path = tmp.path().to_str().unwrap().to_string();

    let result = Manifest::load(&path);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("解析"));
}

// ============ validate 不变量与错误分类 ============

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
