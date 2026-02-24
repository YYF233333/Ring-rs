use super::*;

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
